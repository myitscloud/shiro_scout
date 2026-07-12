//! Persistent PTY shell sessions inside a Docker sandbox container.
//! Item 4.3 — builds on the bollard exec bridge (4.2) to maintain long-running
//! /bin/bash processes inside the sandbox. State (cwd, env, venv) is preserved
//! between commands because the same shell process stays alive.
//!
//! Design per TRUE-STATE-PRESERVATION.md:
//! - Each session runs a persistent /bin/bash inside the container via bollard exec
//! - Commands sent to stdin with a unique delimiter marker
//! - Output collected from stdout/stderr until the marker appears
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use bollard::container::LogOutput;
use bollard::exec::{CreateExecOptions, StartExecResults};
use chrono::{DateTime, Utc};
use futures::StreamExt;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex as AsyncMutex;

use crate::docker_client;

// --------------------------------------------------------------------------
// Error type (C7: typed errors, C10: sanitized before IPC)
// --------------------------------------------------------------------------

#[derive(Debug)]
pub enum PtyError {
    DockerError(String),
    SessionNotFound(String),
    ExecFailed(String),
    Timeout(String),
}

impl std::fmt::Display for PtyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PtyError::DockerError(e) => write!(f, "Docker connection error: {}", e),
            PtyError::SessionNotFound(id) => write!(f, "Session not found: {}", id),
            PtyError::ExecFailed(e) => write!(f, "Exec failed: {}", e),
            PtyError::Timeout(e) => write!(f, "Command timed out: {}", e),
        }
    }
}

impl From<PtyError> for String {
    fn from(e: PtyError) -> Self {
        e.to_string()
    }
}

// --------------------------------------------------------------------------
// Data structures
// --------------------------------------------------------------------------

/// A single persistent shell session inside a sandbox container.
pub struct ContainerShellSession {
    pub id: String,
    pub container_id: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
    /// Async writer to the shell's stdin (Docker exec pipe)
    stdin_writer: Arc<AsyncMutex<Pin<Box<dyn tokio::io::AsyncWrite + Send>>>>,
    /// Shared output buffer filled by background reader
    output_buffer: Arc<Mutex<String>>,
}

// Manual Pin boxing helper
use std::pin::Pin;

/// Manages multiple persistent PTY sessions.
pub struct PtyManager {
    sessions: tokio::sync::RwLock<HashMap<String, ContainerShellSession>>,
    next_id: AtomicU64,
}

