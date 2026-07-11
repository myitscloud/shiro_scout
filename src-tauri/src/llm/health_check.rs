use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;

// --------------------------------------------------------------------------
// Types for IPC (C10: sanitized strings only)
// --------------------------------------------------------------------------

/// Result of a connection test sent over IPC to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConnectionResult {
    pub success: bool,
    pub latency_ms: Option<f64>,
    pub error: Option<String>,
}

impl TestConnectionResult {
    pub fn success(latency_ms: f64) -> Self {
        Self {
            success: true,
            latency_ms: Some(latency_ms),
            error: None,
        }
    }

    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            latency_ms: None,
            error: Some(error),
        }
    }
}

/// Cached health status for a single provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub provider: String,
    pub healthy: bool,
    pub latency_ms: Option<f64>,
    pub last_checked: String,
    pub error: Option<String>,
}

/// Input parameters for the test_llm_connection Tauri command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConnectionInput {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

/// Endpoint configuration for a registered provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderEndpoint {
    pub provider: String,
    pub model: String,
    pub base_url: String,
    pub api_key: Option<String>,
}

// --------------------------------------------------------------------------
// Fallback role priority from MSPEC-011
// --------------------------------------------------------------------------

/// Role used to determine fallback provider priority.
#[derive(Debug, Clone)]
pub enum FallbackRole {
    Chat,
    Utility,
    Embedding,
}

impl FallbackRole {
    /// Returns the priority-ordered provider list for this role.
    pub fn priority_providers(&self) -> &'static [&'static str] {
        match self {
            FallbackRole::Chat | FallbackRole::Utility => {
                &["deepseek", "openai", "groq", "together", "ollama", "litellm"]
            }
            FallbackRole::Embedding => &["deepseek"],
        }
    }
}

// --------------------------------------------------------------------------
// Health check error types (C7: typed errors)
// --------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum HealthError {
    #[error("Connection timed out")]
    Timeout,

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("{0}")]
    Unknown(String),
}

impl HealthError {
    /// Convert to a user-safe message for IPC (C10).
    pub fn user_message(&self) -> String {
        match self {
            HealthError::Timeout => "Connection timed out. Check network and try again.".to_string(),
            HealthError::ConnectionFailed(_) => {
                "Could not connect to the provider. Check the base URL and network.".to_string()
            }
            HealthError::AuthFailed(_) => {
                "Authentication failed. Check your API key.".to_string()
            }
            HealthError::RateLimited => "Rate limited. Try again later.".to_string(),
            HealthError::Unknown(msg) => format!("Connection test failed: {}", msg),
        }
    }
}

// --------------------------------------------------------------------------
// HTTP client trait (Q2: Win32 behind seams → seams for testability)
// --------------------------------------------------------------------------

/// HTTP client abstraction for testability.
/// Unit tests inject a mock; production uses the reqwest implementation.
#[async_trait::async_trait]
pub trait HttpClient: Send + Sync {
    async fn post_json(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        body: serde_json::Value,
        timeout: Duration,
    ) -> Result<(u16, String), HealthError>;
}

/// Production HTTP client backed by reqwest.
pub struct ReqwestClient;

#[async_trait::async_trait]
impl HttpClient for ReqwestClient {
    async fn post_json(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        body: serde_json::Value,
        timeout: Duration,
    ) -> Result<(u16, String), HealthError> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| HealthError::ConnectionFailed(e.to_string()))?;

        let mut req = client.post(url);
        for (k, v) in headers {
            req = req.header(*k, *v);
        }
        req = req.json(&body);

        let response = req.send().await.map_err(|e| {
            if e.is_timeout() {
                HealthError::Timeout
            } else {
                HealthError::ConnectionFailed(e.to_string())
            }
        })?;

        let status = response.status().as_u16();
        let body_text = response
            .text()
            .await
            .map_err(|e| HealthError::ConnectionFailed(e.to_string()))?;

        Ok((status, body_text))
    }
}

// --------------------------------------------------------------------------
// HealthCheck engine
// --------------------------------------------------------------------------

/// Provider health check engine with caching and fallback selection.
pub struct HealthCheck {
    http_client: Box<dyn HttpClient>,
    timeout: Duration,
    cache_duration: Duration,
    health_cache: Arc<RwLock<HashMap<String, (ProviderHealth, Instant)>>>,
}

