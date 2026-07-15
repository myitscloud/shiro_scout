//! Agent context - the shared runtime context for the orchestrator agent loop.
//! Ported from mimicking-agent-zero/agent.rs AgentContext.
//!
//! Holds:
//! - Agent configuration (name, prompts, hooks)
//! - LLM provider instance
//! - Tool registry
//! - Session state (shell, sandbox, workspace)
//! - Event bus for UI communication

use serde::{Deserialize, Serialize};

use crate::agent::history::History;
use crate::agent::state::AgentState;
use crate::agent::loop_data::LoopData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub max_tokens: u32,
    pub temperature: f64,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            name: "deepseek".into(),
            model: "deepseek-v4-flash".into(),
            api_key: None,
            base_url: None,
            max_tokens: 4096,
            temperature: 0.7,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: String,
    pub workspace_path: String,
    pub is_active: bool,
    pub started_at: String,
    pub current_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub max_iterations: u64,
    pub prompts_path: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: "orchestrator".into(),
            max_iterations: 50,
            prompts_path: "usr/agents/orchestrator/prompts".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub config: AgentConfig,
    pub provider: ProviderConfig,
    pub history: History,
    pub state: AgentState,
    pub loop_data: LoopData,
    pub session: SessionState,
    #[serde(skip)]
    pub intervention_flag: bool,
}

impl AgentContext {
    pub fn new(config: AgentConfig, provider: ProviderConfig) -> Self {
        Self {
            config,
            provider,
            history: History::new(),
            state: AgentState::Booting,
            loop_data: LoopData::default(),
            session: SessionState::default(),
            intervention_flag: false,
        }
    }

    pub fn default_orchestrator() -> Self {
        Self::new(AgentConfig::default(), ProviderConfig::default())
    }

    pub fn transition_to(&mut self, new_state: AgentState) -> AgentState {
        std::mem::replace(&mut self.state, new_state)
    }

    pub fn set_intervention(&mut self) {
        self.intervention_flag = true;
    }

    pub fn clear_intervention(&mut self) {
        self.intervention_flag = false;
    }

    pub fn reset_loop(&mut self) {
        self.loop_data.reset_iteration();
        self.loop_data.iteration += 1;
        self.clear_intervention();
    }

    pub fn reset_session(&mut self) {
        self.history.clear();
        self.state = AgentState::Ready;
        self.loop_data = LoopData::default();
        self.clear_intervention();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_context_default_orchestrator() {
        let ctx = AgentContext::default_orchestrator();
        assert_eq!(ctx.config.name, "orchestrator");
        assert_eq!(ctx.state, AgentState::Booting);
        assert!(!ctx.intervention_flag);
    }

    #[test]
    fn test_agent_context_transition_to() {
        let mut ctx = AgentContext::default_orchestrator();
        let prev = ctx.transition_to(AgentState::Ready);
        assert_eq!(prev, AgentState::Booting);
        assert_eq!(ctx.state, AgentState::Ready);
    }

    #[test]
    fn test_agent_context_intervention_flag() {
        let mut ctx = AgentContext::default_orchestrator();
        ctx.set_intervention();
        assert!(ctx.intervention_flag);
        ctx.clear_intervention();
        assert!(!ctx.intervention_flag);
    }

    #[test]
    fn test_agent_context_reset_loop() {
        let mut ctx = AgentContext::default_orchestrator();
        ctx.set_intervention();
        ctx.state = AgentState::Thinking;
        ctx.reset_loop();
        assert!(!ctx.intervention_flag);
        assert_eq!(ctx.loop_data.iteration, 1);
    }

    #[test]
    fn test_agent_context_reset_session() {
        let mut ctx = AgentContext::default_orchestrator();
        ctx.state = AgentState::Done;
        ctx.history.add_user("test");
        ctx.reset_session();
        assert!(ctx.history.messages.is_empty());
        assert_eq!(ctx.state, AgentState::Ready);
        assert_eq!(ctx.loop_data.iteration, 0);
    }
}