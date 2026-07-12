#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod agent;
pub mod bridge_client;
pub mod container;
pub mod docker_client;
pub mod env;
pub mod error;
pub mod llm;
pub mod prompts;
pub mod mcp;
pub mod pty;
pub mod sandbox;
pub mod settings;
pub mod hitl;
pub mod tools;

use agent::persistence::{restore_or_default, AppAgentState};
use serde::Serialize;
use std::sync::Mutex;
use tauri::Emitter;
use tauri::Manager;

// --------------------------------------------------------------------------
// Agent Bridge IPC commands (Wave 3.3)
// Proxies to the Docker sandbox bridge on port 8080
// --------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct AgentStatusPayload {
    pub agent_id: String,
    pub status: String,
}

#[tauri::command]
async fn sandbox_health() -> Result<String, String> {
    let client = bridge_client::BridgeClient::new(None).map_err(|e| format!("{}", e))?;
    let health = client.health().await.map_err(|e| format!("{}", e))?;
    serde_json::to_string(&health).map_err(|e| format!("Serialization error: {}", e))
}

#[tauri::command]
async fn create_agent(name: String, config: Option<serde_json::Value>) -> Result<String, String> {
    let client = bridge_client::BridgeClient::new(None).map_err(|e| format!("{}", e))?;
    let result = client.create_agent(&name, config).await.map_err(|e| format!("{}", e))?;
    serde_json::to_string(&result).map_err(|e| format!("Serialization error: {}", e))
}

#[tauri::command]
async fn run_agent(agent_id: String, prompt: String) -> Result<String, String> {
    let client = bridge_client::BridgeClient::new(None).map_err(|e| format!("{}", e))?;
    let result = client.run_agent(&agent_id, &prompt).await.map_err(|e| format!("{}", e))?;
    serde_json::to_string(&result).map_err(|e| format!("Serialization error: {}", e))
}

#[tauri::command]
async fn stop_agent(agent_id: String) -> Result<String, String> {
    let client = bridge_client::BridgeClient::new(None).map_err(|e| format!("{}", e))?;
    let result = client.stop_agent(&agent_id).await.map_err(|e| format!("{}", e))?;
    serde_json::to_string(&result).map_err(|e| format!("Serialization error: {}", e))
}

#[tauri::command]
async fn get_agent_status(agent_id: String) -> Result<String, String> {
    let client = bridge_client::BridgeClient::new(None).map_err(|e| format!("{}", e))?;
    let result = client.get_agent_status(&agent_id).await.map_err(|e| format!("{}", e))?;
    serde_json::to_string(&result).map_err(|e| format!("Serialization error: {}", e))
}

#[tauri::command]
async fn set_agent_status(app_handle: tauri::AppHandle, agent_id: String, status: String) -> Result<(), String> {
    let payload = AgentStatusPayload {
        agent_id,
        status,
    };
    app_handle
        .emit("agent-status", payload)
        .map_err(|e| format!("Failed to emit agent-status event: {}", e))
}

/// Entry point for the Tauri 2 application.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Auto-restore agent state from disk on app startup
            let ctx = restore_or_default(app.handle());
            let app_state = AppAgentState {
                context: Mutex::new(ctx),
            };
            app.manage(app_state);
            app.manage(crate::hitl::HITLManager::new());
            println!("[lib] Agent state initialized via persistence restore");
            Ok(())
        })
        .on_window_event(|window, event| {
            // Auto-save agent state when the window is being closed
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if let Some(state) = window.try_state::<AppAgentState>() {
                    if let Ok(guard) = state.context.lock() {
                        let _ = agent::persistence::save_agent_state(window.app_handle(), &guard);
                        println!("[lib] Agent state saved on window close");
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            // Settings
            settings::load_settings,
            settings::save_settings,
            settings::load_llm_settings,
            settings::save_llm_settings,
            settings::save_api_key,
            settings::get_api_key,
            settings::delete_api_key,
            settings::test_llm_connection,
            // LLM Health
            crate::llm::health_check::get_provider_health,
            // LLM Streaming
            crate::llm::stream_llm_completion,
            // Docker/Sandbox (Wave 2 + Wave 3)
            crate::docker_client::check_docker_daemon,
            crate::container::create_sandbox,
            crate::container::build_sandbox_image,
            crate::container::start_sandbox,
            crate::container::stop_sandbox,
            crate::container::remove_sandbox,
            crate::container::pull_image,
            crate::container::exec_sandbox_command,
            // Agent Bridge (Wave 3.3)
            sandbox_health,
            create_agent,
            run_agent,
            stop_agent,
            get_agent_status,
            set_agent_status,
            // Agent State Persistence (Item 4.4)
            crate::agent::persistence::persist_save_state,
            crate::agent::persistence::persist_load_state,
            crate::agent::persistence::persist_clear_state,
            // Persistent PTY Sessions (Item 4.3)
            crate::pty::create_pty_session,
            crate::pty::pty_execute_command,
            crate::pty::close_pty_session,
            crate::pty::list_pty_sessions,
            // MCP Server Discovery (Item 4.5)
            crate::mcp::discover_mcp_servers,
            crate::mcp::get_mcp_servers,
            crate::mcp::register_mcp_server,
            crate::mcp::remove_mcp_server,
            crate::mcp::clear_mcp_servers,
            // HITL Confirmation (Item 7.1)
            crate::hitl::request_hitl_confirmation,
            crate::hitl::respond_hitl,
            // Air-gapped mode (Item 7.2)
            crate::container::set_sandbox_network_mode,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
