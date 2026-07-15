//! Agent state persistence across app restarts.
//! Saves and restores the AgentContext (history, loop_data, session, config)
//! to/from the app config directory as JSON.
//!
//! Reference: ORCHESTRATOR-ARCHITECTURE.md §4.4 (Agent State Persistence)

use crate::agent::context::AgentContext;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

// --------------------------------------------------------------------------
// Data structures
// --------------------------------------------------------------------------

/// Wrapper for the complete persisted agent state.
/// Top-level struct so we can version/evolve the schema in the future.
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentPersistedState {
    pub context: AgentContext,
}

// --------------------------------------------------------------------------
// Managed Tauri state
// --------------------------------------------------------------------------

/// Managed state held by the Tauri application for agent context access
/// from setup/close hooks and IPC commands.
pub struct AppAgentState {
    pub context: Mutex<AgentContext>,
}

// --------------------------------------------------------------------------
// Path helpers
// --------------------------------------------------------------------------

/// Returns the path to the persisted agent state file in the app config dir.
fn state_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let config_dir = app_handle
        .path()
        .app_config_dir()
        .expect("Failed to get app config dir");
    std::fs::create_dir_all(&config_dir).ok();
    config_dir.join("agent_state.json")
}

// --------------------------------------------------------------------------
// Public API — save / load / clear / restore
// --------------------------------------------------------------------------

/// Save the current AgentContext to disk so it survives app restarts.
pub fn save_agent_state(app_handle: &tauri::AppHandle, context: &AgentContext) -> Result<(), String> {
    let path = state_path(app_handle);
    let state = AgentPersistedState {
        context: context.clone(),
    };
    let content = serde_json::to_string_pretty(&state)
        .map_err(|e| format!("Failed to serialize agent state: {}", e))?;
    // Use a temporary file + atomic rename to prevent corruption on crash
    let tmp_path = path.with_extension("json.tmp");
    std::fs::write(&tmp_path, &content)
        .map_err(|e| format!("Failed to write agent state: {}", e))?;
    std::fs::rename(&tmp_path, &path)
        .map_err(|e| format!("Failed to atomically save agent state: {}", e))?;
    Ok(())
}

/// Load a previously saved AgentContext from disk.
/// Returns Ok(None) if no saved state exists.
pub fn load_agent_state(app_handle: &tauri::AppHandle) -> Result<Option<AgentContext>, String> {
    let path = state_path(app_handle);
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read agent state file: {}", e))?;
    let state: AgentPersistedState = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse agent state file: {}", e))?;
    Ok(Some(state.context))
}

/// Restore saved agent state or return a default fresh context.
/// Used during Tauri's `.setup()` hook for startup auto-restore.
pub fn restore_or_default(app_handle: &tauri::AppHandle) -> AgentContext {
    match load_agent_state(app_handle) {
        Ok(Some(ctx)) => {
            println!("[persistence] Restored agent state from disk");
            ctx
        }
        Ok(None) => {
            println!("[persistence] No saved state found, using defaults");
            AgentContext::default_orchestrator()
        }
        Err(e) => {
            eprintln!("[persistence] Failed to load agent state: {}; using defaults", e);
            AgentContext::default_orchestrator()
        }
    }
}

/// Delete the persisted agent state from disk.
pub fn clear_agent_state(app_handle: &tauri::AppHandle) -> Result<(), String> {
    let path = state_path(app_handle);
    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|e| format!("Failed to delete agent state file: {}", e))?;
    }
    Ok(())
}

// --------------------------------------------------------------------------
// Tauri commands
// --------------------------------------------------------------------------

#[tauri::command]
pub fn persist_save_state(app_handle: tauri::AppHandle, context: AgentContext) -> Result<(), String> {
    save_agent_state(&app_handle, &context)
}

#[tauri::command]
pub fn persist_load_state(app_handle: tauri::AppHandle) -> Result<Option<AgentContext>, String> {
    load_agent_state(&app_handle)
}

#[tauri::command]
pub fn persist_clear_state(app_handle: tauri::AppHandle) -> Result<(), String> {
    clear_agent_state(&app_handle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::state::AgentState;

    #[test]
    fn test_agent_persisted_state_serde_roundtrip() {
        let context = crate::agent::context::AgentContext::default_orchestrator();
        let state = AgentPersistedState { context: context.clone() };
        let json = serde_json::to_string_pretty(&state).unwrap();
        let deserialized: AgentPersistedState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.context.config.name, context.config.name);
    }

    #[test]
    fn test_agent_persisted_state_deserialize_invalid() {
        let result: Result<AgentPersistedState, _> = serde_json::from_str("not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_persisted_state_deserialize_with_skip_field() {
        let json = r#"{"context":{"config":{"name":"test","max_iterations":10,"prompts_path":"/tmp"},"provider":{"name":"deepseek","model":"deepseek-v4-flash","api_key":null,"base_url":null,"max_tokens":4096,"temperature":0.7},"history":{"messages":[]},"state":"Booting","loop_data":{"iteration":0,"last_response":null,"offset":0},"session":{"session_id":"","workspace_path":"","is_active":false,"started_at":"","current_directory":""}}}"#;
        let state: AgentPersistedState = serde_json::from_str(json).unwrap();
        assert!(!state.context.intervention_flag);
        assert_eq!(state.context.state, AgentState::Booting);
    }
}