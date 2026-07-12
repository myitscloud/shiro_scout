use bollard::container::{
    Config, CreateContainerOptions, LogOutput, RemoveContainerOptions, StopContainerOptions,
};
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::image::CreateImageOptions;
use bollard::models::HostConfig;
use bollard::Docker;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::docker_client;

// --------------------------------------------------------------------------
// Custom error type for sandbox operations
// --------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("Docker API error: {0}")]
    Docker(#[from] bollard::errors::Error),

    #[error("Container not found: {0}")]
    NotFound(String),

    #[error("Container is not running: {0}")]
    NotRunning(String),

    #[error("Operation timed out: {0}")]
    Timeout(String),

    #[error("Exec session failed: {0}")]
    ExecFailed(String),

    #[error("Bridge error: {0}")]
    Bridge(String),
}

impl From<SandboxError> for String {
    fn from(e: SandboxError) -> Self {
        e.to_string()
    }
}

/// BridgeError originates in bridge_client; this is for exec-specific failures.
pub type ContainerResult<T> = Result<T, SandboxError>;

// --------------------------------------------------------------------------
// Data structures
// --------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum NetworkMode {
    #[serde(rename = "bridge")]
    #[default]
    Bridge,
    #[serde(rename = "none")]
    None,
}



impl NetworkMode {
    /// Returns the Docker host_config network_mode value.
    /// Bridge => None (Docker default), None => Some("none").
    pub fn to_docker_string(&self) -> Option<String> {
        match self {
            NetworkMode::Bridge => None,
            NetworkMode::None => Some("none".to_string()),
        }
    }