impl HealthCheck {
    pub fn new(http_client: Box<dyn HttpClient>) -> Self {
        Self {
            http_client,
            timeout: Duration::from_secs(5),
            cache_duration: Duration::from_secs(60),
            health_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    #[allow(dead_code)]
    pub fn with_cache_duration(mut self, duration: Duration) -> Self {
        self.cache_duration = duration;
        self
    }

    /// Test connection to a provider with explicit credentials (for Test Connection button).
    /// Returns latency in milliseconds on success.
    pub async fn test_connection(
        &self,
        provider: &str,
        model: &str,
        api_key: Option<&str>,
        base_url: Option<&str>,
    ) -> Result<f64, HealthError> {
        let default_base_url = default_base_url_for(provider);
        let base_url = base_url.unwrap_or(default_base_url.as_str());

        let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

        let request_body = serde_json::json!({
            "model": model,
            "messages": [{"role": "user", "content": "Hello"}],
            "max_tokens": 1,
        });

        let mut headers: Vec<(&str, &str)> = vec![("Content-Type", "application/json")];
        let auth_header_value;
        if let Some(key) = api_key {
            if !key.is_empty() && provider != "ollama" {
                auth_header_value = format!("Bearer {}", key);
                headers.push(("Authorization", auth_header_value.as_str()));
            }
        }

        let start = Instant::now();
        let (status, body_text) = self
            .http_client
            .post_json(&url, &headers, request_body, self.timeout)
            .await?;
        let latency = start.elapsed().as_secs_f64() * 1000.0;

        match status {
            200..=299 => Ok(latency),
            401 | 403 => Err(HealthError::AuthFailed(format!(
                "Authentication failed (HTTP {})",
                status
            ))),
            429 => Err(HealthError::RateLimited),
            _ => {
                let err_msg = serde_json::from_str::<serde_json::Value>(&body_text)
                    .ok()
                    .and_then(|v| {
                        v["error"]["message"]
                            .as_str()
                            .map(|s| s.to_string())
                    })
                    .unwrap_or_else(|| format!("HTTP {}", status));
                Err(HealthError::Unknown(err_msg))
            }
        }
    }

    /// Check a specific provider, using cache if valid.
    pub async fn check_provider(
        &self,
        name: &str,
        endpoints: &HashMap<String, ProviderEndpoint>,
    ) -> Result<f64, HealthError> {
        // Check cache first
        {
            let cache = self.health_cache.read().await;
            if let Some((health, cached_at)) = cache.get(name) {
                if cached_at.elapsed() < self.cache_duration {
                    return if health.healthy {
                        health
                            .latency_ms
                            .ok_or_else(|| HealthError::Unknown("No latency data".into()))
                    } else {
                        Err(HealthError::Unknown(
                            health
                                .error
                                .clone()
                                .unwrap_or_else(|| "Unknown".into()),
                        ))
                    };
                }
            }
        }

        let endpoint = endpoints
            .get(name)
            .ok_or_else(|| HealthError::Unknown(format!("Provider '{}' not found", name)))?;

        let result = self
            .test_connection(
                &endpoint.provider,
                &endpoint.model,
                endpoint.api_key.as_deref(),
                Some(&endpoint.base_url),
            )
            .await;

        // Update cache
        let now = chrono::Utc::now().to_rfc3339();
        let (healthy, latency_ms, error) = match &result {
            Ok(latency) => (true, Some(*latency), None),
            Err(e) => (false, None, Some(e.to_string())),
        };

        let health_entry = ProviderHealth {
            provider: name.to_string(),
            healthy,
            latency_ms,
            last_checked: now,
            error,
        };

        {
            let mut cache = self.health_cache.write().await;
            cache.insert(name.to_string(), (health_entry, Instant::now()));
        }

        result
    }

    /// Check all registered providers.
    pub async fn check_all(
        &self,
        endpoints: &HashMap<String, ProviderEndpoint>,
    ) -> HashMap<String, Result<f64, HealthError>> {
        let providers: Vec<String> = endpoints.keys().cloned().collect();
        let mut results = HashMap::new();

        for name in providers {
            let r = self.check_provider(&name, endpoints).await;
            results.insert(name, r);
        }

        results
    }

    /// Select a fallback provider from the priority list for the given role.
    /// Returns None when all providers in the priority list have failed.
    pub fn select_fallback(
        &self,
        configured_provider: &str,
        failed_providers: &[String],
        role: FallbackRole,
    ) -> Option<String> {
        let priority = role.priority_providers();

        // Find the configured provider's position in the priority list
        let current_idx = priority.iter().position(|p| *p == configured_provider)?;

        // Try next providers in priority order
        for candidate in &priority[current_idx + 1..] {
            if !failed_providers.iter().any(|f| f == candidate) {
                return Some(candidate.to_string());
            }
        }

        None
    }

    /// Get cached health status for all providers.
    pub async fn get_health_status(&self) -> HashMap<String, ProviderHealth> {
        let cache = self.health_cache.read().await;
        cache
            .iter()
            .map(|(k, (h, _))| (k.clone(), h.clone()))
            .collect()
    }

    #[allow(dead_code)]
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

/// Default base URL for known providers.
pub fn default_base_url_for(provider: &str) -> String {
    match provider {
        "deepseek" => "https://api.deepseek.com/v1".to_string(),
        "openai" => "https://api.openai.com/v1".to_string(),
        "groq" => "https://api.groq.com/openai/v1".to_string(),
        "together" => "https://api.together.xyz/v1".to_string(),
        "ollama" => "http://localhost:11434/v1".to_string(),
        "litellm" => "http://localhost:4000/v1".to_string(),
        _ => format!("https://api.{}.com/v1", provider),
    }
}

// --------------------------------------------------------------------------
// Tauri commands
// --------------------------------------------------------------------------

/// Tauri command: test an LLM connection with given credentials.
/// Returns TestConnectionResult over IPC.
#[tauri::command]
pub async fn get_provider_health(
    app_handle: tauri::AppHandle,
) -> Result<HashMap<String, ProviderHealth>, String> {
    // Currently returns empty map. When HealthCheck is wired as managed state,
    // this will return cached health data. For now, try to load configured
    // providers from settings and run checks.
    let settings = crate::settings::load_llm_settings_internal(&app_handle)
        .map_err(|e| format!("Failed to load settings: {}", e))?;

    let mut endpoints = HashMap::new();

    for (role, setting) in [
        ("chat", &settings.chat),
        ("utility", &settings.utility),
        ("embedding", &settings.embedding),
    ] {
        let base_url = default_base_url_for(&setting.provider);
        endpoints.insert(
            role.to_string(),
            ProviderEndpoint {
                provider: setting.provider.clone(),
                model: setting.model.clone(),
                base_url,
                api_key: setting.api_key.clone(),
            },
        );
    }

    let hc = HealthCheck::new(Box::new(ReqwestClient))
        .with_timeout(Duration::from_secs(5));

    hc.check_all(&endpoints).await;
    let status = hc.get_health_status().await;
    Ok(status)
}

// --------------------------------------------------------------------------
// Unit tests (Q1: no network, Q6: deterministic)
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    // ----------------------------------------------------------------------
    // Mock HTTP client for unit tests
    // ----------------------------------------------------------------------

    struct MockHttpClient {
        status: u16,
        body: String,
        delay: Option<Duration>,
        call_count: Arc<AtomicU64>,
    }

    impl MockHttpClient {
        fn new(status: u16, body: &str) -> Self {
            Self {
                status,
                body: body.to_string(),
                delay: None,
                call_count: Arc::new(AtomicU64::new(0)),
            }
        }

        #[allow(dead_code)]
        fn with_delay(mut self, delay: Duration) -> Self {
            self.delay = Some(delay);
            self
        }
    }

    #[async_trait::async_trait]
    impl HttpClient for MockHttpClient {
        async fn post_json(
            &self,
            _url: &str,
            _headers: &[(&str, &str)],
            _body: serde_json::Value,
            timeout: Duration,
        ) -> Result<(u16, String), HealthError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            if let Some(delay) = self.delay {
                if delay > timeout {
                    tokio::time::sleep(timeout + Duration::from_millis(10)).await;
                    return Err(HealthError::Timeout);
                }
                tokio::time::sleep(delay).await;
            }
            Ok((self.status, self.body.clone()))
        }
    }

    // ----------------------------------------------------------------------
    // test_health_check_success — mock returns 200 immediately
    // ----------------------------------------------------------------------

    #[tokio::test]
    async fn test_health_check_success() {
        let client = MockHttpClient::new(200, r#"{"id":"test"}"#);
        let hc = HealthCheck::new(Box::new(client));
        let result = hc
            .test_connection(
                "deepseek",
                "deepseek-v4-flash",
                Some("sk-test"),
                Some("https://api.deepseek.com/v1"),
            )
            .await;
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        let latency = result.unwrap();
        assert!(latency >= 0.0, "Latency should be non-negative");
    }

    // ----------------------------------------------------------------------
    // test_health_check_auth_failure — mock returns 401
    // ----------------------------------------------------------------------

    #[tokio::test]
    async fn test_health_check_auth_failure() {
        let client =
            MockHttpClient::new(401, r#"{"error":{"message":"Invalid API key"}}"#);
        let hc = HealthCheck::new(Box::new(client));
        let result = hc
            .test_connection(
                "deepseek",
                "deepseek-v4-flash",
                Some("bad-key"),
                Some("https://api.deepseek.com/v1"),
            )
            .await;
        assert!(result.is_err(), "Expected Err, got {:?}", result);
        match result.unwrap_err() {
            HealthError::AuthFailed(msg) => {
                assert!(
                    msg.contains("401"),
                    "AuthFailed should mention HTTP status"
                );
            }
            other => panic!("Expected AuthFailed, got {:?}", other),
        }
    }

    // ----------------------------------------------------------------------
    // test_health_check_timeout — mock that never responds
    // ----------------------------------------------------------------------

    #[tokio::test]
    async fn test_health_check_timeout() {
        let client =
            MockHttpClient::new(200, "ok").with_delay(Duration::from_secs(10));
        let hc =
            HealthCheck::new(Box::new(client)).with_timeout(Duration::from_millis(50));
        let result = hc
            .test_connection(
                "deepseek",
                "deepseek-v4-flash",
                Some("sk-test"),
                Some("https://api.deepseek.com/v1"),
            )
            .await;
        assert!(result.is_err(), "Expected Err, got {:?}", result);
        match result.unwrap_err() {
            HealthError::Timeout => {}
            other => panic!("Expected Timeout, got {:?}", other),
        }
    }

    // ----------------------------------------------------------------------
    // test_fallback_selection — verify fallback chain works
    // ----------------------------------------------------------------------

    #[tokio::test]
    async fn test_fallback_selection() {
        let client = MockHttpClient::new(200, "ok");
        let hc = HealthCheck::new(Box::new(client));
        let failed = vec!["deepseek".to_string()];
        let fallback = hc.select_fallback("deepseek", &failed, FallbackRole::Chat);
        assert_eq!(fallback, Some("openai".to_string()));
    }

    // ----------------------------------------------------------------------
    // test_fallback_exhausted — all providers fail
    // ----------------------------------------------------------------------

    #[tokio::test]
    async fn test_fallback_exhausted() {
        let client = MockHttpClient::new(200, "ok");
        let hc = HealthCheck::new(Box::new(client));
        let failed = vec![
            "deepseek".to_string(),
            "openai".to_string(),
            "groq".to_string(),
            "together".to_string(),
            "ollama".to_string(),
            "litellm".to_string(),
        ];
        let fallback = hc.select_fallback("deepseek", &failed, FallbackRole::Chat);
        assert_eq!(fallback, None);
    }

    // ----------------------------------------------------------------------
    // test_embedding_no_fallback — embedding has only 1 provider
    // ----------------------------------------------------------------------

    #[tokio::test]
    async fn test_embedding_no_fallback() {
        let client = MockHttpClient::new(200, "ok");
        let hc = HealthCheck::new(Box::new(client));
        let failed = vec!["deepseek".to_string()];
        let fallback = hc.select_fallback("deepseek", &failed, FallbackRole::Embedding);
        assert_eq!(fallback, None);
    }

    // ----------------------------------------------------------------------
    // test_fallback_skip_failed — skips intermediate failed providers
    // ----------------------------------------------------------------------

    #[tokio::test]
    async fn test_fallback_skip_failed() {
        let client = MockHttpClient::new(200, "ok");
        let hc = HealthCheck::new(Box::new(client));
        let failed = vec![
            "deepseek".to_string(),
            "openai".to_string(),
            "groq".to_string(),
        ];
        let fallback = hc.select_fallback("deepseek", &failed, FallbackRole::Chat);
        assert_eq!(fallback, Some("together".to_string()));
    }

    // ----------------------------------------------------------------------
    // test_default_base_urls — each provider gets the right default
    // ----------------------------------------------------------------------

    #[test]
    fn test_default_base_urls() {
        assert_eq!(
            default_base_url_for("deepseek"),
            "https://api.deepseek.com/v1"
        );
        assert_eq!(
            default_base_url_for("openai"),
            "https://api.openai.com/v1"
        );
        assert_eq!(
            default_base_url_for("groq"),
            "https://api.groq.com/openai/v1"
        );
        assert_eq!(
            default_base_url_for("together"),
            "https://api.together.xyz/v1"
        );
        assert_eq!(
            default_base_url_for("ollama"),
            "http://localhost:11434/v1"
        );
        assert_eq!(
            default_base_url_for("litellm"),
            "http://localhost:4000/v1"
        );
    }
}
