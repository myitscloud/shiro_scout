/// API key management for LLM providers.
/// Loads keys from Windows Credential Manager (primary), with fallback
/// to environment variables then Tauri settings JSON file.
/// ADR-005: "API keys are stored in the OS keychain and proxied through Tauri host — DeepSeek keys must never enter the container."
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

use super::credential_manager::{CredentialError, CredentialManager};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// API key management errors.
/// C7 compliance: typed errors, sanitized before IPC.
#[derive(Debug, Clone)]
pub enum KeychainError {
    MissingApiKey(String),
    LoadFailed(String),
    CredentialManagerError(String),
}

impl std::fmt::Display for KeychainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeychainError::MissingApiKey(p) => write!(f, "Missing API key for provider: {}", p),
            KeychainError::LoadFailed(msg) => write!(f, "Keychain load failed: {}", msg),
            KeychainError::CredentialManagerError(msg) => {
                write!(f, "Credential Manager error: {}", msg)
            }
        }
    }
}

impl std::error::Error for KeychainError {}

impl From<CredentialError> for KeychainError {
    fn from(err: CredentialError) -> Self {
        KeychainError::CredentialManagerError(err.to_string())
    }
}

// ---------------------------------------------------------------------------
// KeychainConfig — controls backend selection
// ---------------------------------------------------------------------------

/// Controls which credential storage backends are used.
#[derive(Debug, Clone, Default)]
pub struct KeychainConfig {
    /// Use a mock credential manager (for tests / CI)
    pub use_mock: bool,
}

// ---------------------------------------------------------------------------
// Keychain
// ---------------------------------------------------------------------------

/// Manages API keys for LLM providers.
///
/// Priority chain (1 is highest):
/// 1. Windows Credential Manager (primary secure storage)
/// 2. In-memory cache (session lifetime)
/// 3. Environment variable (<PROVIDER>_API_KEY)
/// 4. Settings JSON file (legacy fallback — migration path to keychain)
pub struct Keychain {
    /// In-memory cache of loaded API keys
    keys: HashMap<String, String>,
    /// Path to the Tauri settings file (for fallback / migration)
    #[allow(dead_code)]
    settings_path: Option<PathBuf>,
    /// Configuration
    config: KeychainConfig,
}

impl Keychain {
    /// Create a new Keychain.
    pub fn new(settings_path: Option<PathBuf>) -> Self {
        Self {
            keys: HashMap::new(),
            settings_path,
            config: KeychainConfig::default(),
        }
    }

    /// Create a Keychain with custom configuration.
    pub fn with_config(settings_path: Option<PathBuf>, config: KeychainConfig) -> Self {
        Self {
            keys: HashMap::new(),
            settings_path,
            config,
        }
    }

    /// Create a Keychain without settings file fallback (env vars only).
    pub fn from_env() -> Self {
        Self::new(None)
    }

    /// Load an API key for the given provider.
    ///
    /// Priority:
    /// 1. Windows Credential Manager (if available)
    /// 2. In-memory cache (if previously loaded)
    /// 3. Environment variable (<PROVIDER>_API_KEY, e.g. DEEPSEEK_API_KEY)
    /// 4. Tauri settings JSON file (if settings_path is configured)
    pub fn load_api_key(&mut self, provider_name: &str) -> Option<String> {
        // 1. Try Windows Credential Manager (primary secure storage)
        if let Ok(key) = self.load_from_credential_manager(provider_name) {
            self.keys.insert(provider_name.to_string(), key.clone());
            return Some(key);
        }

        // 2. Check cache second (but don't double-check after WinCred)
        if let Some(key) = self.keys.get(provider_name) {
            return Some(key.clone());
        }

        // 3. Check environment variables
        let env_var = format!("{}_API_KEY", provider_name.to_uppercase());
        if let Ok(key) = env::var(&env_var) {
            if !key.is_empty() {
                self.keys.insert(provider_name.to_string(), key.clone());
                return Some(key);
            }
        }

        // 4. Also check DEEPSEEK_API_KEY for "deepseek" provider (backward compat)
        if provider_name == "deepseek" {
            if let Ok(key) = env::var("DEEPSEEK_API_KEY") {
                if !key.is_empty() {
                    self.keys.insert(provider_name.to_string(), key.clone());
                    return Some(key);
                }
            }
        }

        // 5. Fallback: try loading from Tauri settings file
        if let Some(ref settings_path) = self.settings_path {
            if let Ok(key) = self.load_from_settings(settings_path, provider_name) {
                self.keys.insert(provider_name.to_string(), key.clone());
                // Migrate: if found in settings file, copy to Credential Manager
                self.migrate_to_credential_manager(provider_name, &key);
                return Some(key);
            }
        }

        None
    }

