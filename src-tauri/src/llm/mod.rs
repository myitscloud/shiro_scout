//! LLM integration using Rig crate (v0.39.0).
//! Supports: DeepSeek, OpenAI, Groq, Together, Ollama, LiteLLM.
//! Default: DeepSeek-v4-flash (user preference) for all 3 roles.
//!
//! All LLM calls (streaming and non-streaming) are routed through
//! rig::providers — the agent.rs state machine handles token emission
//! via llm-token events. This module provides shared types and the
//! default base URL helper.

pub mod credential_manager;
pub mod health_check;
pub mod keychain;

use serde::{Deserialize, Serialize};

/// Per-role LLM configuration matching Agent Zero's 3-role pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMProviderConfig {
    pub provider: String,
    pub model: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub max_tokens: u32,
    pub temperature: f64,
}

impl Default for LLMProviderConfig {
    fn default() -> Self {
        Self {
            provider: "deepseek".into(),
            model: "deepseek-v4-flash".into(),
            base_url: Some("https://api.deepseek.com/v1".into()),
            api_key: None,
            max_tokens: 8192,
            temperature: 0.7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmConfig {
    pub chat: LLMProviderConfig,
    pub utility: LLMProviderConfig,
    pub embedding: LLMProviderConfig,
}

impl LlmConfig {
    pub fn deepseek_default() -> Self {
        Self {
            chat: LLMProviderConfig { provider: "deepseek".into(), model: "deepseek-v4-flash".into(), base_url: Some("https://api.deepseek.com/v1".into()), api_key: None, max_tokens: 8192, temperature: 0.7 },
            utility: LLMProviderConfig { provider: "deepseek".into(), model: "deepseek-v4-flash".into(), base_url: Some("https://api.deepseek.com/v1".into()), api_key: None, max_tokens: 4096, temperature: 0.3 },
            embedding: LLMProviderConfig { provider: "deepseek".into(), model: "deepseek-v4-flash".into(), base_url: Some("https://api.deepseek.com/v1".into()), api_key: None, max_tokens: 1024, temperature: 0.0 },
        }
    }
}

// --------------------------------------------------------------------------
// IPC types for streaming LLM events
// --------------------------------------------------------------------------

/// Mirror of the TypeScript LlmTokenPayload consumed by useStreamingLlm.
/// Sent as the Tauri `llm-token` event payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmTokenPayload {
    pub role: String,
    pub token: String,
    pub done: bool,
}

/// Input parameters for stream_llm_completion Tauri command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamCompletionInput {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub system_prompt: Option<String>,
    pub messages: Vec<StreamMessage>,
    pub max_tokens: u32,
    pub temperature: f64,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMessage {
    pub role: String,
    pub content: String,
    pub name: Option<String>,
    pub tool_call_id: Option<String>,
}

// --------------------------------------------------------------------------
// Default base URL helper (shared with health_check)
// --------------------------------------------------------------------------

pub(crate) fn default_base_url_for(provider: &str) -> String {
    match provider {
        "deepseek" => "https://api.deepseek.com".to_string(),
        "openai" => "https://api.openai.com/v1".to_string(),
        "groq" => "https://api.groq.com/openai/v1".to_string(),
        "together" => "https://api.together.xyz/v1".to_string(),
        "ollama" => "http://localhost:11434/v1".to_string(),
        "litellm" => "http://localhost:4000/v1".to_string(),
        _ => format!("https://api.{}.com/v1", provider),
    }
}

