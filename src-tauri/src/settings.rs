use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

use crate::llm::keychain::Keychain;

// --------------------------------------------------------------------------
// Data structures
// --------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub theme: String,
    pub reduce_motion: bool,
    pub provider: String,
    pub model: String,
    pub api_key: String,
    #[serde(alias = "workspacePath")]
    pub workspace_path: String,
    pub last_session_id: Option<String>,
    /// HITL confirmation timeout in seconds (default 30). Range: 5-120.
    pub hitl_timeout_secs: u32,
    /// HITL dangerous operations classification list.
    pub dangerous_operations: DangerousOperationsConfig,
    /// Air-gapped mode default: true means containers start with network_mode: none.
    pub sandbox_air_gapped: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            reduce_motion: false,
            provider: "local".to_string(),
            model: "gpt-4o".to_string(),
            api_key: String::new(),
            workspace_path: String::new(),
            last_session_id: None,
            hitl_timeout_secs: 30,
            dangerous_operations: DangerousOperationsConfig::default(),
            sandbox_air_gapped: true,
        }
    }
}

// --------------------------------------------------------------------------
// HITL (Human-In-The-Loop) configuration (Wave 7.1)
// --------------------------------------------------------------------------

/// Classification of dangerous operations that require HITL confirmation.
/// Configurable per Risk Level: Low, Medium, High, Critical.
/// Each level determines the UI treatment (color, required checkbox, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DangerousOperationsConfig {
    /// Operation names classified as Critical risk (requires checkbox + approval)
    pub critical: Vec<String>,
    /// Operation names classified as High risk (requires warning color + approval)
    pub high: Vec<String>,
    /// Operation names classified as Medium risk (requires approval)
    pub medium: Vec<String>,
    /// Operation names classified as Low risk (informational HITL only)
    pub low: Vec<String>,
}

impl Default for DangerousOperationsConfig {
    fn default() -> Self {
        Self {
            critical: vec![
                "delete_files".to_string(),
                "format_disk".to_string(),
                "shutdown_system".to_string(),
            ],
            high: vec![
                "execute_command".to_string(),
                "network_connect".to_string(),
                "toggle_network_mode".to_string(),
                "modify_registry".to_string(),
                "install_software".to_string(),
                "modify_system_settings".to_string(),
            ],
            medium: vec![
                "write_file".to_string(),
                "modify_file".to_string(),
                "create_directory".to_string(),
                "download_file".to_string(),
                "upload_file".to_string(),
                "start_service".to_string(),
                "stop_service".to_string(),
            ],
            low: vec![
                "read_file".to_string(),
                "list_directory".to_string(),
                "get_system_info".to_string(),
            ],
        }
    }
}

impl DangerousOperationsConfig {
    /// Returns the risk level for a given operation name, or None if not classified.
    pub fn risk_level(&self, operation: &str) -> Option<&str> {
        if self.critical.contains(&operation.to_string()) {
            Some("critical")
        } else if self.high.contains(&operation.to_string()) {
            Some("high")
        } else if self.medium.contains(&operation.to_string()) {
            Some("medium")
        } else if self.low.contains(&operation.to_string()) {
            Some("low")
        } else {
            None
        }
    }

    /// Returns true if the operation is classified as any dangerous level.
    pub fn is_dangerous(&self, operation: &str) -> bool {
        self.risk_level(operation).is_some()
    }
}

// --------------------------------------------------------------------------
// Helpers
// --------------------------------------------------------------------------

fn settings_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let config_dir = app_handle
        .path()
        .app_config_dir()
        .expect("Failed to get app config dir");
    std::fs::create_dir_all(&config_dir).ok();
    config_dir.join("settings.json")
}

// --------------------------------------------------------------------------
// Commands — app settings
// --------------------------------------------------------------------------

/// Load saved settings from the app config directory.
/// Returns None if no settings file exists yet (first run).
#[tauri::command]
pub fn load_settings(app_handle: tauri::AppHandle) -> Result<Option<AppSettings>, String> {
    let path = settings_path(&app_handle);
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;
    let settings: AppSettings = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings file: {}", e))?;
    Ok(Some(settings))
}

/// Save settings to the app config directory.
#[tauri::command]
pub fn save_settings(app_handle: tauri::AppHandle, settings: AppSettings) -> Result<(), String> {
    let path = settings_path(&app_handle);
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write settings file: {}", e))?;
    Ok(())
}

// --------------------------------------------------------------------------
// LLM Provider configuration persistence
// --------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSettings {
    pub chat: ProviderSetting,
    pub utility: ProviderSetting,
    pub embedding: ProviderSetting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSetting {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
}

impl Default for LlmSettings {
    fn default() -> Self {
        Self {
            chat: ProviderSetting {
                provider: "deepseek".to_string(),
                model: "deepseek-v4-flash".to_string(),
                api_key: None,
            },
            utility: ProviderSetting {
                provider: "deepseek".to_string(),
                model: "deepseek-v4-flash".to_string(),
                api_key: None,
            },
            embedding: ProviderSetting {
                provider: "deepseek".to_string(),
                model: "deepseek-v4-flash".to_string(),
                api_key: None,
            },
        }
    }
}

