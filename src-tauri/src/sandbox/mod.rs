/// Secure file path sandboxing for the orchestrator agent.
/// Ported from file-path-controls.md using the `camino` crate.
///
/// Three tiers of path authorization:
/// - Tier 1: Workspace directory — full read/write
/// - Tier 2: Reference directory — read-only
/// - Tier 3: Everything else — blocked
use camino::{Utf8Path, Utf8PathBuf};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthLevel {
    ReadWrite,
    ReadOnly,
    Denied,
}

#[derive(Debug, Clone)]
pub struct AgentSandbox {
    workspace_root: Utf8PathBuf,
    reference_root: Utf8PathBuf,
    canonical_workspace: Option<Utf8PathBuf>,
    allowed_extensions: HashSet<String>,
}

impl AgentSandbox {
    pub fn new(workspace_root: &str, reference_root: &str) -> Self {
        let ws = Utf8PathBuf::from(workspace_root);
        let ref_path = Utf8PathBuf::from(reference_root);
        Self {
            canonical_workspace: ws.canonicalize_utf8().ok(),
            workspace_root: ws,
            reference_root: ref_path,
            allowed_extensions: HashSet::new(),
        }
    }

    pub fn authorize_read(&self, path : &str) -> Result<Utf8PathBuf, String> {
        let path = Utf8Path::new(path);
        let canonical = path.canonicalize_utf8().map_err(|e| format!("Path canonicalization failed: {e}"))?;
        let auth = self.check_path(&canonical);
        match auth {
            AuthLevel::ReadWrite | AuthLevel::ReadOnly => {
                if !self.allowed_extensions.is_empty() {
                    if let Some(ext) = canonical.extension() {
                        if !self.allowed_extensions.contains(&ext.to_lowercase()) {
                            return Err(format!("Extension '.{}' is not allowed for reading", ext));
                        }
                    }
                }
                Ok(canonical)
            }
            AuthLevel::Denied => Err("Path outside the sandbox workspace".to_string())
        }
    }

    pub fn check_path(&self, canonical: &Utf8Path) -> AuthLevel {
        if let Some(ref ws) = self.canonical_workspace {
            if canonical.starts_with(ws) { return AuthLevel::ReadWrite; }
        }
        if canonical.as_str().starts_with(self.workspace_root.as_str()) { return AuthLevel::ReadWrite; }
        if canonical.as_str().starts_with(self.reference_root.as_str()) { return AuthLevel::ReadOnly; }
        AuthLevel::Denied
    }
}
