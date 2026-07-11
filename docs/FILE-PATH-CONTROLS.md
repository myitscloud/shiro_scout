# File Path Controls

> Enterprise-grade secure path sandboxing and live terminal execution feedback for AI coding agents
> Source: Multi-tier path authorization, tokio-based stream multiplexing, and Docker/bollard isolation

---

To build an enterprise-grade sandbox for a coding agent, you need a multi-layered defensive strategy. Relying on the LLM to follow the instruction "Do not exit this folder" will inevitably fail if the agent faces an adversarial prompt, hallucinations, or complex code generation (like accidentally executing `rm -rf /` via a generated script). [1, 2]

Here is a practical, production-ready guide to achieving both **Path Sanity (Preventing Path Traversal)** and **Live Terminal Execution Feedback** in Rust, configured specifically for your workspace rules (Full Read/Write inside the project folder, Read-Only for one folder up containing notes/references, and Blocked everywhere else). [3]

---

## Part 1: Secure Multi-Tier Path Sandboxing

To enforce your custom folder rules, you need a mechanism that checks every single file path the AI requests before the filesystem executes it. We will use **camino** for clean UTF-8 paths, and standard path canonicalization to strip out tricky tricks like `../../etc/passwd`.

```rust
use camino::{Utf8Path, Utf8PathBuf};
use std::io::{Error, ErrorKind, Result};

pub struct AgentSandbox {
    /// Full Read/Write access (e.g., /home/user/projects/my-cloned-repo)
    pub workspace_dir: Utf8PathBuf,
    /// Read-Only reference access (e.g., /home/user/projects/)
    pub reference_dir: Utf8PathBuf,
}

impl AgentSandbox {
    pub fn new(workspace: &str) -> Result<Self> {
        let workspace_path = Utf8PathBuf::from(workspace).canonicalize_utf8()?;

        // "One folder up" logic for references
        let reference_path = workspace_path
            .parent()
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "Workspace has no parent directory"))?
            .to_path_buf();

        Ok(Self {
            workspace_dir: workspace_path,
            reference_dir: reference_path,
        })
    }

    /// Validates an incoming path requested by the LLM and determines operations allowed.
    pub fn authorize_path(&self, raw_requested_path: &str, require_write: bool) -> Result<Utf8PathBuf> {
        let base_path = Utf8Path::new(raw_requested_path);

        // 1. Resolve relative paths against the workspace root
        let absolute_path = if base_path.is_relative() {
            self.workspace_dir.join(base_path)
        } else {
            base_path.to_path_buf()
        };

        // 2. Canonicalize strictly resolves symlinks and removes segments like '.' and '..'
        // Note: If the file doesn't exist yet (creating a new file), canonicalize the parent folder instead.
        let canonical_path = if absolute_path.exists() {
            absolute_path.canonicalize_utf8()?
        } else {
            let parent = absolute_path.parent().ok_or_else(|| {
                Error::new(ErrorKind::NotFound, "Invalid target folder path structure")
            })?;
            let canonical_parent = parent.canonicalize_utf8()?;
            canonical_parent.join(absolute_path.file_name().unwrap_or(""))
        };

        // 3. Enforce the Hierarchical Security Policies
        if canonical_path.starts_with(&self.workspace_dir) {
            // Tier 1: Inside the main cloned project directory -> Full Read & Write allowed
            Ok(canonical_path)
        } else if canonical_path.starts_with(&self.reference_dir) {
            // Tier 2: Inside the parent directory -> Allowed ONLY if it's a read operation
            if require_write {
                return Err(Error::new(
                    ErrorKind::PermissionDenied,
                    "Access Denied: The parent directory is strict Read-Only for documentation, prompts, and notes.",
                ));
            }
            Ok(canonical_path)
        } else {
            // Tier 3: Anywhere else on the host architecture -> Immediate block
            Err(Error::new(
                ErrorKind::PermissionDenied,
                format!(
                    "Security Exception: Path traversal blocked. Action attempted outside of allowed sandboxes: {}",
                    canonical_path
                ),
            ))
        }
    }
}
```

---

## Part 2: Interactive Session & Live Tool Interception

If you use standard `std::process::Command`, execution logs clear out completely on termination. To feed real-time outputs back to the agent precisely like Agent Zero, you must handle stream multiplexing asynchronously via **tokio** and execute shell commands inside your validated workspace directory.

