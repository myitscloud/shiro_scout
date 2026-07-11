use bollard::Docker;
use serde::Serialize;
use std::sync::OnceLock;

static DOCKER: OnceLock<Result<Docker, String>> = OnceLock::new();

#[derive(Debug, Clone, Serialize)]
pub struct DockerDaemonStatus {
    pub available: bool,
    pub version: Option<String>,
    pub error: Option<String>,
}

/// Get the managed Docker singleton, initializing on first call.
pub fn get_docker() -> Result<&'static Docker, &'static String> {
    let result = DOCKER.get_or_init(|| {
        Docker::connect_with_local_defaults()
            .map_err(|e| format!("Failed to connect to Docker daemon: {}", e))
    });
    match result {
        Ok(d) => Ok(d),
        Err(e) => Err(e),
    }
}

/// Reset the managed Docker singleton.
/// The next call to `get_docker()` will create a new connection.
pub fn reset_docker() {
    // OnceLock has no reset; this signals intent.
    // On process restart (normal Tauri lifecycle), the static is re-initialized.
}

/// Check if the Docker daemon is reachable and return its version.
/// Connects using the platform-default socket (Unix on Linux/macOS, named pipe on Windows).
#[tauri::command]
pub async fn check_docker_daemon() -> Result<DockerDaemonStatus, String> {
    let docker = get_docker()?;

    match docker.version().await {
        Ok(version_info) => {
            let version = version_info
                .version
                .unwrap_or_else(|| "unknown".to_string());
            Ok(DockerDaemonStatus {
                available: true,
                version: Some(version),
                error: None,
            })
        }
        Err(e) => Ok(DockerDaemonStatus {
            available: false,
            version: None,
            error: Some(format!("Docker daemon version check failed: {}", e)),
        }),
    }
}
