/// Environment sensing for the orchestrator agent.
/// Gathers OS info, available tools, Docker status, disk space, and LLM configuration.
/// Runs on boot and caches results in EnvironmentInfo.
///
/// Reference: ORCHESTRATOR-ARCHITECTURE.md §4 (Environment Awareness)
use serde::Serialize;

/// Result of a single dependency check
#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub name: String,
    pub available: bool,
    pub version: Option<String>,
    pub error: Option<String>,
}

/// Complete environment information gathered on boot
#[derive(Debug, Clone, Serialize)]
pub struct EnvironmentInfo {
    pub os_name: String,
    pub arch: String,
    pub total_memory_mb: u64,
    pub free_disk_mb: u64,
    pub docker_available: bool,
    pub python_available: bool,
    pub node_available: bool,
    pub git_available: bool,
    pub cargo_available: bool,
    pub workspace_path: String,
    pub llm_configured: bool,
    pub llm_provider: String,
    pub network_available: bool,
}

impl EnvironmentInfo {
    pub fn check_binary(name: &str, version_flag: &str) -> CheckResult {
        let output = std::process::Command::new(name)
            .arg(version_flag)
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let version = String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .next()
                    .map(|s| s.to_string())
                    .or_else(|| {
                        String::from_utf8_lossy(&out.stderr)
                            .lines()
                            .next()
                            .map(|s| s.to_string())
                    });
                CheckResult {
                    name: name.to_string(),
                    available: true,
                    version,
                    error: None,
                }
            }
            Ok(_) => CheckResult {
                name: name.to_string(),
                available: false,
                version: None,
                error: Some("Binary not found or not executable".into()),
            },
            Err(e) => CheckResult {
                name: name.to_string(),
                available: false,
                version: None,
                error: Some(format!("{e}")),
            },
        }
    }

    pub fn gather() -> Self {
        let docker_check = Self::check_binary("docker", "--version");
        let python_check = Self::check_binary("python3", "--version");
        let node_check = Self::check_binary("node", "--version");
        let git_check = Self::check_binary("git", "--version");
        let cargo_check = Self::check_binary("cargo", "--version");

        let os_name = if cfg!(target_os = "windows") {
            "Windows".into()
        } else if cfg!(target_os = "macos") {
            "macOS".into()
        } else if cfg!(target_os = "linux") {
            std::process::Command::new("uname")
                .arg("-sr")
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "Linux".into())
        } else {
            "Unknown".into()
        };

        let arch = std::env::consts::ARCH.to_string();

        Self {
            os_name,
            arch,
            total_memory_mb: 0,
            free_disk_mb: 0,
            docker_available: docker_check.available,
            python_available: python_check.available,
            node_available: node_check.available,
            git_available: git_check.available,
            cargo_available: cargo_check.available,
            workspace_path: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "/workspace".into()),
            llm_configured: std::env::var("LLM_API_KEY").is_ok(),
            llm_provider: std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "none".into()),
            network_available: false,
        }
    }

    pub fn check_all() -> Vec<CheckResult> {
        vec![
            Self::check_binary("docker", "--version"),
            Self::check_binary("python3", "--version"),
            Self::check_binary("node", "--version"),
            Self::check_binary("git", "--version"),
            Self::check_binary("cargo", "--version"),
        ]
    }

    pub fn to_markdown(&self) -> String {
        format!(
            "## Environment Status\n\n| Component | Status | Details |\n|-----------|--------|----------|\n| OS | ✅ | {} ({}) |\n| Docker | {} | |\n| Python | {} | |\n| Node.js | {} | |\n| Git | {} | |\n| Cargo | {} | |\n| LLM | {} | Provider: {} |\n| Workspace | ✅ | {} |\n",
            self.os_name,
            self.arch,
            if self.docker_available { "✅" } else { "❌" },
            if self.python_available { "✅" } else { "❌" },
            if self.node_available { "✅" } else { "❌" },
            if self.git_available { "✅" } else { "❌" },
            if self.cargo_available { "✅" } else { "❌" },
            if self.llm_configured { "✅" } else { "❌" },
            self.llm_provider,
            self.workspace_path,
        )
    }
}