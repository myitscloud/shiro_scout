//! The monologue loop — structural port of agent.py::monologue() (agent0ai/agent-zero,
//! lines ~387–570 at time of extraction). Read core/agent.py in this package
//! side-by-side with this file; section comments reference it.
//!
//! Self-healing happens in exactly four places, all visible in run_message_loop():
//!   [H1] misformat  -> fw.msg_misformat.md   (unparseable output, keep looping)
//!   [H2] repeat     -> fw.msg_repeat.md      (model stuck repeating itself)
//!   [H3] tool error -> fw.error.md           (Repairable errors fed back verbatim)
//!   [H4] not found  -> fw.tool_not_found.md  (bad tool name, list valid tools)
//! Plus fw.intervention.md when the user interjects mid-run.

use crate::core::*;
use crate::prompts::{parse_tool_request, PromptStore};
use crate::vars;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct Agent {
    pub name: String,
    pub prompts: PromptStore,
    pub hooks: HookRegistry,
    pub tools: ToolRegistry,
    pub llm: Arc<dyn LlmClient>,
    pub history: History,
    pub loop_data: LoopData,
    /// Set from the UI thread (Tauri command) when the user sends a message
    /// mid-run; checked between every step like A0's handle_intervention().
    pub intervention: Arc<AtomicBool>,
    pub max_iterations: u64, // safety rail; A0 relies on nudge/repeat prompts instead
}

impl Agent {
    /// agent.py::monologue(). Returns the final `response` tool text.
    pub async fn monologue(&mut self, user_message: String) -> AgentResult<String> {
        self.history.add_user(&user_message);
        self.loop_data = LoopData { user_message: Some(user_message), ..Default::default() };

        self.hooks
            .run(HookPoint::MonologueStart, &mut self.loop_data, &mut self.history)
            .await?;

        let result = self.run_message_loop().await;

        // A0 runs monologue_end in a `finally` — memory plugin memorizes
        // fragments/solutions here (plugins-memory/extensions/python/monologue_end/).
        let _ = self
            .hooks
            .run(HookPoint::MonologueEnd, &mut self.loop_data, &mut self.history)
            .await;

        result
    }

    async fn run_message_loop(&mut self) -> AgentResult<String> {
        loop {
            self.loop_data.iteration += 1;
            if self.loop_data.iteration > self.max_iterations {
                return Err(AgentError::Critical("max iterations exceeded".into()));
            }

            self.hooks
                .run(HookPoint::MessageLoopStart, &mut self.loop_data, &mut self.history)
                .await?;

            let step = self.message_loop_step().await;

            // message_loop_end runs in A0's `finally` regardless of outcome.
            let _ = self
                .hooks
                .run(HookPoint::MessageLoopEnd, &mut self.loop_data, &mut self.history)
                .await;

            match step {
                Ok(Some(final_response)) => return Ok(final_response),
                Ok(None) => continue, // warning injected, model gets another try

                // [H3] THE repair path: error text goes back to the model.
                Err(AgentError::Repairable(err_text)) => {
                    let warning = self
                        .prompts
                        .read("fw.error.md", &vars! {"error" => err_text})?;
                    self.history.add_warning(warning);
                    continue;
                }

                Err(AgentError::Intervention) => {
                    self.intervention.store(false, Ordering::SeqCst);
                    let warning = self.prompts.read("fw.intervention.md", &vars! {})?;
                    self.history.add_warning(warning);
                    continue;
                }

                // Handled/Critical stop the run (A0's HandledException path).
                Err(fatal) => return Err(fatal),
            }
        }
    }