    /// Returns true if no network access is configured.
    pub fn is_air_gapped(&self) -> bool {
        matches!(self, Self::None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub image: String,
    pub workspace_path: String,
    pub memory_mb: u64,
    pub cpu_shares: u64,
    pub network_mode: NetworkMode,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            image: "aegis-sandbox:latest".to_string(),
            workspace_path: String::new(),
            memory_mb: 2048,
            cpu_shares: 512,
            network_mode: NetworkMode::None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SandboxCreateResult {
    pub container_id: String,
    pub container_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImagePullProgress {
    pub status: String,
    pub progress: Option<String>,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecResult {
    pub exit_code: i64,
    pub stdout: String,
    pub stderr: String,
}

// --------------------------------------------------------------------------
// Helper: Build HostConfig per ADR-002 security hardening
// --------------------------------------------------------------------------

fn build_host_config(config: &SandboxConfig) -> HostConfig {
    let mut tmpfs_map = HashMap::new();
    tmpfs_map.insert("/tmp".to_string(), "noexec,nosuid,size=256M".to_string());
    tmpfs_map.insert("/run".to_string(), "noexec,nosuid,size=64M".to_string());

    let mut binds = Vec::new();
    if !config.workspace_path.is_empty() {
        binds.push(format!("{}:/workspace:rw", config.workspace_path));
    }

    let mem_bytes = (config.memory_mb * 1024 * 1024) as i64;
    let cpu_quota = (config.cpu_shares as f64 * 100000.0 / 1024.0) as i64;

    let network_mode = config.network_mode.to_docker_string();

    HostConfig {
        readonly_rootfs: Some(true),
        init: Some(true),
        network_mode,
        cap_drop: Some(vec!["ALL".to_string()]),
        security_opt: Some(vec!["no-new-privileges:true".to_string()]),
        pids_limit: Some(256),
        memory: Some(mem_bytes),
        cpu_period: Some(100000),
        cpu_quota: Some(cpu_quota),
        tmpfs: Some(tmpfs_map),
        binds: Some(binds),
        ..Default::default()
    }
}

// --------------------------------------------------------------------------
// Exec API — run a command inside a sandbox container
// --------------------------------------------------------------------------

/// Execute a command inside a running container and return its output.
pub async fn exec_in_container(
    container_id: &str,
    cmd: &[String],
) -> Result<ExecResult, SandboxError> {
    let docker = docker_client::get_docker()
        .map_err(|e| SandboxError::Bridge(format!("Docker connection: {}", e)))?;

    // Create exec instance
    let exec = docker
        .create_exec(
            container_id,
            CreateExecOptions {
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                cmd: Some(cmd.to_vec()),
                ..Default::default()
            },
        )
        .await
        .map_err(SandboxError::Docker)?;

    // Start exec and collect output
    let output = docker
        .start_exec(&exec.id, None)
        .await
        .map_err(SandboxError::Docker)?;

    let mut stdout = String::new();
    let mut stderr = String::new();

    match output {
        StartExecResults::Attached { mut output, .. } => {
            while let Some(frame) = output.next().await {
                match frame {
                    Ok(log) => {
                        match log {
                            LogOutput::StdOut { message } => {
                                stdout.push_str(&String::from_utf8_lossy(&message));
                            }
                            LogOutput::StdErr { message } => {
                                stderr.push_str(&String::from_utf8_lossy(&message));
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        return Err(SandboxError::ExecFailed(format!(
                            "Exec stream error: {}",
                            e
                        )));
                    }
                }
            }
        }
        StartExecResults::Detached => {
            // Detached exec — poll for exit code via inspect
            let info = docker
                .inspect_exec(&exec.id)
                .await
                .map_err(SandboxError::Docker)?;
            let exit_code = info.exit_code.unwrap_or(-1);
            return Ok(ExecResult {
                exit_code,
                stdout,
                stderr,
            });
        }
    }

    // Get exit code from inspect
    let info = docker
        .inspect_exec(&exec.id)
        .await
        .map_err(SandboxError::Docker)?;
    let exit_code = info.exit_code.unwrap_or(-1);

    Ok(ExecResult {
        exit_code,
        stdout,
        stderr,
    })
}

/// Execute a shell command inside a container (convenience wrapper).
pub async fn exec_shell_in_container(
    container_id: &str,
    shell_cmd: &str,
) -> Result<ExecResult, SandboxError> {
    let cmd = vec!["/bin/sh".to_string(), "-c".to_string(), shell_cmd.to_string()];
    exec_in_container(container_id, &cmd).await
}

// --------------------------------------------------------------------------
// Commands
// --------------------------------------------------------------------------

// --------------------------------------------------------------------------
// Helper: Resolve Docker build context directory
// --------------------------------------------------------------------------

/// Resolve the Docker build context directory containing Dockerfile.sandbox,
/// entrypoint.sh, and the agent-bridge binary.
fn resolve_docker_context() -> Result<std::path::PathBuf, String> {
    use std::path::Path;

    let mut candidates: Vec<std::path::PathBuf> = Vec::new();

    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("src-tauri").join("docker"));
        candidates.push(cwd.join("..").join("src-tauri").join("docker"));
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            candidates.push(exe_dir.join("..").join("..").join("src-tauri").join("docker"));
            candidates.push(exe_dir.join("..").join("..").join("..").join("src-tauri").join("docker"));
        }
    }

    for path in &candidates {
        if path.join("Dockerfile.sandbox").exists() {
            return Ok(path.clone());
        }
    }

    if Path::new("docker/Dockerfile.sandbox").exists() {
        return Ok(Path::new("docker").to_path_buf());
    }

    Err(format!(
        "Could not find Docker build context directory (Dockerfile.sandbox). Searched:\n{}",
        candidates.iter().map(|p| format!("  - {}", p.display())).collect::<Vec<_>>().join("\n")
    ))
}

// --------------------------------------------------------------------------
// Build sandbox image independently (callable from frontend)
// --------------------------------------------------------------------------

/// Build the sandbox Docker image from Dockerfile.sandbox.
/// Returns a success message with the image tag.
#[tauri::command]
pub async fn build_sandbox_image() -> Result<String, String> {
    let docker_dir = resolve_docker_context()?;
    let dockerfile_path = docker_dir.join("Dockerfile.sandbox");
    let image_tag = "aegis-sandbox:latest";

    if !dockerfile_path.exists() {
        return Err(format!(
            "Dockerfile not found at '{}'",
            dockerfile_path.display()
        ));
    }

    println!("[container] Building sandbox image '{}' from {} ...",
        image_tag, dockerfile_path.display());

    let output = tokio::process::Command::new("docker")
        .arg("build")
        .arg("-f")
        .arg(dockerfile_path.to_string_lossy().as_ref())
        .arg("-t")
        .arg(image_tag)
        .arg(docker_dir.to_string_lossy().as_ref())
        .output()
        .await
        .map_err(|e| format!("Failed to execute 'docker build': {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "Docker build failed (exit code: {}).\nSTDERR:\n{}\nSTDOUT:\n{}",
            output.status.code().unwrap_or(-1), stderr, stdout
        ));
    }

    Ok(format!("Successfully built sandbox image '{}'", image_tag))
}

/// Create a new sandbox container with the given configuration.
///
/// Automatically checks if the required Docker image exists. If the image is
/// missing, it builds it from Dockerfile.sandbox before creating the container.
///
/// Returns the container ID and name.
#[tauri::command]
pub async fn create_sandbox(config: SandboxConfig) -> Result<SandboxCreateResult, String> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| format!("Failed to connect to Docker daemon: {}", e))?;

    // Step 1: Check if the required Docker image already exists
    let image_name = &config.image;
    let image_exists = match docker.inspect_image(image_name).await {
        Ok(_) => true,
        Err(ref e) => {
            let err_msg = e.to_string();
            if err_msg.contains("404")
                || err_msg.to_lowercase().contains("no such image")
                || err_msg.to_lowercase().contains("not found")
            {
                false
            } else {
                return Err(format!(
                    "Failed to check image '{}': {}", image_name, err_msg
                ));
            }
        }
    };

    // Step 2: Build image if it does not exist yet
    if !image_exists {
        let docker_dir = match resolve_docker_context() {
            Ok(d) => d,
            Err(paths_searched) => {
                return Err(format!(
                    "Image '{}' not found and auto-build is not possible.\n\
                     Build the image manually:\n\
                       docker build -f src-tauri/docker/Dockerfile.sandbox \\
                         -t {} src-tauri/docker/\n\
                     Searched paths:\n{}",
                    image_name, image_name, paths_searched
                ));
            }
        };

        let dockerfile_path = docker_dir.join("Dockerfile.sandbox");
        if !dockerfile_path.exists() {
            return Err(format!(
                "Image '{}' not found and Dockerfile is missing at '{}'. \
                 Build the image manually:\n\
                   docker build -f src-tauri/docker/Dockerfile.sandbox \
                     -t {} src-tauri/docker/",
                image_name, dockerfile_path.display(), image_name
            ));
        }

        println!(
            "[container] Image '{}' not found - building from '{}' ...",
            image_name, dockerfile_path.display()
        );

        let build_output = tokio::process::Command::new("docker")
            .arg("build")
            .arg("-f")
            .arg(dockerfile_path.to_string_lossy().as_ref())
            .arg("-t")
            .arg(image_name)
            .arg(docker_dir.to_string_lossy().as_ref())
            .output()
            .await
            .map_err(|e| format!("Failed to execute 'docker build': {}", e))?;

        if !build_output.status.success() {
            let stderr = String::from_utf8_lossy(&build_output.stderr);
            let stdout = String::from_utf8_lossy(&build_output.stdout);
            return Err(format!(
                "Docker build failed for image '{}' (exit code: {}).\n\
                 STDERR:\n{}\nSTDOUT:\n{}",
                image_name,
                build_output.status.code().unwrap_or(-1),
                stderr,
                stdout
            ));
        }

        println!("[container] Successfully built image '{}'", image_name);
    }

    // Step 3: Create the container
    let host_config = build_host_config(&config);

    let container_config = Config {
        image: Some(config.image.clone()),
        host_config: Some(host_config),
        working_dir: Some("/workspace".to_string()),
        user: Some("1000:1000".to_string()),
        tty: Some(true),
        open_stdin: Some(true),
        attach_stdin: Some(true),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        healthcheck: None,
        ..Default::default()
    };

    let options = CreateContainerOptions {
        name: "aegis-sandbox",
        platform: None,
    };

    let response = docker
        .create_container(Some(options), container_config)
        .await
        .map_err(|e| format!("Failed to create container: {}", e))?;

    Ok(SandboxCreateResult {
        container_id: response.id,
        container_name: "aegis-sandbox".to_string(),
    })
}

/// Start a sandbox container by ID.
#[tauri::command]
pub async fn start_sandbox(id: String) -> Result<(), String> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| format!("Failed to connect to Docker daemon: {}", e))?;

    docker
        .start_container::<String>(&id, None)
        .await
        .map_err(|e| format!("Failed to start container '{}': {}", id, e))?;

    Ok(())
}

