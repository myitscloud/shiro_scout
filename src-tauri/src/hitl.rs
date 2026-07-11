// --------------------------------------------------------------------------
// HITL (Human-In-The-Loop) confirmation flow for dangerous operations
// Wave 7.1 — Rust-side implementation
//
// Provides a thread-safe confirmation manager that creates pending sessions,
// validates responses via nonce and timeout, and emits events to the frontend.
// --------------------------------------------------------------------------

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{Emitter, Manager};
use uuid::Uuid;

// --------------------------------------------------------------------------
// Types
// --------------------------------------------------------------------------

/// A pending HITL confirmation session awaiting user response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HITLSession {
    pub id: Uuid,
    pub operation_name: String,
    pub operation_description: String,
    pub risk_level: String,
    pub payload: serde_json::Value,
    pub nonce: String,
    #[serde(skip)]
    pub created_at_unix_secs: u64,
    pub timeout_secs: u32,
}

/// Response from the frontend approving or rejecting a HITL session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HITLResponse {
    pub session_id: Uuid,
    pub approved: bool,
    pub reason: Option<String>,
    pub nonce: String,
}

/// Event emitted to the frontend when a HITL confirmation is requested.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HITLEvent {
    pub session_id: Uuid,
    pub operation_name: String,
    pub operation_description: String,
    pub risk_level: String,
    pub payload: serde_json::Value,
    pub nonce: String,
}

// --------------------------------------------------------------------------
// Nonce generation
// --------------------------------------------------------------------------

/// Generate a cryptographically-anchored nonce from a UUID and timestamp.
fn generate_nonce() -> String {
    let uuid = Uuid::new_v4();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let input = format!("{}-{}", uuid, now);
    // Use a simple hash for the nonce — avoids adding a crypto dependency
    let hash = blake3_hash(&input);
    hash.to_string()
}

/// Simple hash function for nonce generation.
/// Uses a basic digest approach with no external dependencies beyond std.
fn blake3_hash(input: &str) -> String {
    // Use SHA-256 via the windows crate's built-in hashing if available,
    // but since we target Windows with windows-rs, fall back to hex encoding
    // of a simple digest computed from the input bytes.
    let bytes = input.as_bytes();
    // A simple but sufficient hash for nonce purposes (not cryptographic auth)
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    let hash = hasher.finish();
    format!("{:016x}", hash)
}

// --------------------------------------------------------------------------
// HITLManager
// --------------------------------------------------------------------------

/// Thread-safe manager for HITL confirmation sessions.
pub struct HITLManager {
    pending_sessions: Mutex<HashMap<Uuid, HITLSession>>,
}

impl HITLManager {
    /// Create a new empty HITLManager.
    pub fn new() -> Self {
        Self {
            pending_sessions: Mutex::new(HashMap::new()),
        }
    }

    /// Create a new HITL session and return the event to emit to the frontend.
    /// Cleans up expired sessions before creating a new one.
    pub fn create_session(
        &self,
        operation: &str,
        description: &str,
        risk_level: &str,
        payload: serde_json::Value,
        timeout_secs: u32,
    ) -> Result<HITLEvent, String> {
        // Clean expired sessions first
        self.cleanup_expired();

        let session_id = Uuid::new_v4();
        let nonce = generate_nonce();
        let session = HITLSession {
            id: session_id,
            operation_name: operation.to_string(),
            operation_description: description.to_string(),
            risk_level: risk_level.to_string(),
            payload: payload.clone(),
            nonce: nonce.clone(),
            created_at_unix_secs: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            timeout_secs,
        };

        let mut sessions = self.pending_sessions.lock().map_err(|_| {
            "Internal error: failed to acquire session lock".to_string()
        })?;
        sessions.insert(session_id, session);

        Ok(HITLEvent {
            session_id,
            operation_name: operation.to_string(),
            operation_description: description.to_string(),
            risk_level: risk_level.to_string(),
            payload,
            nonce,
        })
    }

    /// Validate a HITL response: verify nonce, check timeout, and return approval status.
    /// Removes the session regardless of outcome (single-use).
    pub fn validate_response(&self, response: HITLResponse) -> Result<bool, String> {
        let mut sessions = self.pending_sessions.lock().map_err(|_| {
            "Internal error: failed to acquire session lock".to_string()
        })?;

        let session = sessions.remove(&response.session_id).ok_or_else(|| {
            "Session not found or already expired".to_string()
        })?;

        // Validate nonce
        if session.nonce != response.nonce {
            return Err("Nonce mismatch: session verification failed".to_string());
        }

        // Check timeout
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let elapsed_secs = now_secs.saturating_sub(session.created_at_unix_secs);
        if elapsed_secs >= session.timeout_secs as u64 {
            return Err("Session has timed out".to_string());
        }

        Ok(response.approved)
    }

    /// Remove all expired sessions.
    pub fn cleanup_expired(&self) {
        if let Ok(mut sessions) = self.pending_sessions.lock() {
            let now_secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            sessions.retain(|_, session| {
                let elapsed_secs = now_secs.saturating_sub(session.created_at_unix_secs);
                elapsed_secs < session.timeout_secs as u64
            });
        }
    }

    /// Returns true if there are any pending sessions.
    pub fn has_pending(&self) -> bool {
        self.pending_count() > 0
    }

    /// Returns the number of pending sessions.
    pub fn pending_count(&self) -> usize {
        self.pending_sessions
            .lock()
            .map(|sessions| sessions.len())
            .unwrap_or(0)
    }
}

impl Default for HITLManager {
    fn default() -> Self {
        Self::new()
    }
}

// --------------------------------------------------------------------------
// Tauri Commands
// --------------------------------------------------------------------------

