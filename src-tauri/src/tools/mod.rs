/// Tool Registry for the Orchestrator Agent.
/// Defines the Tool trait and registers available tools.
/// Tool execution is routed through the ToolExecBridge for sandboxed containers.
/// Ported from AGENT_ROLES and ORCHESTRATOR-ARCHITECTURE.md.
use crate::bridge_client::ToolExecBridge;
use crate::error::{AgentError, AgentResult};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ToolType {
    Terminal,
    Read,
    Write,
    CodeAnalysis,
    Docker,
    Response,
    Search,
    DocumentQA,
}

#[derive(Debug)]
pub struct ToolRegistry {
    tools: HashMap<String, ToolType>,
    bridge: Option<ToolExecBridge>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut tools = HashMap::new();
        tools.insert("terminal".into(), ToolType::Terminal);
        tools.insert("read".into(), ToolType::Read);
        tools.insert("write".into(), ToolType::Write);
        tools.insert("code_analysis".into(), ToolType::CodeAnalysis);
        tools.insert("docker".into(), ToolType::Docker);
        tools.insert("response".into(), ToolType::Response);
        tools.insert("search".into(), ToolType::Search);
        tools.insert("document_qa".into(), ToolType::DocumentQA);
        Self { tools, bridge: None }
    }

    /// Attach a ToolExecBridge for sandboxed tool execution.
    pub fn with_bridge(mut self, bridge: ToolExecBridge) -> Self {
        self.bridge = Some(bridge);
        self
    }

    /// Access the bridge, if attached.
    pub fn bridge(&self) -> Option<&ToolExecBridge> {
        self.bridge.as_ref()
    }

    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    pub async fn execute(&self, name: &str, args: &serde_json::Value) -> AgentResult<String> {
        let tool = self.tools.get(name)
            .ok_or_else(|| AgentError::ToolNotFound(name.to_string()))?;
        match tool {
            ToolType::Terminal => self.exec_terminal(args).await,
            ToolType::Read => self.exec_read(args).await,
            ToolType::Write => self.exec_write(args).await,
            ToolType::CodeAnalysis => self.exec_code_analysis(args).await,
            ToolType::Docker => self.exec_docker(args).await,
            ToolType::Response => Ok("Response tool delivered to UI for streaming".into()),
            ToolType::Search => self.exec_search(args).await,
            ToolType::DocumentQA => self.exec_document_qa(args).await,
        }
    }

    async fn exec_terminal(&self, args: &serde_json::Value) -> AgentResult<String> {
        let command = args["command"].as_str()
            .ok_or_else(|| AgentError::Repairable("Missing 'command' argument".into()))?;

        // If bridge is available, run in sandbox container
        if let Some(ref bridge) = self.bridge {
            let result = bridge.execute(command)
                .await
                .map_err(|e| AgentError::Repairable(format!("Bridge exec failed: {}", e)))?;
            if result.success {
                Ok(result.output.trim().to_string())
            } else {
                Err(AgentError::Repairable(format!(
                    "Exit code {}: {}",
                    result.exit_code,
                    if result.stderr.is_empty() { &result.output } else { &result.stderr }
                )))
            }
        } else {
            // Fallback: local execution (container or development mode)
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()
                .map_err(|e| AgentError::Repairable(format!("Command failed: {e}")))?;
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                Err(AgentError::Repairable(format!(
                    "Exit code {}: {}",
                    output.status.code().unwrap_or(-1),
                    String::from_utf8_lossy(&output.stderr).trim()
                )))
            }
        }
    }

    async fn exec_docker(&self, args: &serde_json::Value) -> AgentResult<String> {
        let command = args["command"].as_str()
            .ok_or_else(|| AgentError::Repairable("Missing 'command' argument".into()))?;

        // Docker tool always uses the bridge if available
        if let Some(ref bridge) = self.bridge {
            let result = bridge.execute(command)
                .await
                .map_err(|e| AgentError::Repairable(format!("Bridge exec failed: {}", e)))?;
            Ok(serde_json::json!({
                "success": result.success,
                "output": result.output,
                "stderr": result.stderr,
                "exit_code": result.exit_code,
            }).to_string())
        } else {
            Err(AgentError::Repairable("No bridge available for Docker tool".into()))
        }
    }

    async fn exec_read(&self, args: &serde_json::Value) -> AgentResult<String> {
        Ok(format!("Read tool: {}", args))
    }

    async fn exec_write(&self, args: &serde_json::Value) -> AgentResult<String> {
        Ok(format!("Write tool: {}", args))
    }

    async fn exec_code_analysis(&self, args: &serde_json::Value) -> AgentResult<String> {
        Ok(format!("Code Analysis tool: {}", args))
    }

    async fn exec_search(&self, args: &serde_json::Value) -> AgentResult<String> {
        Ok(format!("Search tool: {}", args))
    }

    async fn exec_document_qa(&self, args: &serde_json::Value) -> AgentResult<String> {
        Ok(format!("Document QA tool: {}", args))
    }
}