impl Default for PtyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            sessions: tokio::sync::RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    fn generate_id(&self) -> String {
        let n = self.next_id.fetch_add(1, Ordering::SeqCst);
        format!("pty-{}", n)
    }

    /// Create a new persistent shell session inside the given container.
    pub async fn create_session(&self, container_id: &str) -> Result<String, PtyError> {
        let docker = docker_client::get_docker()
            .map_err(|e| PtyError::DockerError(e.clone()))?;

        // Create exec instance for a long-running bash with stdin attached
        let exec = docker
            .create_exec(
                container_id,
                CreateExecOptions {
                    attach_stdin: Some(true),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    cmd: Some(vec!["/bin/bash".to_string()]),
                    working_dir: Some("/workspace".to_string()),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| PtyError::ExecFailed(format!("create exec: {}", e)))?;

        // Start the exec and get attached I/O streams
        let result = docker
            .start_exec(&exec.id, None)
            .await
            .map_err(|e| PtyError::ExecFailed(format!("start exec: {}", e)))?;

        let (output_stream, input_writer) = match result {
            StartExecResults::Attached { output, input } => (output, input),
            StartExecResults::Detached => {
                return Err(PtyError::ExecFailed(
                    "exec started detached (expected attached)".to_string(),
                ));
            }
        };

        let session_id = self.generate_id();
        let output_buffer: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
        let bg_buffer = output_buffer.clone();
        let bg_session_id = session_id.clone();

        // Spawn background task to continuously read output from the shell
        tokio::spawn(async move {
            let mut stream = output_stream;
            loop {
                match stream.next().await {
                    Some(Ok(LogOutput::StdOut { message }))
                    | Some(Ok(LogOutput::StdErr { message })) => {
                        if let Ok(text) = String::from_utf8(message.to_vec()) {
                            bg_buffer.lock().unwrap().push_str(&text);
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(e)) => {
                        eprintln!("[pty:{}] stream error: {}", bg_session_id, e);
                        break;
                    }
                    None => break,
                }
            }
            eprintln!("[pty:{}] background reader terminated", bg_session_id);
        });

        let session = ContainerShellSession {
            id: session_id.clone(),
            container_id: container_id.to_string(),
            created_at: Utc::now(),
            last_used_at: Utc::now(),
            stdin_writer: Arc::new(AsyncMutex::new(input_writer)),
            output_buffer,
        };

        self.sessions
            .write()
            .await
            .insert(session_id.clone(), session);

        Ok(session_id)
    }

    /// Execute a command in a persistent session using delimiter detection.
    pub async fn execute_command(
        &self,
        session_id: &str,
        command: &str,
        timeout_secs: u64,
    ) -> Result<String, PtyError> {
        let (start_len, stdin_writer, output_buffer) = {
            let sessions = self.sessions.read().await;
            let session = sessions
                .get(session_id)
                .ok_or_else(|| PtyError::SessionNotFound(session_id.to_string()))?;

            let start_len = session.output_buffer.lock().unwrap().len();
            let stdin_writer = session.stdin_writer.clone();
            let output_buffer = session.output_buffer.clone();
            (start_len, stdin_writer, output_buffer)
        };

        // Generate unique delimiter for this command
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let delimiter = format!("__PTY_DONE_{}__", now);

        // Send command + delimiter to the shell's stdin
        let cmd_line = format!("{}\necho {}\n", command, delimiter);
        {
            let mut writer = stdin_writer
                .lock()
                .await;
            writer
                .write_all(cmd_line.as_bytes())
                .await
                .map_err(|e| PtyError::ExecFailed(format!("write stdin: {}", e)))?;
            writer
                .flush()
                .await
                .map_err(|e| PtyError::ExecFailed(format!("flush stdin: {}", e)))?;
        }

        // Poll buffer until delimiter appears or timeout
        let timeout = Duration::from_secs(timeout_secs);
        let start = std::time::Instant::now();

        loop {
            tokio::time::sleep(Duration::from_millis(50)).await;

            let buffer_content = output_buffer.lock().unwrap().clone();

            if let Some(delim_pos) = buffer_content.find(&delimiter) {
                // Extract only the new output (from start position to delimiter)
                let raw_output = if delim_pos > start_len {
                    buffer_content[start_len..delim_pos].to_string()
                } else {
                    String::new()
                };
                return Ok(clean_terminal_output(&raw_output));
            }

            if start.elapsed() > timeout {
                return Err(PtyError::Timeout(format!(
                    "command execution exceeded {}s",
                    timeout_secs
                )));
            }
        }
    }

    /// Close and clean up a session.
    pub async fn close_session(&self, session_id: &str) -> Result<(), PtyError> {
        let mut sessions = self.sessions.write().await;
        sessions
            .remove(session_id)
            .ok_or_else(|| PtyError::SessionNotFound(session_id.to_string()))?;
        // Background reader task terminates automatically when exec stream ends
        Ok(())
    }

    /// List all active session IDs.
    pub async fn list_sessions(&self) -> Vec<String> {
        let sessions = self.sessions.read().await;
        sessions.keys().cloned().collect()
    }

    /// Return count of active sessions.
    pub async fn session_count(&self) -> usize {
        self.sessions.read().await.len()
    }
}

/// Strip empty trailing lines from raw terminal output.
fn clean_terminal_output(raw: &str) -> String {
    raw.trim_end_matches(['\n', '\r']).to_string()
}

// --------------------------------------------------------------------------
// Global singleton for non-Tauri code access
// --------------------------------------------------------------------------

fn pty_manager() -> &'static PtyManager {
    static INSTANCE: std::sync::OnceLock<PtyManager> = std::sync::OnceLock::new();
    INSTANCE.get_or_init(PtyManager::new)
}

