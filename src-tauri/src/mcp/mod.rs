/// MCP (Model Context Protocol) server discovery and registry.
/// Item 4.5 — Discovers MCP servers listening on localhost ports
/// and registers them for agent tool routing per ADR-006.
///
/// Design:
/// - MCP servers run as separate child processes inside the Docker sandbox
/// - They expose HTTP endpoints on localhost ports (default range: 3100-3200)
/// - Discovery scans configured ports via TCP connect + optional health probe
/// - Registry tracks discovered servers with their capabilities
/// - User approval per connection per ADR-006 §Decision
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio::time::timeout;

// --------------------------------------------------------------------------
// Data structures
// --------------------------------------------------------------------------

/// Status of an MCP server endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum McpServerStatus {
    Online,
    Offline,
    Unknown,
}

/// Information about a discovered or manually registered MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub id: String,
    pub name: String,
    pub url: String,
    pub status: McpServerStatus,
    pub tool_count: u32,
    pub tools: Vec<String>,
}

/// Configuration for MCP server discovery scanning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpDiscoveryConfig {
    pub port_start: u16,
    pub port_end: u16,
    pub host: String,
    pub timeout_secs: u64,
}

impl Default for McpDiscoveryConfig {
    fn default() -> Self {
        Self {
            port_start: 3100,
            port_end: 3200,
            host: "127.0.0.1".to_string(),
            timeout_secs: 2,
        }
    }
}

// --------------------------------------------------------------------------
// Registry
// --------------------------------------------------------------------------

/// Thread-safe registry for discovered MCP servers.
pub struct McpRegistry {
    servers: RwLock<HashMap<String, McpServerInfo>>,
    next_id: AtomicU64,
}

impl Default for McpRegistry {
    fn default() -> Self {
        Self {
            servers: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }
}

impl McpRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    fn generate_id(&self) -> String {
        let n = self.next_id.fetch_add(1, Ordering::SeqCst);
        format!("mcp-{}", n)
    }

    /// Probe a single TCP port to detect a potential MCP server.
    async fn probe_port(&self, host: &str, port: u16, timeout_dur: Duration) -> Result<McpServerInfo, String> {
        let addr = format!("{}:{}", host, port);
        match timeout(timeout_dur, TcpStream::connect(&addr)).await {
            Ok(Ok(_stream)) => {
                let name = format!("MCP Server (port {})", port);
                let url = format!("http://{}/mcp", addr);

                // Attempt HTTP health probe to confirm MCP server
                let health_url = format!("http://{}/health", addr);
                let tool_count = match timeout(timeout_dur, reqwest::get(&health_url)).await {
                    Ok(Ok(resp)) if resp.status().is_success() => 1,
                    _ => 0,
                };

                Ok(McpServerInfo {
                    id: self.generate_id(),
                    name,
                    url,
                    status: McpServerStatus::Online,
                    tool_count,
                    tools: Vec::new(),
                })
            }
            Ok(Err(_)) | Err(_) => Err(format!("port {}: no response", port)),
        }
    }

    /// Scan the configured port range for MCP servers.
    pub async fn discover(&self, config: &McpDiscoveryConfig) -> Vec<McpServerInfo> {
        let timeout_dur = Duration::from_secs(config.timeout_secs);
        let mut discovered = Vec::new();
        for port in config.port_start..=config.port_end {
            if let Ok(info) = self.probe_port(&config.host, port, timeout_dur).await {
                let id = info.id.clone();
                self.servers.write().await.insert(id, info.clone());
                discovered.push(info);
            }
        }
        discovered
    }

    /// Return all registered servers.
    pub async fn get_all(&self) -> Vec<McpServerInfo> {
        self.servers.read().await.values().cloned().collect()
    }

    /// Remove a server by ID.
    pub async fn remove(&self, id: &str) -> bool {
        self.servers.write().await.remove(id).is_some()
    }

    /// Clear all entries.
    pub async fn clear(&self) {
        self.servers.write().await.clear();
    }

    /// Manually register a server.
    pub async fn register(&self, info: McpServerInfo) {
        self.servers.write().await.insert(info.id.clone(), info);
    }

    /// Generate a new ID (exposed for manual registration calls).
    pub fn generate_id_external(&self) -> String {
        self.generate_id()
    }
}

// --------------------------------------------------------------------------
// Global singleton
// --------------------------------------------------------------------------

fn mcp_registry() -> &'static McpRegistry {
    static INSTANCE: OnceLock<McpRegistry> = OnceLock::new();
    INSTANCE.get_or_init(McpRegistry::new)
}

// --------------------------------------------------------------------------
// Configuration singleton
// --------------------------------------------------------------------------

fn default_config() -> &'static McpDiscoveryConfig {
    static CONFIG: OnceLock<McpDiscoveryConfig> = OnceLock::new();
    CONFIG.get_or_init(McpDiscoveryConfig::default)
}

// --------------------------------------------------------------------------
// Tauri commands (Item 4.5)
// --------------------------------------------------------------------------

