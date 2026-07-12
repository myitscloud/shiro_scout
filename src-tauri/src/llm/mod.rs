//! LLM integration using Rig crate (v0.39.0).
//! Supports: DeepSeek, OpenAI, Groq, Together, Ollama, LiteLLM.
//! Default: DeepSeek-v4-flash (user preference) for all 3 roles.

pub mod credential_manager;
pub mod health_check;
pub mod keychain;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

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

// --------------------------------------------------------------------------
// Tauri command: stream an LLM completion and emit tokens via IPC events
// --------------------------------------------------------------------------

/// Tauri command that calls the LLM provider's /chat/completions endpoint
/// with `stream: true`, parses the SSE response, and emits each token
/// as an `llm-token` IPC event consumed by the frontend's useStreamingLlm hook.
///
/// Returns Ok(()) when the stream completes. On error, emits a final event
/// with `done: true` and returns an Err.
#[tauri::command]
pub async fn stream_llm_completion(
    app_handle: AppHandle,
    input: StreamCompletionInput,
) -> Result<(), String> {
    let base_url = input
        .base_url
        .clone()
        .unwrap_or_else(|| default_base_url_for(&input.provider));
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    // Resolve API key: explicit input wins; otherwise fall back to the
    // host-side keychain (Windows Credential Manager → env var → settings).
    // ADR-005: keys are host-resident; the WebView need not supply them.
    let resolved_api_key: Option<String> = match input.api_key.as_deref() {
        Some(k) if !k.is_empty() => Some(k.to_string()),
        _ => {
            let mut kc = keychain::Keychain::new(None);
            kc.load_api_key(&input.provider)
        }
    };

    if resolved_api_key.is_none() && input.provider != "ollama" {
        let _ = app_handle.emit(
            "llm-token",
            LlmTokenPayload { role: input.role.clone(), token: String::new(), done: true },
        );
        return Err(format!(
            "No API key found for provider '{}' (checked request, Credential Manager, environment)",
            input.provider
        ));
    }

    // Build messages array: prepend system prompt if provided
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

    // Build HTTP client and request
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let mut req = client.post(&url);

    // Add authorization header
    if let Some(ref key) = resolved_api_key {
        if !key.is_empty() && input.provider != "ollama" {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
    }
    req = req.header("Content-Type", "application/json");
    req = req.header("Accept", "text/event-stream");
    req = req.json(&request_body);

    // Send request
    let response = req
        .send()
        .await
        .map_err(|e| {
            let msg = if e.is_timeout() {
                "Request timed out".to_string()
            } else if e.is_connect() {
                "Could not connect to provider".to_string()
            } else {
                format!("Request failed: {}", e)
            };
            msg
        })?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        let err_msg = serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|v| v["error"]["message"].as_str().map(String::from))
            .unwrap_or_else(|| format!("HTTP {}", status.as_u16()));

        // Emit a final done event so the frontend can clean up
        let _ = app_handle.emit(
            "llm-token",
            LlmTokenPayload {
                role: input.role.clone(),
                token: String::new(),
                done: true,
            },
        );
        return Err(err_msg);
    }

    // Stream SSE response
    use futures::StreamExt;

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Stream read error: {}", e))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        // Process complete SSE lines from the buffer
        while let Some(line_end) = buffer.find('\n') {
            let line = buffer[..line_end].trim_end().to_string();
            buffer.drain(..=line_end);

            // SSE data lines start with "data: "
            if let Some(data_str) = line.strip_prefix("data: ") {
                let trimmed = data_str.trim();

                // SSE stream end signal
                if trimmed == "[DONE]" {
                    break;
                }

                // Parse JSON chunk
                if let Ok(parsed) =
                    serde_json::from_str::<serde_json::Value>(trimmed)
                {
                    // Extract delta content from choices[0].delta.content
                    if let Some(choices) = parsed["choices"].as_array() {
                        if let Some(choice) = choices.first() {
                            if let Some(delta) = choice["delta"].as_object() {
                                if let Some(content) = delta["content"].as_str() {
                                    if !content.is_empty() {
                                        let _ = app_handle.emit(
                                            "llm-token",
                                            LlmTokenPayload {
                                                role: input.role.clone(),
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
            role: input.role.clone(),
            token: String::new(),
            done: true,
        },
    );

    Ok(())
}