/// Internal API for bridge_client.rs to create a session.
pub async fn create_session_internal(container_id: &str) -> Result<String, PtyError> {
    pty_manager().create_session(container_id).await
}

/// Internal API for bridge_client.rs to execute a command in a session.
pub async fn execute_session_internal(
    session_id: &str,
    command: &str,
    timeout_secs: u64,
) -> Result<String, PtyError> {
    pty_manager()
        .execute_command(session_id, command, timeout_secs)
        .await
}

/// Internal API for bridge_client.rs to close a session.
pub async fn close_session_internal(session_id: &str) -> Result<(), PtyError> {
    pty_manager().close_session(session_id).await
}

// --------------------------------------------------------------------------
// Tauri commands
// --------------------------------------------------------------------------

/// Create a new persistent PTY session in the sandbox container.
#[tauri::command]
pub async fn create_pty_session(container_id: String) -> Result<String, String> {
    create_session_internal(&container_id)
        .await
        .map_err(|e| e.to_string())
}

/// Execute a command in a persistent PTY session.
#[tauri::command]
pub async fn pty_execute_command(
    session_id: String,
    command: String,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let timeout = timeout_secs.unwrap_or(60);
    execute_session_internal(&session_id, &command, timeout)
        .await
        .map_err(|e| e.to_string())
}

/// Close a persistent PTY session.
#[tauri::command]
pub async fn close_pty_session(session_id: String) -> Result<(), String> {
    close_session_internal(&session_id)
        .await
        .map_err(|e| e.to_string())
}

/// List all active PTY sessions.
#[tauri::command]
pub async fn list_pty_sessions() -> Result<Vec<String>, String> {
    Ok(pty_manager().list_sessions().await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pty_error_display_connection() {
        let err = PtyError::DockerError("connection refused".into());
        let msg = format!("{}", err);
        assert!(msg.contains("connection refused"));
    }

    #[test]
    fn test_pty_error_display_session_not_found() {
        let err = PtyError::SessionNotFound("pty-42".into());
        let msg = format!("{}", err);
        assert!(msg.contains("pty-42"));
    }

    #[test]
    fn test_pty_error_display_exec_failed() {
        let err = PtyError::ExecFailed("bash not found".into());
        let msg = format!("{}", err);
        assert!(msg.contains("bash not found"));
    }

    #[test]
    fn test_pty_error_display_timeout() {
        let err = PtyError::Timeout("30s".into());
        let msg = format!("{}", err);
        assert!(msg.contains("30s"));
    }

    #[test]
    fn test_pty_error_into_string() {
        let err = PtyError::SessionNotFound("missing".into());
        let s: String = err.into();
        assert!(s.contains("missing"));
    }

    #[test]
    fn test_pty_manager_new() {
        let mgr = PtyManager::new();
        // Should be empty initially
        let sessions = futures::executor::block_on(mgr.list_sessions());
        assert!(sessions.is_empty());
        let count = futures::executor::block_on(mgr.session_count());
        assert_eq!(count, 0);
    }

    #[test]
    fn test_pty_manager_generate_id() {
        let mgr = PtyManager::new();
        let id1 = mgr.generate_id();
        assert!(id1.starts_with("pty-"));
        let id2 = mgr.generate_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_clean_terminal_output() {
        assert_eq!(clean_terminal_output("hello\n"), "hello");
        assert_eq!(clean_terminal_output("hello\n\n"), "hello");
        assert_eq!(clean_terminal_output("hello\r\n"), "hello");
        assert_eq!(clean_terminal_output("hello"), "hello");
        assert_eq!(clean_terminal_output(""), "");
    }
}
