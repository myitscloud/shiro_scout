/// Conversation history for the Orchestrator Agent.
/// Tracks user, assistant, and tool messages with timestamps.
/// Ported from mimicking-agent-zero/agent.rs History.
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History {
    pub messages: Vec<Message>,
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

impl History {
    pub fn new() -> Self {
        Self { messages: Vec::new() }
    }

    pub fn add_user(&mut self, msg: &str) {
        self.messages.push(Message {
            role: "user".into(),
            content: msg.to_string(),
            timestamp: Some(Utc::now().to_rfc3339()),
        });
    }

    pub fn add_assistant(&mut self, msg: &str) {
        self.messages.push(Message {
            role: "assistant".into(),
            content: msg.to_string(),
            timestamp: Some(Utc::now().to_rfc3339()),
        });
    }

    pub fn add_tool(&mut self, tool_name: &str, output: &str) {
        self.messages.push(Message {
            // Use 'assistant' role instead of 'tool' because DeepSeek API requires
            // tool_call_id for tool messages, and our simulated tool results don't
            // participate in a tool_call chain. Assistant role avoids the validation.
            role: "assistant".into(),
            content: format!("[Tool {}]\n{}", tool_name, output),
            timestamp: Some(Utc::now().to_rfc3339()),
        });
    }

    pub fn add_warning(&mut self, msg: &str) {
        self.messages.push(Message {
            role: "system".into(),
            content: format!("[Warning] {}", msg),
            timestamp: Some(Utc::now().to_rfc3339()),
        });
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn to_messages(&self) -> String {
        self.messages.iter().map(|m| {
            format!("{}: {}", m.role, m.content)
        }).collect::<Vec<_>>().join("\n")
    }

    pub fn summary(&self) -> String {
        format!("{} messages ({} user, {} assistant, {} tool, {} system)",
            self.messages.len(),
            self.messages.iter().filter(|m| m.role == "user").count(),
            self.messages.iter().filter(|m| m.role == "assistant").count(),
            self.messages.iter().filter(|m| m.role == "tool").count(),
            self.messages.iter().filter(|m| m.role == "system").count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_new_is_empty() {
        let h = History::new();
        assert!(h.messages.is_empty());
    }

    #[test]
    fn test_history_add_user() {
        let mut h = History::new();
        h.add_user("hello");
        assert_eq!(h.messages.len(), 1);
        assert_eq!(h.messages[0].role, "user");
        assert_eq!(h.messages[0].content, "hello");
    }

    #[test]
    fn test_history_add_assistant() {
        let mut h = History::new();
        h.add_assistant("world");
        assert_eq!(h.messages[0].role, "assistant");
        assert_eq!(h.messages[0].content, "world");
    }

    #[test]
    fn test_history_add_tool() {
        let mut h = History::new();
        h.add_tool("read_file", "contents");
        assert_eq!(h.messages[0].role, "tool");
        assert!(h.messages[0].content.contains("read_file"));
    }

    #[test]
    fn test_history_add_warning() {
        let mut h = History::new();
        h.add_warning("something went wrong");
        assert_eq!(h.messages[0].role, "warning");
    }

    #[test]
    fn test_history_clear() {
        let mut h = History::new();
        h.add_user("test");
        h.add_assistant("response");
        assert_eq!(h.messages.len(), 2);
        h.clear();
        assert!(h.messages.is_empty());
    }

    #[test]
    fn test_history_summary() {
        let mut h = History::new();
        h.add_user("msg1");
        h.add_assistant("msg2");
        h.add_tool("tool", "output");
        h.add_warning("warn");
        let s = h.summary();
        assert!(s.contains("4 messages"));
        assert!(s.contains("1 user"));
        assert!(s.contains("1 assistant"));
        assert!(s.contains("1 tool"));
        assert!(s.contains("1 warning"));
    }

    #[test]
    fn test_history_to_messages() {
        let mut h = History::new();
        h.add_user("hello");
        let output = h.to_messages();
        assert!(output.contains("user: hello"));
    }
}