/// Stop a sandbox container by ID with a 10-second graceful timeout, then force-kill.
#[tauri::command]
pub async fn stop_sandbox(id: String) -> Result<(), String> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| format!("Failed to connect to Docker daemon: {}", e))?;

    // Graceful stop with 10-second timeout
    let stop_opts = StopContainerOptions { t: 10 };
    match docker.stop_container(&id, Some(stop_opts)).await {
        Ok(_) => Ok(()),
        Err(bollard::errors::Error::DockerResponseServerError {
            status_code: 304, ..
        }) => {
            // 304 = already stopped — not an error
            Ok(())
        }
        Err(e) => Err(format!("Failed to stop container '{}': {}", id, e)),
    }
}

/// Remove a sandbox container by ID (force remove if running).
#[tauri::command]
pub async fn remove_sandbox(id: String) -> Result<(), String> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| format!("Failed to connect to Docker daemon: {}", e))?;

    let remove_opts = RemoveContainerOptions {
        force: true,
        v: true,
        link: false,
    };

    docker
        .remove_container(&id, Some(remove_opts))
        .await
        .map_err(|e| format!("Failed to remove container '{}': {}", id, e))?;

    Ok(())
}

/// Pull a Docker image with progress reporting.
/// Emits progress events via the Tauri app handle (placeholder for Phase 2 event wiring).
#[tauri::command]
pub async fn pull_image(image_name: String) -> Result<(), String> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| format!("Failed to connect to Docker daemon: {}", e))?;

    let options = CreateImageOptions {
        from_image: image_name.as_str(),
        tag: "latest",
        ..Default::default()
    };

    let mut stream = docker.create_image(Some(options), None, None);

    while let Some(item) = stream.next().await {
        match item {
            Ok(info) => {
                // Progress info available but Tauri event wiring is Phase 2.
                // For now we just process the stream to completion.
                let _progress = ImagePullProgress {
                    status: info.status.unwrap_or_default(),
                    progress: info.progress.map(|p| p.to_string()),
                    id: info.id,
                };
            }
            Err(e) => {
                return Err(format!("Failed to pull image '{}': {}", image_name, e));
            }
        }
    }

    Ok(())
}

