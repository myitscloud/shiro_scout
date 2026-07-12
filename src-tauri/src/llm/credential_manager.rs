//! Windows Credential Manager integration using the `windows` crate.
//! Wraps CredWriteW / CredReadW / CredDeleteW for secure API key storage.
//! ADR-005 mandates: "API keys are stored in the OS keychain and proxied through Tauri host."
//!
//! Credential target format: `Windows Credential` type named `ShiroScout/{provider_name}`
use serde::Serialize;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Constants (Win32 API values, always available)
// ---------------------------------------------------------------------------

const CRED_TYPE_GENERIC: u32 = 1;
const CRED_PERSIST_LOCAL_MACHINE: u32 = 2;
const ERROR_NOT_FOUND: i32 = 1168;

// ---------------------------------------------------------------------------
// Platform-specific imports
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
use windows::core::PWSTR;

#[cfg(target_os = "windows")]
use windows::Win32::Security::Credentials::{
    CredDeleteW, CredFree, CredReadW, CredWriteW, CREDENTIALW,
};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors from Windows Credential Manager operations.
/// C7 compliance: typed errors, sanitized before IPC.
#[derive(Debug, Clone, Serialize)]
pub enum CredentialError {
    /// Win32 API call failed (sanitized message, no raw OS error text)
    Win32(String),
    /// Credential not found for the given provider
    NotFound(String),
    /// Invalid input (empty provider name or key)
    InvalidInput(String),
    /// Credential Manager not supported on this platform
    NotSupported,
}

impl std::fmt::Display for CredentialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CredentialError::Win32(msg) => write!(f, "Credential Manager error: {}", msg),
            CredentialError::NotFound(name) => {
                write!(f, "Credential not found for provider: {}", name)
            }
            CredentialError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            CredentialError::NotSupported => {
                write!(f, "Credential Manager not supported on this platform")
            }
        }
    }
}

impl std::error::Error for CredentialError {}

// ---------------------------------------------------------------------------
// CredentialManager
// ---------------------------------------------------------------------------

/// Windows Credential Manager wrapper.
/// On non-Windows platforms, `new()` returns `None` and operations fail with
/// `NotSupported`. This allows the codebase to compile uniformly while gracefully
/// degrading when not running on Windows (e.g., in CI container).
pub struct CredentialManager;

impl CredentialManager {
    /// Create a new credential manager.
    /// Returns `Some` on Windows, `None` on other platforms.
    pub fn new() -> Option<Self> {
        if cfg!(target_os = "windows") {
            Some(Self)
        } else {
            None
        }
    }

    /// Write an API key to Windows Credential Manager.
    ///
    /// Credential type: Generic (`CRED_TYPE_GENERIC` = 1)
    /// Persistence: Local machine (`CRED_PERSIST_LOCAL_MACHINE` = 2)
    /// Target name: `ShiroScout/{provider}`
    pub fn write(provider: &str, api_key: &str) -> Result<(), CredentialError> {
        if !cfg!(target_os = "windows") {
            return Err(CredentialError::NotSupported);
        }
        #[cfg(target_os = "windows")]
        return Self::write_impl(provider, api_key);

        #[cfg(not(target_os = "windows"))]
        unreachable!()
    }

    /// Read an API key from Windows Credential Manager.
    pub fn read(provider: &str) -> Result<String, CredentialError> {
        if !cfg!(target_os = "windows") {
            return Err(CredentialError::NotSupported);
        }
        #[cfg(target_os = "windows")]
        return Self::read_impl(provider);

        #[cfg(not(target_os = "windows"))]
        unreachable!()
    }

    /// Delete an API key from Windows Credential Manager.
    pub fn delete(provider: &str) -> Result<(), CredentialError> {
        if !cfg!(target_os = "windows") {
            return Err(CredentialError::NotSupported);
        }
        #[cfg(target_os = "windows")]
        return Self::delete_impl(provider, true);

        #[cfg(not(target_os = "windows"))]
        unreachable!()
    }

    /// Check if a credential exists for the given provider.
    pub fn exists(provider: &str) -> bool {
        Self::read(provider).is_ok()
    }

    /// Build the credential target name: `ShiroScout/{provider}`
    fn credential_name(provider: &str) -> String {
        format!("ShiroScout/{}", provider)
    }