fn llm_settings_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let config_dir = app_handle
        .path()
        .app_config_dir()
        .expect("Failed to get app config dir");
    std::fs::create_dir_all(&config_dir).ok();
    config_dir.join("llm_settings.json")
}

/// Load LLM provider settings from the app config directory.
/// Returns default settings if no file exists yet.
#[tauri::command]
pub fn load_llm_settings(app_handle: tauri::AppHandle) -> Result<LlmSettings, String> {
    let path = llm_settings_path(&app_handle);
    if !path.exists() {
        return Ok(LlmSettings::default());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read LLM settings file: {}", e))?;
    let settings: LlmSettings = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse LLM settings file: {}", e))?;
    Ok(settings)
}

/// Save LLM provider settings to the app config directory.
#[tauri::command]
pub fn save_llm_settings(app_handle: tauri::AppHandle, settings: LlmSettings) -> Result<(), String> {
    let path = llm_settings_path(&app_handle);
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize LLM settings: {}", e))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write LLM settings file: {}", e))?;
    Ok(())
}



/// Internal helper: load LLM settings from file (no Tauri command wrapper).
/// Used by health_check module to avoid circular dependency.
pub fn load_llm_settings_internal(app_handle: &tauri::AppHandle) -> Result<LlmSettings, String> {
    let path = llm_settings_path(app_handle);
    if !path.exists() {
        return Ok(LlmSettings::default());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read LLM settings file: {}", e))?;
    let settings: LlmSettings = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse LLM settings file: {}", e))?;
    Ok(settings)
}

/// Test an LLM provider connection by making a real HTTP request.
/// Returns a JSON string with { success, latency_ms, error }.
/// C7: typed errors sanitized before IPC.
#[tauri::command]
pub async fn test_llm_connection(
    settings: ProviderSetting,
) -> Result<String, String> {
    let hc = crate::llm::health_check::HealthCheck::new(
        Box::new(crate::llm::health_check::ReqwestClient),
    );
    let result = hc
        .test_connection(
            &settings.provider,
            &settings.model,
            settings.api_key.as_deref(),
            None,
        )
        .await;

    match result {
        Ok(latency) => {
            let r = crate::llm::health_check::TestConnectionResult::success(latency);
            serde_json::to_string(&r).map_err(|e| format!("Serialization error: {}", e))
        }
        Err(e) => {
            let r = crate::llm::health_check::TestConnectionResult::failure(
                e.user_message(),
            );
            serde_json::to_string(&r).map_err(|e| format!("Serialization error: {}", e))
        }
    }
}

// --------------------------------------------------------------------------
// API Key Management (Windows Credential Manager integration)
// Uses Keychain from llm::keychain which delegates to the `windows` crate
// for Windows Credential Manager, with fallback chain.
// C10: sanitized strings only — no raw OS errors in IPC responses.
// --------------------------------------------------------------------------

/// Save an API key for a provider to the secure storage (Windows Credential Manager).
///
/// On Windows, persists to Credential Manager as `ShiroScout/{provider}`.
/// On non-Windows platforms (e.g. CI container), stores in app settings JSON.
#[tauri::command]
pub fn save_api_key(
    app_handle: tauri::AppHandle,
    provider: String,
    api_key: String,
) -> Result<(), String> {
    if provider.trim().is_empty() {
        return Err("Provider name must not be empty".to_string());
    }
    if api_key.is_empty() {
        return Err("API key must not be empty".to_string());
    }

    // Create keychain with settings file fallback path
    let settings_path = llm_settings_path(&app_handle);
    let mut keychain = Keychain::new(Some(settings_path));

    // set_key() writes to WinCred (primary) + in-memory cache
    keychain.set_key(&provider, api_key);

    Ok(())
}

/// Get an API key for a provider.
///
/// Priority chain:
/// 1. Windows Credential Manager
/// 2. In-memory cache (session lifetime)
/// 3. Environment variable (<PROVIDER>_API_KEY)
/// 4. Settings JSON file (legacy fallback)
#[tauri::command]
pub fn get_api_key(app_handle: tauri::AppHandle, provider: String) -> Result<String, String> {
    if provider.trim().is_empty() {
        return Err("Provider name must not be empty".to_string());
    }

    let settings_path = llm_settings_path(&app_handle);
    let mut keychain = Keychain::new(Some(settings_path));

    keychain
        .require_api_key(&provider)
        .map_err(|e| match e {
            crate::llm::keychain::KeychainError::MissingApiKey(p) => {
                format!("API key not found for provider '{}'", p)
            }
            crate::llm::keychain::KeychainError::CredentialManagerError(msg) => {
                format!("Failed to access credential store: {}", msg)
            }
            crate::llm::keychain::KeychainError::LoadFailed(msg) => {
                format!("Failed to load API key: {}", msg)
            }
        })
}

/// Delete an API key for a provider from the credential store.
#[tauri::command]
pub fn delete_api_key(app_handle: tauri::AppHandle, provider: String) -> Result<(), String> {
    if provider.trim().is_empty() {
        return Err("Provider name must not be empty".to_string());
    }

    let settings_path = llm_settings_path(&app_handle);
    let mut keychain = Keychain::new(Some(settings_path));

    keychain.delete_key(&provider).map_err(|e| format!("{}", e))
}
