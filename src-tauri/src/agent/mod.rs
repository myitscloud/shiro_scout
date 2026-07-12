//! Orchestrator Agent module.
//! Contains the core agent struct, state machine, context, and message loop.
//!
//! Reference architecture: docs/Arch_Design/ORCHESTRATOR-ARCHITECTURE.md
//! Source port: docs/mimicking-agent-zero/agent.rs
// Module inception is by design - agent.rs contains the core Agent struct
#[allow(clippy::module_inception)]
pub mod agent;
pub mod context;
pub mod state;
pub mod loop_data;
pub mod history;
pub mod persistence;

pub use agent::Agent;
pub use context::AgentContext;
pub use state::AgentState;
pub use loop_data::LoopData;
pub use history::History;