Below is an architecture showing how a tool wraps execution, captures terminal states (stdout + stderr), and pipes text straight back to the LLM agent.

```rust
use tokio::process::Command;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};

pub struct TerminalTool {
    sandbox: AgentSandbox,
}

impl TerminalTool {
    pub fn new(sandbox: AgentSandbox) -> Self {
        Self { sandbox }
    }

    /// Executes a shell script on behalf of the agent securely
    pub async fn execute_command(&self, command_str: &str) -> std::io::Result<String> {
        // Enforce that the execution runtime environment initializes inside the designated workspace path
        let execution_directory = &self.sandbox.workspace_dir;

        // Setup process. We combine stdout and stderr into the same stream so the LLM gets absolute chronological logs
        let mut child = Command::new("bash")
            .arg("-c")
            .arg(command_str)
            .current_dir(execution_directory)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let mut out_reader = BufReader::new(stdout).lines();
        let mut err_reader = BufReader::new(stderr).lines();

        let mut combined_output = String::new();

        // Asynchronously stream line-by-line out of the active terminal session
        loop {
            tokio::select! {
                res = out_reader.next_line() => {
                    if let Ok(Some(line)) = res {
                        // Print to terminal for human visibility
                        println!("[Agent Stdout]: {}", line);
                        combined_output.push_str(&format!("{}\n", line));
                    } else { break; }
                }
                res = err_reader.next_line() => {
                    if let Ok(Some(line)) = res {
                        // Catch runtime warnings, pip missing requirements, module errors
                        println!("[Agent Stderr]: {}", line);
                        combined_output.push_str(&format!("ERROR: {}\n", line));
                    }
                }
            }
        }

        let status = child.wait().await?;
        if !status.success() {
            combined_output.push_str(&format!("\nCommand exited with structural failure code: {}", status));
        }

        // Returns this combined block cleanly directly back into the LLM's prompt pipeline
        Ok(combined_output)
    }
}
```

---

## Part 3: Integrating with the AI Framework (rig-rs)

When hooking these features up to rig, your tools expose clear instructions to the model. Because your core code enforces safety via the `AgentSandbox`, you protect your underlying native OS regardless of how experimental or recursive the LLM's commands become.

```rust
// A hypothetical implementation of a Rig Tool
pub struct WriteFileTool {
    pub sandbox: AgentSandbox,
}

// Pseudocode demonstrating how the tool intercepts operations
async fn call_tool(tool: &WriteFileTool, raw_path: String, content: String) -> String {
    // Validate path before touching the local drive
    match tool.sandbox.authorize_path(&raw_path, true) {
        Ok(safe_path) => {
            match tokio::fs::write(&safe_path, content).await {
                Ok(_) => "File written successfully.".to_string(),
                Err(e) => format!("Failed to write file due to OS error: {}", e)
            }
        },
        Err(security_err) => {
            // This error text goes directly to the LLM context.
            // When it reads "Permission Denied: ...", it triggers the self-aware logic
            // telling it that it cannot go there, forcing it to pivot back inside the workspace folder.
            format!("Execution Blocked: {}", security_err)
        }
    }
}
```

---

## Pro-Tip for Ultimate Isolation

If you are planning to clone arbitrary external GitHub repositories, running the code using Python's host environment natively can still bypass path checks if the generated code utilizes OS commands directly (e.g., an LLM writing a python script containing `os.system("rm -rf /")`).

To completely lock it down while maintaining this exact structural design, take the `TerminalTool` code block above and alter the target execution vector from native bash to a **Docker container execute command via the bollard crate:**

```rust
// Instead of bash -c, your tools tell docker to run it inside an isolated bubble
Command::new("docker")
    .args(["exec", "-w", "/workspace", "agent_container_id", "bash", "-c", command_str]);
```

Would you like to explore how to set up the `portable-pty` crate for true state preservation (keeping environment variables and terminal states active across multiple distinct LLM tool invocations), or look deeper into implementing the Docker / bollard isolation loop?

---

## References

[1] [https://shanedeconinck.be](https://shanedeconinck.be/posts/docker-sandbox-coding-agents/)

[2] [https://medium.com](https://medium.com/aimonks/when-the-model-isnt-the-bottleneck-inside-openclaw-s-production-grade-agent-architecture-86acfacfb84c)

[3] [https://medium.com](https://medium.com/h7w/path-traversal-and-remediation-in-javascript-fbe8f4f95c26)