    /// Validate provider and key input.
    fn validate_input(provider: &str, api_key: &str) -> Result<(), CredentialError> {
        if provider.is_empty() {
            return Err(CredentialError::InvalidInput(
                "Provider name must not be empty".into(),
            ));
        }
        if api_key.is_empty() {
            return Err(CredentialError::InvalidInput(
                "API key must not be empty".into(),
            ));
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Windows implementation
    // -----------------------------------------------------------------------

    #[cfg(target_os = "windows")]
    fn write_impl(provider: &str, api_key: &str) -> Result<(), CredentialError> {
        Self::validate_input(provider, api_key)?;

        let target_name = Self::credential_name(provider);
        let target_wide: Vec<u16> =
            target_name.encode_utf16().chain(std::iter::once(0)).collect();
        let key_bytes = api_key.as_bytes();

        let cred = CREDENTIALW {
            Type: windows::Win32::Security::Credentials::CRED_TYPE(CRED_TYPE_GENERIC),
            TargetName: PWSTR::from_raw(target_wide.as_ptr() as *mut u16),
            CredentialBlobSize: key_bytes.len() as u32,
            CredentialBlob: key_bytes.as_ptr() as *mut u8,
            Persist: windows::Win32::Security::Credentials::CRED_PERSIST(CRED_PERSIST_LOCAL_MACHINE),
            UserName: PWSTR::from_raw(std::ptr::null_mut()),
            ..Default::default()
        };

        unsafe {
            if CredWriteW(&cred, 0).is_ok() {
                Ok(())
            } else {
                let err = std::io::Error::last_os_error();
                Err(CredentialError::Win32(format!(
                    "Failed to write credential: {}",
                    Self::sanitize_os_error(&err)
                )))
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn read_impl(provider: &str) -> Result<String, CredentialError> {
        if provider.is_empty() {
            return Err(CredentialError::InvalidInput(
                "Provider name must not be empty".into(),
            ));
        }

        let target_name = Self::credential_name(provider);
        let target_wide: Vec<u16> =
            target_name.encode_utf16().chain(std::iter::once(0)).collect();

        unsafe {
            let mut pcred: *mut CREDENTIALW = std::ptr::null_mut();
            let result = CredReadW(
                PWSTR::from_raw(target_wide.as_ptr() as *mut u16),
                windows::Win32::Security::Credentials::CRED_TYPE(CRED_TYPE_GENERIC),
                Some(0),
                &mut pcred,
            );

            if result.is_ok() {
                let cred = &*pcred;
                let blob = std::slice::from_raw_parts(
                    cred.CredentialBlob as *const u8,
                    cred.CredentialBlobSize as usize,
                );
                let key = String::from_utf8_lossy(blob).to_string();
                CredFree(pcred as *mut _);
                Ok(key)
            } else {
                let err = std::io::Error::last_os_error();
                match err.raw_os_error() {
                    Some(code) if code == ERROR_NOT_FOUND => {
                        Err(CredentialError::NotFound(provider.to_string()))
                    }
                    _ => Err(CredentialError::Win32(format!(
                        "Failed to read credential: {}",
                        Self::sanitize_os_error(&err)
                    ))),
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn delete_impl(provider: &str, confirm: bool) -> Result<(), CredentialError> {
        if !confirm {
            return Err(CredentialError::InvalidInput(
                "delete requires explicit confirmation".into(),
            ));
        }
        if provider.is_empty() {
            return Err(CredentialError::InvalidInput(
                "Provider name must not be empty".into(),
            ));
        }

        let target_name = Self::credential_name(provider);
        let target_wide: Vec<u16> =
            target_name.encode_utf16().chain(std::iter::once(0)).collect();

        unsafe {
            let result = CredDeleteW(
                PWSTR::from_raw(target_wide.as_ptr() as *mut u16),
                windows::Win32::Security::Credentials::CRED_TYPE(CRED_TYPE_GENERIC),
                Some(0),
            );

            if result.is_ok() {
                Ok(())
            } else {
                let err = std::io::Error::last_os_error();
                match err.raw_os_error() {
                    Some(code) if code == ERROR_NOT_FOUND => {
                        Err(CredentialError::NotFound(provider.to_string()))
                    }
                    _ => Err(CredentialError::Win32(format!(
                        "Failed to delete credential: {}",
                        Self::sanitize_os_error(&err)
                    ))),
                }
            }
        }
    }

    /// Sanitize OS error to remove paths/USER-PC/internal details (C10).
    /// Returns a human-safe string fragment.
    fn sanitize_os_error(err: &std::io::Error) -> String {
        let _msg = err.to_string();
        // Extract just the error code and short description
        if let Some(code) = err.raw_os_error() {
            format!("OS error code {}", code)
        } else {
            // Fallback: generic category
            format!("OS error: {}", err.kind())
        }
    }
}

// ---------------------------------------------------------------------------
// MockCredentialManager — for tests and CI environments
// ---------------------------------------------------------------------------

/// In-memory mock credential store for testing.
/// Mirrors the CredentialManager interface but operates on a local HashMap.
pub struct MockCredentialManager {
    store: HashMap<String, String>,
}

impl MockCredentialManager {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    pub fn write(&mut self, provider: &str, api_key: &str) -> Result<(), CredentialError> {
        if provider.is_empty() || api_key.is_empty() {
            return Err(CredentialError::InvalidInput(
                "Provider name and API key must not be empty".into(),
            ));
        }
        self.store
            .insert(provider.to_string(), api_key.to_string());
        Ok(())
    }

    pub fn read(&self, provider: &str) -> Result<String, CredentialError> {
        self.store
            .get(provider)
            .cloned()
            .ok_or_else(|| CredentialError::NotFound(provider.to_string()))
    }

    pub fn delete(&mut self, provider: &str) -> Result<(), CredentialError> {
        if self.store.remove(provider).is_some() {
            Ok(())
        } else {
            Err(CredentialError::NotFound(provider.to_string()))
        }
    }

    pub fn exists(&self, provider: &str) -> bool {
        self.store.contains_key(provider)
    }
}

impl Default for MockCredentialManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_name_format() {
        assert_eq!(
            CredentialManager::credential_name("deepseek"),
            "ShiroScout/deepseek"
        );
        assert_eq!(
            CredentialManager::credential_name("openai"),
            "ShiroScout/openai"
        );
    }

    #[test]
    fn test_new_returns_none_on_non_windows() {
        // On Windows, this returns Some; on other platforms, None.
        // We just verify it doesn't panic and the Option is consistent.
        let cm = CredentialManager::new();
        if cfg!(target_os = "windows") {
            assert!(cm.is_some());
        } else {
            assert!(cm.is_none());
        }
    }

    #[test]
    fn test_write_empty_input() {
        let result = CredentialManager::write("", "key123");
        assert!(result.is_err());
        match result {
            Err(CredentialError::InvalidInput(_)) => {}
            _ => panic!("Expected InvalidInput error"),
        }

        let result = CredentialManager::write("deepseek", "");
        assert!(result.is_err());
        match result {
            Err(CredentialError::InvalidInput(_)) => {}
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_mock_write_and_read() {
        let mut mock = MockCredentialManager::new();
        assert!(!mock.exists("deepseek"));

        mock.write("deepseek", "sk-abc123")
            .expect("Mock write should succeed");
        assert!(mock.exists("deepseek"));
        assert_eq!(mock.read("deepseek").unwrap(), "sk-abc123");
    }

    #[test]
    fn test_mock_read_not_found() {
        let mock = MockCredentialManager::new();
        let result = mock.read("nonexistent");
        match result {
            Err(CredentialError::NotFound(name)) => {
                assert_eq!(name, "nonexistent");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_mock_delete() {
        let mut mock = MockCredentialManager::new();
        mock.write("deepseek", "sk-xyz")
            .expect("Mock write should succeed");
        assert!(mock.exists("deepseek"));

        mock.delete("deepseek").expect("Mock delete should succeed");
        assert!(!mock.exists("deepseek"));
    }

    #[test]
    fn test_mock_delete_not_found() {
        let mut mock = MockCredentialManager::new();
        let result = mock.delete("nonexistent");
        match result {
            Err(CredentialError::NotFound(name)) => {
                assert_eq!(name, "nonexistent");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_mock_overwrite() {
        let mut mock = MockCredentialManager::new();
        mock.write("deepseek", "key1").unwrap();
        mock.write("deepseek", "key2").unwrap();
        assert_eq!(mock.read("deepseek").unwrap(), "key2");
    }

    #[test]
    fn test_mock_multiple_providers() {
        let mut mock = MockCredentialManager::new();
        mock.write("deepseek", "ds-key").unwrap();
        mock.write("openai", "oa-key").unwrap();
        mock.write("groq", "gr-key").unwrap();

        assert_eq!(mock.read("deepseek").unwrap(), "ds-key");
        assert_eq!(mock.read("openai").unwrap(), "oa-key");
        assert_eq!(mock.read("groq").unwrap(), "gr-key");
    }
}
