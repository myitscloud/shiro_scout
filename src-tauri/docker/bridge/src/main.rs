use axum::{
    extract::State,
    http::StatusCode,
    Json, Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};

// --------------------------------------------------------------------------
// State
// --------------------------------------------------------------------------

#[derive(Clone)]
struct AppState {
    agents: Arc<Mutex<HashMap<String, AgentSession>>>,
    tauri_host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentSession {
    id: String,
    status: AgentStatus,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = \"snake_case\")]
pub enum AgentStatus {
    Idle,
    Running,
    Stopped,
    Error,
}

// --------------------------------------------------------------------------
// Request / Response types
// --------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct CreateAgentRequest {
    name: String,
    config: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct CreateAgentResponse {
    agent_id: String,
    status: AgentStatus,
}

#[derive(Debug, Deserialize)]
struct RunAgentRequest {
    agent_id: String,
    prompt: String,
}

#[derive(Debug, Serialize)]
struct RunAgentResponse {
    agent_id: String,
    result: String,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct StopAgentRequest {
    agent_id: String,
}

#[derive(Debug, Serialize)]
struct StopAgentResponse {
    agent_id: String,
    status: AgentStatus,
}

#[derive(Debug, Deserialize)]
struct AgentStatusRequest {
    agent_id: String,
}

#[derive(Debug, Serialize)]
struct AgentStatusResponse {
    agent_id: String,
    status: AgentStatus,
    session: Option<AgentSession>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ToolExecRequest {
    tool: String,
    args: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ToolExecResponse {
    success: bool,
    output: serde_json::Value,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    agents_running: usize,
}

// --------------------------------------------------------------------------
// Handlers
// --------------------------------------------------------------------------

async fn health_handler(State(state): State<AppState>) -> Json<HealthResponse> {
    let agents = state.agents.lock().await;
    let running = agents.values().filter(|a| a.status == AgentStatus::Running).count();
    Json(HealthResponse {
        status: \"ok\".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        agents_running: running,
    })
}

async fn create_agent_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateAgentRequest>,
) -> Result<Json<CreateAgentResponse>, (StatusCode, String)> {
    let agent_id = format!("agent-{}", uuid_v4_simple());
    let now = iso_timestamp();
    let session = AgentSession {
        id: agent_id.clone(),
        status: AgentStatus::Idle,
        created_at: now.clone(),
        updated_at: now,
    };
    
    let mut agents = state.agents.lock().await;
    agents.insert(agent_id.clone(), session);
    info!(agent_id = %agent_id, name = %req.name, \"Agent created\");
    
    Ok(Json(CreateAgentResponse {
        agent_id,
        status: AgentStatus::Idle,
    }))
}

async fn run_agent_handler(
    State(state): State<AppState>,
    Json(req): Json<RunAgentRequest>,
) -> Result<Json<RunAgentResponse>, (StatusCode, String)> {
    let mut agents = state.agents.lock().await;
    let session = agents.get_mut(&req.agent_id).ok_or_else(|| {
        (StatusCode::NOT_FOUND, format!("Agent '{}' not found", req.agent_id))
    })?;
    
    session.status = AgentStatus::Running;
    session.updated_at = iso_timestamp();
    
    // In production, this dispatches to AgentKit child process via ts-node
    // For now, return a stub result
    info!(agent_id = %req.agent_id, prompt_len = %req.prompt.len(), \"Agent running\");
    
    Ok(Json(RunAgentResponse {
        agent_id: req.agent_id,
        result: serde_json::json!({"message": "Agent processing started. Use getAgentStatus() for updates."}).to_string(),
        stream: true,
    }))
}

async fn stop_agent_handler(
    State(state): State<AppState>,
    Json(req): Json<StopAgentRequest>,
) -> Result<Json<StopAgentResponse>, (StatusCode, String)> {
    let mut agents = state.agents.lock().await;
    let session = agents.get_mut(&req.agent_id).ok_or_else(|| {
        (StatusCode::NOT_FOUND, format!("Agent '{}' not found", req.agent_id))
    })?;
    
    session.status = AgentStatus::Stopped;
    session.updated_at = iso_timestamp();
    info!(agent_id = %req.agent_id, \"Agent stopped\");
    
    Ok(Json(StopAgentResponse {
        agent_id: req.agent_id,
        status: AgentStatus::Stopped,
    }))
}

async fn get_agent_status_handler(
    State(state): State<AppState>,
    Json(req): Json<AgentStatusRequest>,
) -> Result<Json<AgentStatusResponse>, (StatusCode, String)> {
    let agents = state.agents.lock().await;
    let session = agents.get(&req.agent_id).ok_or_else(|| {
        (StatusCode::NOT_FOUND, format!("Agent '{}' not found", req.agent_id))
    })?;
    
    Ok(Json(AgentStatusResponse {
        agent_id: req.agent_id,
        status: session.status.clone(),
        session: Some(session.clone()),
    }))
}

async fn tool_exec_handler(
    State(state): State<AppState>,
    Json(req): Json<ToolExecRequest>,
) -> Result<Json<ToolExecResponse>, (StatusCode, String)> {
    info!(tool = %req.tool, \"Tool execution requested\");
    
    // Forward tool execution to Tauri host via HTTP
    let client = reqwest::Client::new();
    let tauri_url = format!("{}/api/execute-tool", state.tauri_host);
    
    match client.post(&tauri_url)
        .json(&req)
        .send()
        .await
    {
        Ok(resp) => {
            match resp.json::<ToolExecResponse>().await {
                Ok(result) => Ok(Json(result)),
                Err(e) => Err((StatusCode::BAD_GATEWAY, format!("Failed to parse tool response: {}", e))),
            }
        }
        Err(e) => {
            error!(error = %e, \"Failed to forward tool exec to Tauri host\");
            Err((StatusCode::BAD_GATEWAY, format!("Tauri host unreachable: {}", e)))
        }
    }
}

// --------------------------------------------------------------------------
// Helpers
// --------------------------------------------------------------------------

fn uuid_v4_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("{:x}-{:x}", ts.as_secs(), ts.subsec_nanos())
}

fn iso_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = d.as_secs();
    let nanos = d.subsec_nanos();
    // Format as ISO 8601 approximation
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let mins = (time_secs % 3600) / 60;
    let secs_remain = time_secs % 60;
    format!("2026-07-{:02}T{:02}:{:02}:{:02}Z", 10 + days, hours, mins, secs_remain)
}

// --------------------------------------------------------------------------
// Main
// --------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| \"agent_bridge=info,tower_http=info\".into())
        )
        .init();

    let tauri_host = std::env::var("TAURI_HOST")
        .unwrap_or_else(|_| \"http://host.docker.internal:8081\".to_string());

    let state = AppState {
        agents: Arc::new(Mutex::new(HashMap::new())),
        tauri_host,
    };

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/api/create-agent", post(create_agent_handler))
        .route("/api/run-agent", post(run_agent_handler))
        .route("/api/stop-agent", post(stop_agent_handler))
        .route("/api/get-agent-status", post(get_agent_status_handler))
        .route("/api/execute-tool", post(tool_exec_handler))
        .with_state(state);

    let addr = \"0.0.0.0:8080\";
    info!(address = %addr, \"Agent bridge starting\");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}