/// The main Agent struct - the core orchestrator agent loop.
/// Ported from mimicking-agent-zero/agent.rs (Agent + AgentLoop).
///
/// Self-healing loop implements H1-H4:
/// [H1] Misformat  → LLM output didn't parse as JSON → inject fw.msg_misformat.md
/// [H2] Repeat     → Output identical to previous turn → inject fw.msg_repeat.md
/// [H3] Error      → Tool returned Repairable error → inject fw.error.md
/// [H4] Not Found  → Tool name doesn't match registry → inject fw.tool_not_found.md
///
/// State Machine (Wave 0 → Wave 4.1): idle → thinking (with SSE token events)
/// → tool (bridge invoke) → done.
use crate::agent::context::AgentContext;
use crate::agent::state::AgentState;
use crate::bridge_client::ToolExecBridge;
use crate::error::{AgentError, AgentResult};
use crate::prompts::PromptStore;
use crate::tools::ToolRegistry;
use crate::llm::{LlmTokenPayload, StreamCompletionInput, StreamMessage};
use rig::client::CompletionClient;
use rig::completion::Chat;
use tauri::{AppHandle, Emitter};

#[derive(Debug)]
pub struct Agent {
    pub name: String,
    pub context: AgentContext,
    pub tools: ToolRegistry,
    pub prompts: PromptStore,
    pub llm_provider: Option<String>,
    pub llm_model: Option<String>,
    pub system_prompt: Option<String>,
    /// Optional handle for emitting llm-token events during thinking state.
    pub app_handle: Option<AppHandle>,
}

impl Agent {
    pub async fn new_orchestrator() -> Self {
        let context = AgentContext::default_orchestrator();
        let tools = ToolRegistry::new();
        let mut prompts = PromptStore::new();
        let _ = prompts.load_from(&context.config.prompts_path);

        Self {
            name: "orchestrator".into(),
            context,
            tools,
            prompts,
            llm_provider: Some("deepseek".into()),
            llm_model: Some("deepseek-v4-flash".into()),
            system_prompt: None,
            app_handle: None,
        }
    }

    /// Attach a Tauri AppHandle for event emission during streaming.
    pub fn with_app_handle(mut self, app_handle: AppHandle) -> Self {
        self.app_handle = Some(app_handle);
        self
    }

    /// Attach a ToolExecBridge for sandboxed Docker exec tool execution.
    pub fn with_tool_bridge(mut self, bridge: ToolExecBridge) -> Self {
        self.tools = self.tools.with_bridge(bridge);
        self
    }

    pub async fn boot(&mut self) -> AgentResult<String> {
        self.context.transition_to(AgentState::Booting);

        let main_prompt = self.prompts.main_system_prompt()?;
        let specifics = self.prompts.specifics_prompt()?;

        let system_prompt = format!("{}\n\n{}", main_prompt, specifics);
        self.system_prompt = Some(system_prompt.clone());

        self.context.transition_to(AgentState::Ready);

        Ok(system_prompt)
    }

