//! monologue-core: Agent Zero's loop, prompts, and repair model in Rust.
//!
//! STATUS: structural skeleton, NOT compile-verified (no toolchain in the
//! extraction environment). Expect to adjust lifetimes/signatures when wiring
//! into your workspace. The A0 sources it ports sit alongside this crate in
//! the package — treat core/agent.py as the reference implementation.
//!
//! Suggested build order:
//!   1. Implement LlmClient for DeepSeek (reqwest + SSE + tokio timeout).
//!   2. Register ResponseTool + one real tool (e.g. PowerShell runner via
//!      tokio::process, stdout/stderr -> ToolOutcome::Continue).
//!   3. Add a SystemPrompt hook that reads agent.system.main.md.
//!   4. Run the loop headless; only then wire Tauri events/commands.

pub mod agent;
pub mod core;
pub mod prompts;

use crate::core::*;
use async_trait::async_trait;

/// Example SystemPrompt hook = extensions-python/system_prompt/_10_main_prompt.py.
pub struct MainSystemPrompt {
    pub store: std::sync::Arc<prompts::PromptStore>,
    pub agent_name: String,
}

#[async_trait]
impl Extension for MainSystemPrompt {
    fn order(&self) -> u32 { 10 }
    async fn execute(&self, loop_data: &mut LoopData, _h: &mut History) -> AgentResult<()> {
        // agent.system.main.md stitches role/communication/solving/tips.
        // If you skip A0's include mechanism, read the parts individually:
        for part in [
            "agent.system.main.role.md",
            "agent.system.main.environment.md",
            "agent.system.main.communication.md",
            "agent.system.main.solving.md",
            "agent.system.main.tips.md",
        ] {
            let text = self.store.read(part, &vars! {"agent_name" => self.agent_name})?;
            loop_data.system_prompt_parts.push(text);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tauri 2 wiring sketch (enable feature "tauri-app" + tauri dep)
// ---------------------------------------------------------------------------
//
// #[cfg(feature = "tauri-app")]
// pub mod tauri_glue {
//     use super::*;
//     use std::sync::Arc;
//     use tauri::{AppHandle, Emitter, State};
//     use tokio::sync::Mutex;
//
//     pub struct AgentState(pub Arc<Mutex<agent::Agent>>);
//
//     #[tauri::command]
//     pub async fn agent_send(
//         app: AppHandle,
//         state: State<'_, AgentState>,
//         message: String,
//     ) -> Result<String, String> {
//         let mut agent = state.0.lock().await;
//         // Stream chunks to React via app.emit("agent://chunk", chunk) from
//         // the LlmClient on_chunk callback; emit "agent://tool" on tool calls.
//         agent.monologue(message).await.map_err(|e| e.to_string())
//     }
//
//     #[tauri::command]
//     pub async fn agent_interrupt(state: State<'_, AgentState>) -> Result<(), String> {
//         let agent = state.0.lock().await;
//         agent.intervention.store(true, std::sync::atomic::Ordering::SeqCst);
//         Ok(()) // next check_intervention() injects fw.intervention.md
//     }
// }
//
// SECURITY NOTES (your priorities):
// - Keep the loop in the Rust core, never in the WebView. React only renders
//   events and calls the two commands above.
// - Port helpers/secrets.py's masking idea: scrub credentials from any error
//   text BEFORE it enters history via fw.error.md (A0 does this in
//   extensions-python/error_format/_10_mask_errors.py).
// - Tool allowlist per agent profile (agents/<profile>/agent.yaml) rather
//   than a global registry, if subordinate agents will run untrusted tasks.
