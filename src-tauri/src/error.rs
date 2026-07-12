//! Error taxonomy for the Orchestrator Agent.
//! Ported from mimicking-agent-zero/agent.rs error handling and Agent Zero's helpers/errors.py.
use serde::Serialize;
use thiserror::Error;

/// Typed agent error tree with recovery levels.
/// Related to mimicking-agent-zero/agent.rs Error tree and Agent Zero's helpers/errors.py.
#[derive(Error, Debug, Serialize)]
pub enum AgentError {
    #[error("{0}")]
    Repairable(String),

    #[error("User intervention")]
    Intervention,

    #[error("Critical: {0}")]
    Critical(String),

    #[error("Prompt error: {0}")]
    Prompt(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("JSON parse error: {0}")]
    Json(String),
}

/// Result type for orchestrator agent operations.
pub type AgentResult<T> = std::result::Result<T, AgentError>;