    /// Process a user message through the full state machine cycle:
    /// idle → thinking (SSE token streaming) → tool (bridge invoke) → done.
    ///
    /// When app_handle is available, emits `llm-token` events during thinking
    /// so the frontend can display incremental tokens via StreamingText.
    pub async fn process_message(&mut self, user_message: &str) -> AgentResult<String> {
        self.context.history.add_user(user_message);
        self.context.reset_loop();
        self.context.transition_to(AgentState::Idle);

        loop {
            if self.context.loop_data.iteration > self.context.config.max_iterations {
                return Err(AgentError::Critical("Maximum iterations exceeded".into()));
            }

            if self.context.intervention_flag {
                self.context.clear_intervention();
                let intervention = self.prompts.fw_intervention();
                self.context.history.add_warning(&intervention);
                continue;
            }

            // ── idle → thinking (with streaming token events) ──
            self.context.transition_to(AgentState::Thinking);
            let model_response = match self.think().await {
                Ok(response) => response,
                Err(AgentError::Intervention) => {
                    let intervention = self.prompts.fw_intervention();
                    self.context.history.add_warning(&intervention);
                    continue;
                }
                Err(e) => return Err(e),
            };

            // [H2] Repeat detection
            if let Some(ref last) = self.context.loop_data.last_response {
                if last == &model_response {
                    let repeat_msg = self.prompts.fw_repeat();
                    self.context.history.add_warning(&repeat_msg);
                    self.context.loop_data.iteration += 1;
                    continue;
                }
            }
            self.context.loop_data.last_response = Some(model_response.clone());

            // Parse the model response as JSON tool request or plain text
            match self.parse_tool_request(&model_response) {
                Ok(Ok((tool_name, tool_args))) => {
                    if !self.tools.has_tool(&tool_name) {
                        let not_found = self.prompts.tool_not_found(&tool_name);
                        self.context.history.add_warning(&not_found);
                        self.context.loop_data.iteration += 1;
                        continue;
                    }
                    // ── thinking → tool (executing via bridge) ──
                    self.context.transition_to(AgentState::Executing);
                    let result = self.tools.execute(&tool_name, &tool_args).await;
                    match result {
                        Ok(output) => {
                            self.context.history.add_tool(&tool_name, &output);
                            // For response() tool, mark done and return result
                            if tool_name == "response" {
                                self.context.transition_to(AgentState::Done);
                                return Ok(output);
                            }
                            // Return to thinking for next LLM call
                            self.context.transition_to(AgentState::Thinking);
                        }
                        Err(AgentError::Repairable(_err_msg)) => {
                            // [H3] Error: push error context
                            let error_prompt = self.prompts.fw_error();
                            self.context.history.add_warning(&error_prompt);
                            self.context.loop_data.iteration += 1;
                        }
                        Err(e) => return Err(e),
                    }
                }
                Ok(Err(text)) => {
                    // Not a tool call — final response. Mark done and return.
                    self.context.transition_to(AgentState::Streaming);
                    self.context.transition_to(AgentState::Done);
                    return Ok(text);
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Think by calling the LLM provider with streaming SSE events.
    /// When app_handle is available, emits `llm-token` events per token chunk
    /// so the frontend displays incremental text via StreamingText.
    /// Falls back to the llm_call builder when AppHandle is absent.
    async fn think(&self) -> AgentResult<String> {
        // If we have an AppHandle, use the streaming LLM infrastructure
        // to emit llm-token events during thinking.
        if let Some(ref app_handle) = self.app_handle {
            return self.think_with_streaming(app_handle).await;
        }
        // Legacy path: no streaming
        self.llm_call().await
    }

    /// Streaming thinking: calls the LLM provider via the streaming infrastructure
    /// from llm/mod.rs, emitting `llm-token` events per chunk so the frontend
    /// can display incremental tokens.
    async fn think_with_streaming(&self, app_handle: &AppHandle) -> AgentResult<String> {
        let provider_name = &self.context.provider.name;
        let model = &self.context.provider.model;
        let api_key = self.context.provider.api_key.as_deref().unwrap_or("");
        let base_url = self.context.provider.base_url.clone();
        let max_tokens = self.context.provider.max_tokens;
        let temperature = self.context.provider.temperature;

        let system_prompt = self.system_prompt.as_deref().unwrap_or("You are a helpful AI assistant.");
        let messages: Vec<StreamMessage> = self.context.history.messages.iter().map(|m| {
            StreamMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            }
        }).collect();

        let input = StreamCompletionInput {
            provider: provider_name.clone(),
            model: model.clone(),
            api_key: if api_key.is_empty() { None } else { Some(api_key.to_string()) },
            base_url,
            system_prompt: Some(system_prompt.to_string()),
            messages,
            max_tokens,
            temperature,
            role: "assistant".to_string(),
        };

        // Re-use the streaming logic from llm/mod.rs by constructing
        // a reqwest client, calling the provider, and parsing SSE events.
        let base_url = input
            .base_url
            .clone()
            .unwrap_or_else(|| crate::llm::default_base_url_for(&input.provider));
        let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

        let mut api_messages: Vec<serde_json::Value> = Vec::new();
        if let Some(system) = &input.system_prompt {
            api_messages.push(serde_json::json!({"role": "system", "content": system}));
        }
        for msg in &input.messages {
            api_messages.push(serde_json::json!({"role": msg.role, "content": msg.content}));
        }

        let request_body = serde_json::json!({
            "model": input.model,
            "messages": api_messages,
            "max_tokens": input.max_tokens,
            "temperature": input.temperature,
            "stream": true,
        });

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .map_err(|e| AgentError::Critical(format!("Failed to create HTTP client: {}", e)))?;

        let mut req = client.post(&url);
        if let Some(ref key) = input.api_key {
            if !key.is_empty() && input.provider != "ollama" {
                req = req.header("Authorization", format!("Bearer {}", key));
            }
        }
        req = req.header("Content-Type", "application/json");
        req = req.header("Accept", "text/event-stream");
        req = req.json(&request_body);

        let response = req.send().await
            .map_err(|e| AgentError::Critical(format!("LLM request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            let err_msg = serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|v| v["error"]["message"].as_str().map(String::from))
                .unwrap_or_else(|| format!("HTTP {}", status.as_u16()));

            let _ = app_handle.emit(
                "llm-token",
                LlmTokenPayload {
                    role: "assistant".to_string(),
                    token: String::new(),
                    done: true,
                },
            );
            return Err(AgentError::Critical(err_msg));
        }

        use futures::StreamExt;
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut full_text = String::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| AgentError::Critical(format!("Stream read error: {}", e)))?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(pos) = buffer.find('\n') {
                let line_end = pos;
                let line = buffer[..line_end].trim_end().to_string();
                buffer.drain(..=line_end);

                if let Some(data_str) = line.strip_prefix("data: ") {
                    let trimmed = data_str.trim();
                    if trimmed == "[DONE]" {
                        break;
                    }

                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
                        if let Some(choices) = parsed["choices"].as_array() {
                            if let Some(choice) = choices.first() {
                                if let Some(delta) = choice["delta"].as_object() {
                                    if let Some(content) = delta["content"].as_str() {
                                        if !content.is_empty() {
                                            full_text.push_str(content);
                                            let _ = app_handle.emit(
                                                "llm-token",
                                                LlmTokenPayload {
                                                    role: "assistant".to_string(),
                                                    token: content.to_string(),
                                                    done: false,
                                                },
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Emit final done event
        let _ = app_handle.emit(
            "llm-token",
            LlmTokenPayload {
                role: "assistant".to_string(),
                token: String::new(),
                done: true,
            },
        );

        if full_text.is_empty() {
            return Err(AgentError::Critical("LLM returned empty response".into()));
        }
        Ok(full_text)
    }

    /// Real LLM call using rig v0.39.0 provider (non-streaming fallback).
    async fn llm_call(&self) -> AgentResult<String> {
        let provider_name = &self.context.provider.name;
        let model = &self.context.provider.model;
        let api_key = self.context.provider.api_key.as_deref().unwrap_or("");
        let base_url = self.context.provider.base_url.as_deref();
        let max_tokens = self.context.provider.max_tokens as u64;
        let temperature = self.context.provider.temperature;

        let system_prompt = self.system_prompt.as_deref().unwrap_or("You are a helpful AI assistant.");
        let messages = self.context.history.to_messages();

        let (_client_url, client_api_key) = match provider_name.as_str() {
            "deepseek" => (base_url.unwrap_or("https://api.deepseek.com/v1"), api_key),
            "openai" => (base_url.unwrap_or("https://api.openai.com/v1"), api_key),
            "groq" => (base_url.unwrap_or("https://api.groq.com/openai/v1"), api_key),
            "together" => (base_url.unwrap_or("https://api.together.xyz/v1"), api_key),
            "ollama" => (base_url.unwrap_or("http://localhost:11434/v1"), "ollama"),
            "litellm" => (base_url.unwrap_or("http://localhost:4000/v1"), api_key),
            _ => return Err(AgentError::Critical(format!("Unknown provider: {}", provider_name))),
        };

        let client = rig::providers::openai::Client::new(client_api_key)
            .map_err(|e| AgentError::Critical(format!("Failed to create LLM client: {}", e)))?;
        let agent = client
            .agent(model)
            .preamble(system_prompt)
            .temperature(temperature)
            .max_tokens(max_tokens)
            .build();

        let mut chat_history = Vec::new();
        let response = agent
            .chat(messages, &mut chat_history)
            .await
            .map_err(|e| AgentError::Critical(format!("LLM call failed: {}", e)))?;

        Ok(response)
    }

    /// Parse LLM response into (tool_name, tool_args) or plain text.
    /// Returns Ok(Ok((name, args))) for a valid tool call,
    /// Ok(Err(text)) for plain text response,
    /// Err(e) for parse error.
    fn parse_tool_request(&self, response: &str) -> Result<Result<(String, serde_json::Value), String>, AgentError> {
        // Try JSON parse first
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(response) {
            if let Some(obj) = val.as_object() {
                if let Some(tn) = obj.get("tool_name").and_then(|v| v.as_str()) {
                    let tool_args = obj.get("tool_args").cloned().unwrap_or(serde_json::Value::Null);
                    return Ok(Ok((tn.to_string(), tool_args)));
                }
            }
        }

        // Fallback: `passe tool_name arg1 val1`
        if let Some(passed) = response.strip_prefix("passe ") {
            let parts: Vec<&str> = passed.split_whitespace().collect();
            if parts.len() >= 2 {
                let tool_name = parts[0].to_string();
                let args_text = parts[1..].join(" ");
                let mut map = serde_json::Map::new();
                map.insert("text".to_string(), serde_json::Value::String(args_text));
                return Ok(Ok((tool_name, serde_json::Value::Object(map))));
            }
        }

        // Presume it's final response
        Ok(Err(response.to_string()))
    }
}