    /// Get an API key or return a MissingApiKey error.
    pub fn require_api_key(&mut self, provider_name: &str) -> Result<String, KeychainError> {
        self.load_api_key(provider_name)
            .ok_or_else(|| KeychainError::MissingApiKey(provider_name.to_string()))
    }

    /// Manually set an API key for a provider (useful for UI entry).
    /// Persists to Windows Credential Manager immediately.
    pub fn set_key(&mut self, provider_name: &str, api_key: String) {
        self.keys.insert(provider_name.to_string(), api_key.clone());

        // Persist to Windows Credential Manager
        if let Err(e) = self.write_to_credential_manager(provider_name, &api_key) {
            // Log but don't fail — cache is sufficient for session lifetime
            eprintln!(
                "[keychain] Failed to persist key to Credential Manager: {}",
                e
            );
        }
    }

    /// Delete an API key for a provider.
    /// Removes from cache and Windows Credential Manager.
    pub fn delete_key(&mut self, provider_name: &str) -> Result<(), KeychainError> {
        self.keys.remove(provider_name);
        self.delete_from_credential_manager(provider_name)
    }

    /// Check if a key is currently loaded for the given provider.
    pub fn has_key(&self, provider_name: &str) -> bool {
        self.keys.contains_key(provider_name)
    }

    /// Clear all cached keys (does NOT clear Credential Manager).
    pub fn clear(&mut self) {
        self.keys.clear();
    }

    /// Remove a specific provider's cached key (does NOT clear Credential Manager).
    pub fn remove(&mut self, provider_name: &str) {
        self.keys.remove(provider_name);
    }

    /// List all providers that have keys loaded or available.
    pub fn list_providers(&self) -> Vec<String> {
        self.keys.keys().cloned().collect()
    }

    // -----------------------------------------------------------------------
    // Windows Credential Manager integration
    // -----------------------------------------------------------------------

    /// Try to load a key from Windows Credential Manager.
    fn load_from_credential_manager(&self, provider_name: &str) -> Result<String, CredentialError> {
        if self.config.use_mock {
            return Err(CredentialError::NotSupported);
        }
        CredentialManager::read(provider_name)
    }

    /// Write a key to Windows Credential Manager.
    fn write_to_credential_manager(
        &self,
        provider_name: &str,
        api_key: &str,
    ) -> Result<(), CredentialError> {
        if self.config.use_mock {
            return Err(CredentialError::NotSupported);
        }
        CredentialManager::write(provider_name, api_key)
    }

    /// Delete a key from Windows Credential Manager.
    fn delete_from_credential_manager(
        &self,
        provider_name: &str,
    ) -> Result<(), KeychainError> {
        if self.config.use_mock {
            return Ok(());
        }
        CredentialManager::delete(provider_name).map_err(|e| match e {
            CredentialError::NotFound(_) => KeychainError::MissingApiKey(provider_name.to_string()),
            other => KeychainError::CredentialManagerError(other.to_string()),
        })
    }

    /// Migrate a key from settings file to Credential Manager.
    /// This is best-effort; failure to migrate does not block the key from being used.
    fn migrate_to_credential_manager(&self, provider_name: &str, api_key: &str) {
        if self.config.use_mock {
            return;
        }
        // Only migrate if Credential Manager doesn't already have this provider
        if CredentialManager::exists(provider_name) {
            return;
        }
        if let Err(e) = CredentialManager::write(provider_name, api_key) {
            eprintln!(
                "[keychain] Migration: failed to write key to Credential Manager for provider '{}': {}",
                provider_name, e
            );
        }
    }