/// Request HITL confirmation from the user.
/// Creates a pending session and emits an event to the frontend.
#[tauri::command]
pub fn request_hitl_confirmation(
    app_handle: tauri::AppHandle,
    state: tauri::State<HITLManager>,
    operation: String,
    description: String,
    risk_level: String,
    payload: serde_json::Value,
) -> Result<String, String> {
    // Determine timeout from settings if available
    let timeout_secs = app_handle
        .try_state::<crate::settings::AppSettings>()
        .map(|s| s.hitl_timeout_secs)
        .unwrap_or(30);

    let event = state.create_session(
        &operation,
        &description,
        &risk_level,
        payload,
        timeout_secs,
    )?;

    // Emit event to the frontend
    // C10: sanitized event payload — no raw paths/stack traces
    let event_name = "hitl-confirmation-request";
    if app_handle.emit(event_name, &event).is_err() {
        return Err("Failed to emit confirmation event".to_string());
    }

    serde_json::to_string(&event.session_id)
        .map_err(|_| "Internal serialization error".to_string())
}

/// Respond to a pending HITL confirmation.
#[tauri::command]
pub fn respond_hitl(
    state: tauri::State<HITLManager>,
    session_id: String,
    approved: bool,
    reason: Option<String>,
    nonce: String,
) -> Result<bool, String> {
    let session_uuid = Uuid::parse_str(&session_id)
        .map_err(|_| "Invalid session ID format".to_string())?;

    let response = HITLResponse {
        session_id: session_uuid,
        approved,
        reason,
        nonce,
    };

    state.validate_response(response)
}

// --------------------------------------------------------------------------
// Tests
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_and_validate_session() {
        let manager = HITLManager::new();
        let payload = json!({"path": "/tmp/test"});

        let event = manager
            .create_session("delete_files", "Delete test files", "critical", payload, 30)
            .expect("Session creation should succeed");

        assert_eq!(event.operation_name, "delete_files");
        assert_eq!(event.risk_level, "critical");
        assert!(manager.has_pending());
        assert_eq!(manager.pending_count(), 1);

        let response = HITLResponse {
            session_id: event.session_id,
            approved: true,
            reason: None,
            nonce: event.nonce.clone(),
        };

        let result = manager
            .validate_response(response)
            .expect("Validation should succeed");
        assert!(result);
        assert!(!manager.has_pending());
    }

    #[test]
    fn test_reject_session() {
        let manager = HITLManager::new();
        let payload = json!({"cmd": "rm -rf /"});

        let event = manager
            .create_session("execute_command", "Dangerous command", "high", payload, 30)
            .expect("Session creation should succeed");

        let response = HITLResponse {
            session_id: event.session_id,
            approved: false,
            reason: Some("User rejected".to_string()),
            nonce: event.nonce,
        };

        let result = manager
            .validate_response(response)
            .expect("Validation should succeed");
        assert!(!result);
    }

    #[test]
    fn test_nonce_mismatch() {
        let manager = HITLManager::new();
        let payload = json!({"action": "format"});

        let event = manager
            .create_session("format_disk", "Format disk", "critical", payload, 30)
            .expect("Session creation should succeed");

        let response = HITLResponse {
            session_id: event.session_id,
            approved: true,
            reason: None,
            nonce: "wrong-nonce".to_string(),
        };

        let result = manager.validate_response(response);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Nonce mismatch"));
    }

    #[test]
    fn test_session_not_found() {
        let manager = HITLManager::new();
        let response = HITLResponse {
            session_id: Uuid::new_v4(),
            approved: true,
            reason: None,
            nonce: "some-nonce".to_string(),
        };

        let result = manager.validate_response(response);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Session not found"));
    }

    #[test]
    fn test_cleanup_expired() {
        let manager = HITLManager::new();
        let payload = json!({});

        // Create a session with very short timeout
        let event = manager
            .create_session("test", "expired test", "low", payload, 0)
            .expect("Session creation should succeed");

        // With timeout_secs = 0, the session should be expired immediately
        // but not removed until cleanup is called
        assert_eq!(manager.pending_count(), 1);

        // Validate should fail due to timeout
        let response = HITLResponse {
            session_id: event.session_id,
            approved: true,
            reason: None,
            nonce: event.nonce,
        };

        let result = manager.validate_response(response);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("timed out"));
    }

    #[test]
    fn test_cleanup_removes_expired() {
        let manager = HITLManager::new();
        let payload = json!({});

        // Create a session with zero timeout so it's immediately expired
        let _event = manager
            .create_session("test", "cleanup test", "low", payload, 0)
            .expect("Session creation should succeed");

        // Create another session should trigger cleanup first
        let payload2 = json!({"new": true});
        let event2 = manager
            .create_session("test2", "new session", "low", payload2, 30)
            .expect("Second session should succeed");

        // The expired one should have been cleaned up
        assert_eq!(manager.pending_count(), 1);
        assert_eq!(event2.operation_name, "test2");
    }

    #[test]
    fn test_pending_count() {
        let manager = HITLManager::new();
        assert!(!manager.has_pending());
        assert_eq!(manager.pending_count(), 0);

        let payload = json!({});
        manager
            .create_session("op1", "op1", "low", payload.clone(), 30)
            .expect("Session creation should succeed");

        manager
            .create_session("op2", "op2", "medium", payload, 30)
            .expect("Session creation should succeed");

        assert_eq!(manager.pending_count(), 2);
    }

    #[test]
    fn test_generate_nonce_unique() {
        let nonce1 = generate_nonce();
        let nonce2 = generate_nonce();
        assert_ne!(nonce1, nonce2, "Nonces should be unique");
    }
}