    /// One iteration: build prompt -> call LLM -> repeat-check -> parse -> tool.
    /// Ok(Some(text)) = `response` tool fired; Ok(None) = loop again.
    async fn message_loop_step(&mut self) -> AgentResult<Option<String>> {
        self.check_intervention()?;

        // --- prepare_prompt() ------------------------------------------------
        self.hooks
            .run(HookPoint::MessageLoopPromptsBefore, &mut self.loop_data, &mut self.history)
            .await?;

        // System prompt is COMPOSED by hooks, not one file: see
        // extensions-python/system_prompt/_10_main_prompt.py .. _14_project_prompt.py.
        // Your _10 hook should push prompts.read("agent.system.main.md", ...)
        // (which stitches role/communication/solving/tips) into system_prompt_parts.
        self.loop_data.system_prompt_parts.clear();
        self.hooks
            .run(HookPoint::SystemPrompt, &mut self.loop_data, &mut self.history)
            .await?;

        // Transient context: datetime, agent info, memory recall, skills —
        // extensions-python/message_loop_prompts_after/_60.._75 + memory plugin's _50.
        self.loop_data.extras.clear();
        self.hooks
            .run(HookPoint::MessageLoopPromptsAfter, &mut self.loop_data, &mut self.history)
            .await?;

        let mut system = self.loop_data.system_prompt_parts.join("\n\n");
        for (_, extra) in &self.loop_data.extras {
            system.push_str("\n\n");
            system.push_str(extra);
        }

        self.hooks
            .run(HookPoint::BeforeMainLlmCall, &mut self.loop_data, &mut self.history)
            .await?;
        // TODO: rate limiter here (helpers/rate_limiter.py) — you'll want this
        // for DeepSeek peak/off-peak budget control.

        // --- call main LLM ---------------------------------------------------
        self.check_intervention()?;
        let response = self
            .llm
            .chat_stream(&system, &self.history.messages, &|_chunk, _full| {
                // Stream to the UI here (Tauri event emit). See lib.rs.
                // A0 parity: early-stop once a valid ToolRequest parses from `_full`.
            })
            .await?;
        self.check_intervention()?;

        // --- [H2] repeat detection (agent.py ~line 505) ----------------------
        if self.loop_data.last_response.as_deref() == Some(response.as_str()) {
            self.history.add_ai_response(&response);
            let warning = self.prompts.read("fw.msg_repeat.md", &vars! {})?;
            self.history.add_warning(warning);
            return Ok(None);
        }
        self.loop_data.last_response = Some(response.clone());
        self.history.add_ai_response(&response);

        // --- [H1] parse tool request (agent.py ~line 1400-1510) --------------
        let Some(request) = parse_tool_request(&response) else {
            let warning = self.prompts.read("fw.msg_misformat.md", &vars! {})?;
            self.history.add_warning(warning);
            return Ok(None);
        };

        // --- [H4] + execute ---------------------------------------------------
        self.hooks
            .run(HookPoint::ToolExecuteBefore, &mut self.loop_data, &mut self.history)
            .await?;

        let Some(tool) = self.tools.get(&request.tool_name) else {
            // A0 routes this through tools/unknown.py; same effect:
            let warning = self.prompts.read(
                "fw.tool_not_found.md",
                &vars! {"tool_name" => request.tool_name, "tools_prompt" => "see system prompt"},
            )?;
            self.history.add_warning(warning);
            return Ok(None);
        };

        // Tool errors should surface as AgentError::Repairable(msg) so [H3]
        // catches them — that is what makes tools self-healing. Reserve
        // Critical for genuinely unrecoverable states.
        let outcome = tool.execute(&request.tool_args, &mut self.history).await?;

        self.hooks
            .run(HookPoint::ToolExecuteAfter, &mut self.loop_data, &mut self.history)
            .await?;

        match outcome {
            ToolOutcome::FinalResponse(text) => Ok(Some(text)),
            ToolOutcome::Continue(result) => {
                let rendered = self.prompts.read(
                    "fw.tool_result.md",
                    &vars! {"tool_name" => request.tool_name, "tool_result" => result},
                )?;
                self.history.add_tool_result(rendered);
                Ok(None)
            }
        }
    }

    fn check_intervention(&self) -> AgentResult<()> {
        if self.intervention.load(Ordering::SeqCst) {
            Err(AgentError::Intervention)
        } else {
            Ok(())
        }
    }
}