/// Execute a shell command inside a sandbox container (Tauri command).
/// Returns JSON with exit_code, stdout, stderr.
#[tauri::command]
pub(crate) async fn exec_sandbox_command(
    container_id: String,
    command: String,
) -> Result<String, String> {
    let result = exec_shell_in_container(&container_id, &command).await?;
    serde_json::to_string(&result)
        .map_err(|e| format!("Failed to serialize exec result: {}", e))
}

/// Apply a new network mode to the sandbox configuration.
/// HITL confirmation must be obtained by the frontend before calling this
/// (per 7.1 dangerous operation — operation name: "toggle_network_mode").
/// Returns the applied NetworkMode value.
#[tauri::command]
pub async fn set_sandbox_network_mode(
    new_mode: NetworkMode,
) -> Result<NetworkMode, String> {
    // Mode application: the frontend orchestrated HITL via request_hitl_confirmation/respond_hitl
    // before calling this command. The actual container config update is handled by
    // the frontend when rebuilding the container with the new SandboxConfig.
    Ok(new_mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_config_default() {
        let cfg = SandboxConfig::default();
        assert_eq!(cfg.image, "aegis-sandbox:latest");
        assert_eq!(cfg.memory_mb, 2048);
        assert_eq!(cfg.cpu_shares, 512);
        assert!(cfg.network_mode.is_air_gapped());
    }

    #[test]
    fn test_network_mode_default() {
        assert!(!NetworkMode::default().is_air_gapped());
    }

    #[test]
    fn test_network_mode_bridge_docker_string() {
        assert_eq!(NetworkMode::Bridge.to_docker_string(), None);
    }

    #[test]
    fn test_network_mode_none_docker_string() {
        assert_eq!(NetworkMode::None.to_docker_string(), Some("none".to_string()));
    }

    #[test]
    fn test_network_mode_serde_roundtrip() {
        let bridge_json = r#""bridge""#;
        let none_json = r#""none""#;
        let bridge: NetworkMode = serde_json::from_str(bridge_json).unwrap();
        let none: NetworkMode = serde_json::from_str(none_json).unwrap();
        assert!(!bridge.is_air_gapped());
        assert!(none.is_air_gapped());
        assert_eq!(serde_json::to_string(&bridge).unwrap(), bridge_json);
        assert_eq!(serde_json::to_string(&none).unwrap(), none_json);
    }

    #[test]
    fn test_sandbox_error_to_string() {
        let err = SandboxError::NotFound("abc".into());
        let s: String = err.into();
        assert!(s.contains("abc"));
    }

    #[test]
    fn test_sandbox_error_docker_display() {
        let err = SandboxError::Docker(bollard::errors::Error::DockerResponseServerError {
            status_code: 404,
            message: "container not found".into(),
        });
        let msg = format!("{}", err);
        assert!(msg.contains("container not found"));
    }

    #[test]
    fn test_sandbox_error_not_found() {
        let err = SandboxError::NotFound("test-container".into());
        assert_eq!(format!("{}", err), "Container not found: test-container");
    }

    #[test]
    fn test_sandbox_error_not_running() {
        let err = SandboxError::NotRunning("test".into());
        assert_eq!(format!("{}", err), "Container is not running: test");
    }

    #[test]
    fn test_sandbox_error_timeout() {
        let err = SandboxError::Timeout("operation timed out".into());
        assert!(format!("{}", err).contains("timed out"));
    }

    #[test]
    fn test_sandbox_error_exec_failed() {
        let err = SandboxError::ExecFailed("oom".into());
        assert!(format!("{}", err).contains("oom"));
    }

    #[test]
    fn test_sandbox_error_bridge() {
        let err = SandboxError::Bridge("bridge unavailable".into());
        assert!(format!("{}", err).contains("bridge unavailable"));
    }

    #[test]
    fn test_exec_result_serde_roundtrip() {
        let result = ExecResult {
            exit_code: 0,
            stdout: "hello".into(),
            stderr: String::new(),
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: ExecResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.exit_code, 0);
        assert_eq!(deserialized.stdout, "hello");
    }

    #[test]
    fn test_sandbox_create_result_serde() {
        let res = SandboxCreateResult {
            container_id: "abc123".into(),
            container_name: "test".into(),
        };
        let json = serde_json::to_string(&res).unwrap();
        assert!(json.contains("abc123"));
    }

    #[test]
    fn test_image_pull_progress_serde() {
        let progress = ImagePullProgress {
            status: "Downloading".into(),
            progress: Some("50%".into()),
            id: Some("sha256:abc".into()),
        };
        let json = serde_json::to_string(&progress).unwrap();
        assert!(json.contains("Downloading"));
    }
}
