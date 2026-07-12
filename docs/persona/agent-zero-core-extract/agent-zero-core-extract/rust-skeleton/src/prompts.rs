//! Prompt loading + lenient JSON parsing.
//!
//! Ports two A0 pieces:
//!   1. helpers/files.py parse_file(): read a prompt by name with an override
//!      chain (agents/<profile>/prompts/X shadows prompts/X) and {{var}} substitution.
//!   2. helpers/dirty_json.py + helpers/extract_tools.py: never let malformed
//!      model output crash the loop — repair it or report Repairable.

use crate::core::{AgentError, AgentResult, ToolRequest};
use include_dir::{include_dir, Dir};
use serde_json::Value;
use std::collections::HashMap;

/// Point these at the folders extracted in this package. Copy `prompts/` and
/// `agents/` next to this crate's Cargo.toml (or adjust the paths).
static PROMPTS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/prompts");
static AGENTS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/agents");

pub struct PromptStore {
    /// e.g. Some("developer") -> agents/developer/prompts/* shadows prompts/*
    pub profile: Option<String>,
}

impl PromptStore {
    pub fn new(profile: Option<String>) -> Self {
        Self { profile }
    }

    /// A0's agent.read_prompt("fw.error.md", error=...) equivalent.
    pub fn read(&self, name: &str, vars: &HashMap<&str, String>) -> AgentResult<String> {
        let raw = self
            .lookup(name)
            .ok_or_else(|| AgentError::Critical(format!("prompt not found: {name}")))?;
        Ok(substitute(raw, vars))
    }

    fn lookup(&self, name: &str) -> Option<&str> {
        // 1) profile override: agents/<profile>/prompts/<name>
        if let Some(p) = &self.profile {
            let path = format!("{p}/prompts/{name}");
            if let Some(f) = AGENTS.get_file(&path) {
                return f.contents_utf8();
            }
        }
        // 2) root prompts/<name>
        PROMPTS.get_file(name).and_then(|f| f.contents_utf8())
    }
}

/// {{variable}} substitution, same syntax as A0's markdown prompts.
/// (A0 also supports python-generated prompts (*.py) and nested includes —
/// add those only if you actually use them.)
fn substitute(template: &str, vars: &HashMap<&str, String>) -> String {
    let mut out = template.to_string();
    for (k, v) in vars {
        out = out.replace(&format!("{{{{{k}}}}}"), v);
    }
    out
}

#[macro_export]
macro_rules! vars {
    ($($k:expr => $v:expr),* $(,)?) => {{
        let mut m: std::collections::HashMap<&str, String> = std::collections::HashMap::new();
        $( m.insert($k, $v.to_string()); )*
        m
    }};
}

// ---------------------------------------------------------------------------
// Dirty JSON — the parse half of self-healing
// ---------------------------------------------------------------------------

/// extract_tools.extract_json_root_string(): find the outermost {...} block in
/// text that may be wrapped in prose or ```json fences.
pub fn extract_json_root(text: &str) -> Option<&str> {
    let start = text.find('{')?;
    let mut depth = 0usize;
    let mut in_str = false;
    let mut escape = false;
    for (i, c) in text[start..].char_indices() {
        if in_str {
            match c {
                _ if escape => escape = false,
                '\\' => escape = true,
                '"' => in_str = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => in_str = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&text[start..=start + i]);
                }
            }
            _ => {}
        }
    }
    // Unbalanced (model got cut off) — return the tail and let repair try.
    Some(&text[start..])
}

/// helpers/dirty_json.py, minimum viable port. Strategy ladder:
///   1. strict serde_json
///   2. strip markdown fences, retry
///   3. cheap repairs: trailing commas, unbalanced braces
///   4. give up -> caller injects prompts/fw.msg_misformat.md and loops
/// For full parity, port dirty_json.py's tokenizer (it also fixes single
/// quotes, unquoted keys, and truncated strings).
pub fn json_parse_dirty(text: &str) -> Option<Value> {
    if let Ok(v) = serde_json::from_str::<Value>(text) {
        return Some(v);
    }
    let cleaned = text
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    if let Ok(v) = serde_json::from_str::<Value>(cleaned) {
        return Some(v);
    }
    let mut repaired = cleaned.replace(",}", "}").replace(",]", "]");
    let open = repaired.matches('{').count();
    let close = repaired.matches('}').count();
    for _ in close..open {
        repaired.push('}');
    }
    serde_json::from_str::<Value>(&repaired).ok()
}

/// Full pipeline: raw model text -> ToolRequest, or None (=> misformat warning).
pub fn parse_tool_request(response: &str) -> Option<ToolRequest> {
    let root = extract_json_root(response)?;
    let value = json_parse_dirty(root)?;
    serde_json::from_value::<ToolRequest>(value).ok()
}