    // -----------------------------------------------------------------------
    // Settings file operations
    // -----------------------------------------------------------------------

    /// Load an API key from the Tauri settings JSON file.
    fn load_from_settings(
        &self,
        settings_path: &PathBuf,
        provider_name: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(settings_path)?;
        let settings: serde_json::Value = serde_json::from_str(&content)?;

        // Look for: settings.llm_providers.<provider_name>.api_key
        if let Some(key) = settings["llm_providers"][provider_name]["api_key"]
            .as_str()
            .filter(|s| !s.is_empty())
        {
            return Ok(key.to_string());
        }

        // Also check flat structure: settings.<provider_name>_api_key
        let flat_key = format!("{}_api_key", provider_name);
        if let Some(key) = settings[flat_key].as_str().filter(|s| !s.is_empty()) {
            return Ok(key.to_string());
        }

        Err("API key not found in settings".into())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a Keychain configured to use mock credential manager
    fn test_keychain() -> Keychain {
        Keychain::with_config(
            None,
            KeychainConfig {
                use_mock: true,
                ..Default::default()
            },
        )
    }

    #[test]
    fn test_env_var_mapping() {
        let _keychain = Keychain::from_env();
        assert_eq!(
            format!("{}_API_KEY", "deepseek".to_uppercase()),
            "DEEPSEEK_API_KEY"
        );
    }

    #[test]
    fn test_set_and_get() {
        let mut keychain = test_keychain();
        keychain.set_key("ollama", "ollama-key-123".into());
        assert_eq!(
            keychain.load_api_key("ollama").unwrap(),
            "ollama-key-123"
        );
        assert!(keychain.has_key("ollama"));
    }

    #[test]
    fn test_require_api_key_missing() {
        let mut keychain = test_keychain();
        let result = keychain.require_api_key("nonexistent");
        assert!(result.is_err());
        match result {
            Err(KeychainError::MissingApiKey(name)) => {
                assert_eq!(name, "nonexistent");
            }
            _ => panic!("Expected MissingApiKey error"),
        }
    }

    #[test]
    fn test_delete_key() {
        let mut keychain = test_keychain();
        keychain.set_key("test-provider", "test-key-1".into());
        assert!(keychain.has_key("test-provider"));

        keychain
            .delete_key("test-provider")
            .expect("delete_key should succeed");
        assert!(!keychain.has_key("test-provider"));
    }

    #[test]
    fn test_delete_nonexistent_key() {
        let mut keychain = test_keychain();
        let result = keychain.delete_key("nonexistent");
        // Should still succeed (cache remove is no-op, CM not found is fine)
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_providers() {
        let mut keychain = test_keychain();
        keychain.set_key("deepseek", "ds-key".into());
        keychain.set_key("openai", "oa-key".into());

        let providers = keychain.list_providers();
        assert!(providers.contains(&"deepseek".to_string()));
        assert!(providers.contains(&"openai".to_string()));
    }

    #[test]
    fn test_clear_cache() {
        let mut keychain = test_keychain();
        keychain.set_key("deepseek", "ds-key".into());
        assert!(keychain.has_key("deepseek"));

        keychain.clear();
        assert!(!keychain.has_key("deepseek"));
    }

    #[test]
    fn test_remove_specific() {
        let mut keychain = test_keychain();
        keychain.set_key("deepseek", "ds-key".into());
        keychain.set_key("openai", "oa-key".into());

        keychain.remove("deepseek");
        assert!(!keychain.has_key("deepseek"));
        assert!(keychain.has_key("openai"));
    }

    #[test]
    fn test_set_key_overwrite() {
        let mut keychain = test_keychain();
        keychain.set_key("deepseek", "key1".into());
        keychain.set_key("deepseek", "key2".into());
        assert_eq!(keychain.load_api_key("deepseek").unwrap(), "key2");
    }

    #[test]
    fn test_keychain_default_config() {
        let config = KeychainConfig::default();
        assert!(!config.use_mock);
    }
}
