//! Prompt templates for the Orchestrator Agent.
//! Loads system prompts from the agent's prompt files on disk,
//! and formats tool descriptions for inclusion in the system prompt.
use std::collections::HashMap;
use crate::error::{AgentError, AgentResult};

/// Stores and serves prompt templates.
#[derive(Debug)]
pub struct PromptStore {
    prompts: HashMap<String, String>,
}

impl Default for PromptStore {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptStore {
    pub fn new() -> Self {
        Self {
            prompts: HashMap::new(),
        }
    }

    pub fn load_from(&mut self, path: &str) -> Result<(), String> {
        let dir = std::path::Path::new(path);
        if !dir.exists() || !dir.is_dir() {
            return Err(format!("Prompt directory not found: {}", path));
        }

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    if let Some(name) = entry_path.file_stem() {
                        if let Some(name_str) = name.to_str() {
                            if let Ok(content) = std::fs::read_to_string(&entry_path) {
                                self.prompts.insert(name_str.to_string(), content);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.prompts.get(name).map(|s| s.as_str())
    }

    pub fn register(&mut self, name: &str, content: &str) {
        self.prompts.insert(name.to_string(), content.to_string());
    }

    pub fn tool_descriptions(&self, tools: &[(&str, &str)]) -> String {
        if tools.is_empty() {
            return "No tools available.".to_string();
        }
        tools.iter()
            .map(|(name, desc)| format!("- `{}`: {}", name, desc))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn tool_not_found(&self, tool_name: &str) -> String {
        format!(
            "The tool `{}` you tried to use is not registered. You may have misspelled the name.",
            tool_name,
        )
    }

    pub fn tool_error(&self, tool_name: &str, error: &str) -> String {
        format!(
            "The tool `{}` failed with the following error:\n{}\n\nProcess this error and decide whether to retry, use a different tool, or report the result to the user.",
            tool_name,
            error
        )
    }

    pub fn main_system_prompt(&self) -> AgentResult<String> {
        self.prompts.get("main")
            .map(|s| s.to_string())
            .or_else(|| Some("You are an AI assistant. Respond to the user's requests.".to_string()))
            .ok_or_else(|| AgentError::Prompt("Failed to build system prompt".into()))
    }

    pub fn specifics_prompt(&self) -> AgentResult<String> {
        self.prompts.get("specifics")
            .map(|s| s.to_string())
            .or_else(|| Some("You have access to various tools. Use them when appropriate.".to_string()))
            .ok_or_else(|| AgentError::Prompt("Failed to build specifics prompt".into()))
    }

    pub fn fw_intervention(&self) -> String {
        "[INTERVENTION] The user has interrupted the current flow. Pause and wait for their input.".to_string()
    }

    pub fn fw_repeat(&self) -> String {
        "[REPEAT] Your last response was identical to the previous one. Try a different approach.".to_string()
    }

    pub fn fw_error(&self) -> String {
        "[ERROR] A tool returned an error. Review the error and decide whether to retry, use a different tool, or report to the user.".to_string()
    }
}