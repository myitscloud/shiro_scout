#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod agent;
pub mod bridge_client;
pub mod container;
pub mod docker_client;
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
use std::sync::OnceLock;
use tauri::Emitter;
use tauri::Manager;

use crate::agent::agent::Agent;
use crate::bridge_client::ToolExecBridge;

// --------------------------------------------------------------------------
// Global agent instance for the inline orchestrator
// Preserves conversation history across chat turns.
// --------------------------------------------------------------------------

static AGENT_INSTANCE: OnceLock<Mutex<Option<(Agent, String)>>> = OnceLock::new();

fn get_agent_mutex() -> &'static Mutex<Option<(Agent, String)>> {
    AGENT_INSTANCE.get_or_init(|| Mutex::new(None))
}

/// Process a user message through the ShiroScout agent state machine.
/// The agent calls the LLM, parses JSON tool calls, executes them in
/// the sandbox via ToolExecBridge, and returns the final response.
/// Conversation history persists across calls.
#[tauri::command]
async fn process_agent_message(
    app_handle: tauri::AppHandle,
    message: String,
) -> Result<String, String> {
    // Take the agent out of storage
    let mut agent_tuple: Option<(Agent, String)> = {
        let lock = get_agent_mutex();
        let mut guard = lock.lock().map_err(|e| format!("Lock error: {}", e))?;
        guard.take()
    };

    let (mut agent, cid) = if let Some((ag, id)) = agent_tuple.take() {
        // Reuse existing agent
        (ag, id)
    } else {
        // Create fresh agent
        let mut a = Agent::new_orchestrator().await;
        a.system_prompt = Some(
            "You are ShiroScout, an autonomous AI engineering agent with tool execution capability.\n\
             You can execute shell commands in the sandbox container by returning a JSON tool call:\n\
             {\"tool_name\": \"terminal\", \"tool_args\": {\"command\": \"<shell command>\"}}\n\n\
             Rules:\n- When the user asks to read/list/show files or run commands, call the terminal tool.\n- When asked to create/edit/write files, call the terminal tool with appropriate shell commands.\n- After a tool call returns output, use it to formulate your response in natural language.\n- For conversation or questions that don't need tools, respond directly in plain text.\n- Always respond in the same language as the user's message.".to_string()
        );
        a.app_handle = Some(app_handle.clone());
        let cid = "aegis-sandbox".to_string();
        let bridge = ToolExecBridge::new(cid.clone());
        a.tools = a.tools.with_bridge(bridge);
        (a, cid)
    };

    // Process the message through the agent state machine
    let result = agent.process_message(&message).await;

    // Store the agent back for the next call
    {
        let lock = get_agent_mutex();
        let mut guard = lock.lock().map_err(|e| format!("Lock error: {}", e))?;
        *guard = Some((agent, cid));
    }

    match result {
        Ok(response) => Ok(response),
        Err(e) => Err(format!("{}", e)),
    }
}

/// --------------------------------------------------------------------------
/// Agent Bridge IPC commands (Wave 3.3)
/// Proxies to the Docker sandbox bridge on port 8080
/// --------------------------------------------------------------------------
/// --------------------------------------------------------------------------

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
        .plugin(tauri_plugin_shell::init()).plugin(tauri_plugin_dialog::init()).plugin(tauri_plugin_updater::Builder::new().build())
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
            // Docker/Sandbox (Wave 2 + Wave 3)
            crate::docker_client::check_docker_daemon,
            crate::container::create_sandbox,
            crate::container::build_sandbox_image,
            crate::container::start_sandbox,
            crate::container::stop_sandbox,
            crate::container::remove_sandbox,
            crate::container::pull_image,
            crate::container::exec_sandbox_command,
            // Inline Agent (ShiroScout)
            process_agent_message,
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

