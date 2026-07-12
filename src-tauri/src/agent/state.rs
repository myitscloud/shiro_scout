//! Agent state machine for the Orchestrator.
//! Tracks the lifecycle of the agent from boot through conversation.
//! Each state has a clear purpose and transition rules.
//!
//! Reference: ORCHESTRATOR-ARCHITECTURE.md §3 (Agent State Machine)
//!
//! Wave 0 → Wave 4.1: Flow is idle → thinking (with streaming) → tool (bridge invoke) → done.
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    /// Agent is starting up (transient)
    Booting,
    /// Checking dependencies and environment (transient)
    CheckingDependencies,
    /// First-run setup (transient)
    Onboarding,
    /// Ready to accept input (stable, accepting input)
    Ready,
    /// LLM is processing — uses streaming infrastructure to emit llm-token events (transient)
    Thinking,
    /// Tool/bridge execution in progress (transient)
    Executing,
    /// Streaming response being emitted to frontend (transient)
    Streaming,
    /// Recovering from error (transient)
    Recovering,
    /// No active processing, awaiting next input (stable, accepting input)
    Idle,
    /// Final response is ready — agent has a completed result (terminal for a message cycle)
    Done,
    /// Agent is waiting for HITL (Human-In-The-Loop) confirmation (stable, waiting)
    PendingConfirmation,
    /// Agent is shutting down
    ShuttingDown,
}

impl AgentState {
    /// Returns true if the agent is in a running state that prohibits new input.
    pub fn is_busy(&self) -> bool {
        matches!(
            self,
            AgentState::Thinking
                | AgentState::Executing
                | AgentState::Streaming
                | AgentState::Recovering
                | AgentState::PendingConfirmation
        )
    }

    /// Returns true if the agent can accept a new user message.
    pub fn can_accept_input(&self) -> bool {
        matches!(self, AgentState::Ready | AgentState::Idle | AgentState::Done | AgentState::PendingConfirmation)
    }

    /// Human-readable label for the current state.
    pub fn label(&self) -> &str {
        match self {
            AgentState::Booting => "Starting up",
            AgentState::CheckingDependencies => "Checking dependencies",
            AgentState::Onboarding => "First-run setup",
            AgentState::Ready => "Ready",
            AgentState::Thinking => "Thinking",
            AgentState::Executing => "Executing",
            AgentState::Streaming => "Responding",
            AgentState::Recovering => "Recovering from error",
            AgentState::Idle => "Idle",
            AgentState::Done => "Done",
            AgentState::PendingConfirmation => "Awaiting approval",
            AgentState::ShuttingDown => "Shutting down",
        }
    }
}

impl fmt::Display for AgentState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_can_accept_input() {
        assert!(AgentState::Ready.can_accept_input());
        assert!(AgentState::Idle.can_accept_input());
        assert!(AgentState::Done.can_accept_input());
        assert!(!AgentState::Booting.can_accept_input());
        assert!(!AgentState::CheckingDependencies.can_accept_input());
        assert!(!AgentState::Thinking.can_accept_input());
        assert!(!AgentState::Executing.can_accept_input());
        assert!(!AgentState::Streaming.can_accept_input());
        assert!(!AgentState::Recovering.can_accept_input());
        assert!(!AgentState::ShuttingDown.can_accept_input());
    }

    #[test]
    fn test_state_is_busy() {
        assert!(AgentState::Thinking.is_busy());
        assert!(AgentState::Executing.is_busy());
        assert!(AgentState::Streaming.is_busy());
        assert!(AgentState::Recovering.is_busy());
        assert!(!AgentState::Booting.is_busy());
        assert!(!AgentState::CheckingDependencies.is_busy());
        assert!(!AgentState::Onboarding.is_busy());
        assert!(!AgentState::Ready.is_busy());
        assert!(!AgentState::Idle.is_busy());
        assert!(!AgentState::Done.is_busy());
        assert!(!AgentState::ShuttingDown.is_busy());
    }

    #[test]
    fn test_state_label() {
        assert_eq!(AgentState::Booting.label(), "Starting up");
        assert_eq!(AgentState::CheckingDependencies.label(), "Checking dependencies");
        assert_eq!(AgentState::Onboarding.label(), "First-run setup");
        assert_eq!(AgentState::Ready.label(), "Ready");
        assert_eq!(AgentState::Thinking.label(), "Thinking");
        assert_eq!(AgentState::Executing.label(), "Executing");
        assert_eq!(AgentState::Streaming.label(), "Responding");
        assert_eq!(AgentState::Recovering.label(), "Recovering from error");
        assert_eq!(AgentState::Idle.label(), "Idle");
        assert_eq!(AgentState::Done.label(), "Done");
        assert_eq!(AgentState::ShuttingDown.label(), "Shutting down");
    }

    #[test]
    fn test_state_display() {
        assert_eq!(AgentState::Ready.to_string(), "Ready");
        assert_eq!(AgentState::Idle.to_string(), "Idle");
    }

    #[test]
    fn test_state_equality() {
        assert_eq!(AgentState::Ready, AgentState::Ready);
        assert_ne!(AgentState::Ready, AgentState::Idle);
    }

    #[test]
    fn test_state_transition_tracking() {
        let states = vec![
            AgentState::Booting,
            AgentState::CheckingDependencies,
            AgentState::Onboarding,
            AgentState::Ready,
            AgentState::Thinking,
            AgentState::Executing,
            AgentState::Streaming,
            AgentState::Recovering,
            AgentState::Idle,
            AgentState::Done,
            AgentState::PendingConfirmation,
            AgentState::ShuttingDown,
        ];
        assert_eq!(states.len(), 12);
        for i in 0..states.len() {
            for j in (i + 1)..states.len() {
                assert_ne!(states[i], states[j]);
            }
        }
    }
}