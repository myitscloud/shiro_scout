//! Core types ported from Agent Zero's helpers/errors.py, helpers/extension.py,
//! tools/*, and helpers/history.py.
//!
//! The single most important idea in this file is `AgentError`. Agent Zero's
//! "self-healing" is nothing more mystical than this taxonomy:
//!
//!   Repairable   -> error text is injected back into chat history via
//!                   prompts/fw.error.md and the loop CONTINUES (model fixes itself)
//!   Intervention -> user interrupted; inject fw.intervention.md, continue
//!   Handled      -> already surfaced to user; kill the loop quietly
//!   Critical     -> inject fw.msg_critical_error.md-style log, stop the monologue

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Errors (helpers/errors.py: RepairableException / InterventionException / HandledException)
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    /// Feed back to the model and keep looping. This is the repair path.
    #[error("{0}")]
    Repairable(String),

    /// User sent a message mid-run (A0's "intervention").
    #[error("user intervention")]
    Intervention,

    /// Already logged/shown; terminate without re-reporting.
    #[error("handled: {0}")]
    Handled(String),

    /// Unrecoverable (config missing, auth failure, panic-adjacent). Stops the run.
    #[error("critical: {0}")]
    Critical(String),
}

pub type AgentResult<T> = Result<T, AgentError>;

// ---------------------------------------------------------------------------
// Chat history (subset of helpers/history.py)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Default)]
pub struct History {
    pub messages: Vec<ChatMessage>,
}

impl History {
    pub fn add_user(&mut self, content: impl Into<String>) {
        self.messages.push(ChatMessage { role: Role::User, content: content.into() });
    }

    pub fn add_ai_response(&mut self, content: impl Into<String>) {
        self.messages.push(ChatMessage { role: Role::Assistant, content: content.into() });
    }

    /// A0's hist_add_warning: framework messages (fw.*) enter history as user-role
    /// messages so the model sees them as environment feedback, not its own words.
    pub fn add_warning(&mut self, content: impl Into<String>) {
        self.add_user(content);
    }

    /// A0's hist_add_tool_result: rendered via prompts/fw.tool_result.md.
    pub fn add_tool_result(&mut self, rendered: impl Into<String>) {
        self.add_user(rendered);
    }

    pub fn last_assistant(&self) -> Option<&str> {
        self.messages.iter().rev()
            .find(|m| m.role == Role::Assistant)
            .map(|m| m.content.as_str())
    }

    // TODO(port helpers/history.py): topic/bulk summarization + compression using
    // prompts/fw.topic_summary.{sys,msg}.md and fw.bulk_summary.{sys,msg}.md.
    // That compression is a large part of A0's long-run "self-awareness".
    pub fn compress_if_needed(&mut self, _token_budget: usize) { /* todo!() */ }
}

// ---------------------------------------------------------------------------
// Extension hooks (helpers/extension.py + extensions/python/<hook_dir>/_NN_name.py)
// ---------------------------------------------------------------------------

/// One variant per directory under extensions-python/ that you care about.
/// A0 discovers these from folder names; in Rust we enumerate them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookPoint {
    AgentInit,
    MonologueStart,
    MessageLoopStart,
    MessageLoopPromptsBefore,
    SystemPrompt,             // extensions-python/system_prompt/_10.._14 compose the persona
    MessageLoopPromptsAfter,  // datetime, agent info, skill recall, memory recall live here
    BeforeMainLlmCall,
    ToolExecuteBefore,
    ToolExecuteAfter,
    MessageLoopEnd,
    MonologueEnd,             // A0's memory plugin memorizes fragments/solutions here
}

/// Mutable state threaded through one loop iteration (A0's LoopData).
#[derive(Default)]
pub struct LoopData {
    pub iteration: u64,
    pub user_message: Option<String>,
    pub system_prompt_parts: Vec<String>, // SystemPrompt hooks push into this
    pub extras: HashMap<String, String>,  // PromptsAfter hooks append transient context
    pub last_response: Option<String>,    // for fw.msg_repeat.md detection
}

#[async_trait]
pub trait Extension: Send + Sync {
    /// Mirrors the _10_, _50_, _90_ filename prefixes: lower runs first.
    fn order(&self) -> u32 { 50 }
    async fn execute(&self, loop_data: &mut LoopData, history: &mut History) -> AgentResult<()>;
}

#[derive(Default)]
pub struct HookRegistry {
    hooks: HashMap<HookPoint, Vec<Arc<dyn Extension>>>,
}

impl HookRegistry {
    pub fn register(&mut self, point: HookPoint, ext: Arc<dyn Extension>) {
        let v = self.hooks.entry(point).or_default();
        v.push(ext);
        v.sort_by_key(|e| e.order());
    }

    pub async fn run(
        &self,
        point: HookPoint,
        loop_data: &mut LoopData,
        history: &mut History,
    ) -> AgentResult<()> {
        if let Some(list) = self.hooks.get(&point) {
            for ext in list {
                ext.execute(loop_data, history).await?;
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tools (tools/*.py + prompts/agent.system.tool.*.md)
// ---------------------------------------------------------------------------

/// The JSON contract the model must emit (see prompts/agent.system.main.communication.md):
/// { "thoughts": [...], "tool_name": "...", "tool_args": { ... } }
#[derive(Debug, Clone, Deserialize)]
pub struct ToolRequest {
    #[serde(default)]
    pub thoughts: Vec<String>,
    pub tool_name: String,
    #[serde(default)]
    pub tool_args: Value,
}

pub enum ToolOutcome {
    /// Tool ran; result goes back into history and the loop continues.
    Continue(String),
    /// The `response` tool: final answer, break the monologue.
    FinalResponse(String),
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    async fn execute(&self, args: &Value, history: &mut History) -> AgentResult<ToolOutcome>;
}

#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<&'static str, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name(), tool);
    }
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }
}

/// The mandatory loop-breaker (tools/response.py).
pub struct ResponseTool;

#[async_trait]
impl Tool for ResponseTool {
    fn name(&self) -> &'static str { "response" }
    async fn execute(&self, args: &Value, _h: &mut History) -> AgentResult<ToolOutcome> {
        let text = args.get("text").and_then(|v| v.as_str()).unwrap_or_default();
        Ok(ToolOutcome::FinalResponse(text.to_string()))
    }
}

// ---------------------------------------------------------------------------
// LLM client (models.py / helpers/call_llm.py, trimmed to one trait)
// ---------------------------------------------------------------------------

#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Stream a completion. `on_chunk(chunk, full_so_far)` mirrors A0's
    /// stream_callback. Return the full response text.
    ///
    /// Implementation notes for your DeepSeek/OpenAI-compatible client:
    /// - reqwest POST /chat/completions with stream=true, parse SSE lines
    /// - wrap in tokio::time::timeout(...); on elapse return
    ///   AgentError::Repairable(prompts.read("fw.msg_timeout.md", ...))
    /// - A0 PARITY (optional): inside on_chunk, run extract_json_root +
    ///   json_parse_dirty on `full_so_far`; if a *valid* ToolRequest already
    ///   parses, abort the stream early to save tokens (agent.py ~line 443).
    async fn chat_stream(
        &self,
        system: &str,
        messages: &[ChatMessage],
        on_chunk: &(dyn Fn(&str, &str) + Send + Sync),
    ) -> AgentResult<String>;
}