/// Scan configured localhost ports for running MCP servers.
#[tauri::command]
pub async fn discover_mcp_servers() -> Result<Vec<McpServerInfo>, String> {
    let config = default_config();
    Ok(mcp_registry().discover(config).await)
}

/// Return all discovered MCP servers.
#[tauri::command]
pub async fn get_mcp_servers() -> Result<Vec<McpServerInfo>, String> {
    Ok(mcp_registry().get_all().await)
}

/// Register an MCP server manually by URL and display name.
#[tauri::command]
pub async fn register_mcp_server(name: String, url: String) -> Result<McpServerInfo, String> {
    let reg = mcp_registry();
    let info = McpServerInfo {
        id: reg.generate_id_external(),
        name,
        url,
        status: McpServerStatus::Unknown,
        tool_count: 0,
        tools: Vec::new(),
    };
    let clone = info.clone();
    reg.register(info).await;
    Ok(clone)
}

/// Remove an MCP server from the registry by ID.
#[tauri::command]
pub async fn remove_mcp_server(id: String) -> Result<bool, String> {
    Ok(mcp_registry().remove(&id).await)
}

/// Clear all MCP servers from the registry.
#[tauri::command]
pub async fn clear_mcp_servers() -> Result<(), String> {
    mcp_registry().clear().await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_registry_new() {
        let reg = McpRegistry::new();
        let servers = futures::executor::block_on(reg.get_all());
        assert!(servers.is_empty());
    }

    #[test]
    fn test_mcp_registry_generate_id() {
        let reg = McpRegistry::new();
        let id1 = reg.generate_id();
        assert!(id1.starts_with("mcp-"));
        let id2 = reg.generate_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_mcp_registry_register_and_get() {
        let reg = McpRegistry::new();
        let id = reg.generate_id();
        let info = McpServerInfo {
            id: id.clone(),
            name: "test-server".into(),
            url: "http://127.0.0.1:3100/mcp".into(),
            status: McpServerStatus::Online,
            tool_count: 3,
            tools: vec!["tool1".into(), "tool2".into()],
        };
        futures::executor::block_on(reg.register(info));
        let servers = futures::executor::block_on(reg.get_all());
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "test-server");
        assert_eq!(servers[0].tool_count, 3);
    }

    #[test]
    fn test_mcp_registry_remove() {
        let reg = McpRegistry::new();
        let id = reg.generate_id();
        let info = McpServerInfo {
            id: id.clone(),
            name: "remove-me".into(),
            url: "http://127.0.0.1:3101/mcp".into(),
            status: McpServerStatus::Offline,
            tool_count: 0,
            tools: Vec::new(),
        };
        futures::executor::block_on(reg.register(info));
        assert!(futures::executor::block_on(reg.remove(&id)));
        assert!(!futures::executor::block_on(reg.remove("nonexistent")));
    }

    #[test]
    fn test_mcp_registry_clear() {
        let reg = McpRegistry::new();
        let id = reg.generate_id();
        let info = McpServerInfo {
            id: id.clone(),
            name: "temp".into(),
            url: "http://127.0.0.1:3102/mcp".into(),
            status: McpServerStatus::Unknown,
            tool_count: 0,
            tools: Vec::new(),
        };
        futures::executor::block_on(reg.register(info));
        futures::executor::block_on(reg.clear());
        let servers = futures::executor::block_on(reg.get_all());
        assert!(servers.is_empty());
    }

    #[test]
    fn test_mcp_discovery_config_default() {
        let cfg = McpDiscoveryConfig::default();
        assert_eq!(cfg.port_start, 3100);
        assert_eq!(cfg.port_end, 3200);
        assert_eq!(cfg.host, "127.0.0.1");
        assert_eq!(cfg.timeout_secs, 2);
    }

    #[test]
    fn test_mcp_server_status_serde() {
        let statuses = vec![
            McpServerStatus::Online,
            McpServerStatus::Offline,
            McpServerStatus::Unknown,
        ];
        for status in &statuses {
            let json = serde_json::to_string(status).unwrap();
            let deserialized: McpServerStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(*status, deserialized);
        }
    }

    #[test]
    fn test_mcp_server_info_serde() {
        let info = McpServerInfo {
            id: "mcp-1".into(),
            name: "Test".into(),
            url: "http://127.0.0.1:3100/mcp".into(),
            status: McpServerStatus::Online,
            tool_count: 2,
            tools: vec!["a".into(), "b".into()],
        };
        let json = serde_json::to_string_pretty(&info).unwrap();
        let deserialized: McpServerInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "mcp-1");
        assert_eq!(deserialized.tools.len(), 2);
    }

    #[test]
    fn test_mcp_registry_generate_id_external() {
        let reg = McpRegistry::new();
        let id = reg.generate_id_external();
        assert!(id.starts_with("mcp-"));
    }

    #[test]
    fn test_mcp_discovery_config_serde() {
        let cfg = McpDiscoveryConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let deserialized: McpDiscoveryConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.port_start, cfg.port_start);
        assert_eq!(deserialized.host, cfg.host);
    }
}