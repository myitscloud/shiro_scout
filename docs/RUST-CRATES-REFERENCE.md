# Rust Crates Reference for Project Aegis

> Reference list of Rust crates for building AI coding agent capabilities, organized by role.
> Source: Team discussion 2026-07-08

## 1. Orchestration & Agent Frameworks

These crates provide the "brain" for your agent, handling the LLM context, chat loops, and tool calling integration.

| Crate | Description | Priority |
|-------|-------------|----------|
| [rig-rs (Rig)](https://github.com/0xPlaygrounds/rig) | Modular AI agent framework supporting multiple LLM providers (OpenAI, Anthropic, etc.). Has a dedicated "agent" abstraction for defining tools and conversational context. | **Top Recommendation** |
| [llm-chain](https://crates.io/crates/llm-agent) | "LangChain of Rust." Excellent for complex chains of prompts and local model integration. | Alternative to rig-rs for chain-heavy workflows |
| [ollama-rs](https://crates.io/crates/ollama-rs) | Essential binding for communicating with Ollama server (Llama 3, DeepSeek Coder, etc.) | Required for local LLM support |

## 2. Code Understanding & Parsing

For an agent to "read" code effectively, it needs to understand structure (ASTs) rather than just raw text.

| Crate | Description |
|-------|-------------|
| [tree-sitter](https://crates.io/crates/tree-sitter) | **Required.** Industry standard for parsing code into an Abstract Syntax Tree (AST). Enables agent to "see" function definitions, classes, and variables across any language (Rust, Python, JS, etc.). |
| [syn](https://crates.io/crates/syn) | Most powerful parser for Rust code analysis and editing. Creates a manipulatable AST of Rust source code. |
| [ropey](https://crates.io/crates/ropey) | Essential for handling large text files efficiently. Enables instant editing and slicing of 10,000-line files without memory spikes. |

## 3. File Editing, Searching & Diffing

These tools allow the agent to modify files safely and verify changes.

| Crate | Description |
|-------|-------------|
| [similar](https://crates.io/crates/similar) | **Gold standard** for computing diffs in Rust. Use for "before vs. after" views and verifying edits. |
| [ignore](https://crates.io/crates/ignore) | Better than walkdir for coding agents — respects `.gitignore` files. Prevents wasted tokens reading `node_modules` or `target` directories. |
| [grep-searcher](https://crates.io/crates/grep-searcher) | The library behind ripgrep. Fastest possible text search for finding function definitions in milliseconds. |

## 4. Command Execution & Sandboxing

"Extensive" coding implies running tests and scripts. Safety is critical.

| Crate | Description |
|-------|-------------|
| [portable-pty](https://crates.io/crates/portable-pty) | **Crucial.** Spawn a real, persistent pseudo-terminal (Bash, Zsh, or PowerShell). If the AI types `cd /app`, the PTY stays in `/app` for the next command, mimicking exactly how Agent Zero interacts with a terminal. |
| [duct](https://crates.io/crates/duct) | If you don't need a full PTY but want to chain heavy shell commands, build pipelines, and easily redirect stderr into stdout so the LLM can see its own compilation or installation failures. |
| [bollard](https://crates.io/crates/bollard) | **Highly Desirable.** Primary Docker client for Rust. Spin up Docker containers for sandboxed code execution. |
| [tokio](https://crates.io/crates/tokio) | Async runtime for concurrent network requests (LLM API), file I/O, and shell execution. |

## 5. Git & Version Control

| Crate | Description |
|-------|-------------|
| [git2](https://crates.io/crates/git2) | Safe, high-performance bindings to libgit2. Let the agent clone repos, create branches, commit changes, inspect git history. |

## 6. The "Eyes": Environment Sensing & Diagnostics

To make your AI self-aware on startup, it needs to instantly map its surroundings. These crates gather OS data, hardware specs, paths, and environment variables to inject into the AI's initial context block.

| Crate | Description |
|-------|-------------|
| [sysinfo](https://crates.io/crates/sysinfo) | **Required.** Comprehensive system metrics. On startup, tell the LLM: "You are running on Ubuntu 24.04, with 16GB RAM, and your current CPU usage is 12%." |
| [which](https://crates.io/crates/which) | Essential for tool verification. Before the agent tries to run a command, check if binaries like `python3`, `git`, or `docker` are actually installed, allowing the AI to flag missing dependencies immediately. |
| [platforms](https://crates.io/crates/platforms) | Provides compile-time and runtime target triples (e.g., `x86_64-unknown-linux-gnu`), helping the AI determine exactly what flavor of binary it needs to download or compile. |
| [current_platform](https://crates.io/crates/current_platform) | Lightweight alternative to `platforms` for runtime target triple detection. |
| [dotenvy](https://crates.io/crates/dotenvy) | Well-maintained fork of the original dotenv. Cleanly load API keys and local configuration profiles so the agent knows its own operational boundaries. |

## 7. The "Sensors": Log Catching & Error Streaming

For an AI to self-heal, its internal application logs and command line errors must be intercepted, structured, and fed back into the LLM's prompt window.

| Crate | Description |
|-------|-------------|
| [tracing](https://crates.io/crates/tracing) + [tracing-appender](https://crates.io/crates/tracing-appender) | **Industry standard** for async logging and diagnostics. Configure a custom Layer that simultaneously writes logs to screen and pushes them into an in-memory ring buffer. If the app crashes or throws an error, the AI can read that exact buffer to diagnose itself. |
| [miette](https://crates.io/crates/miette) | Beautiful, highly diagnostic error reporting with "help text" fields. Parse internal Rust errors into highly readable markdown format before handing them to the LLM. |

## 8. File-System & Path Architecture

An agent needs to safely navigate directories without accidentally breaking out of its workspace.

| Crate | Description |
|-------|-------------|
| [camino](https://crates.io/crates/camino) | Provides `Utf8Path` and `Utf8PathBuf`. Standard Rust paths are not guaranteed valid UTF-8, causing headaches when converting paths to strings for an LLM. Camino ensures all paths are clean, valid strings the AI can read and write without throwing unwraps. |
| [safe-path](https://crates.io/crates/safe-path) | Prevents directory traversal attacks. If the LLM tries `cat ../../../etc/passwd`, sanitize paths and ensure the AI stays strictly within its sandbox folder. |
| [path_clean](https://crates.io/crates/path_clean) | Alternative path sanitization — normalizes paths, removes traversal components, resolves symlinks safely. |

## Summary: The "Perfect" Stack

For a production-grade coding agent, the recommended stack:

| Capability | Recommended Crate |
|---|---|
| Agent Logic | rig-rs |
| Environment Sensing | sysinfo + which + platforms |
| Parsing | tree-sitter |
| File Search | ignore + grep-searcher |
| Diffing | similar |
| Terminal | portable-pty |
| Git | git2 |
| Logging | tracing + tracing-appender |
| Error Reporting | miette |
| Path Safety | camino + safe-path |
| File Editing | ropey |
| Sandbox | bollard |

## Boot Sequence Example

To replicate Agent Zero's boot sequence, your Rust main loop would look like this:

```rust
// 1. Gather "Self-Awareness" Context
let mut system = sysinfo::System::new_all();
system.refresh_all();
let python_installed = which::which("python3").is_ok();
let current_dir = camino::Utf8PathBuf::from_path_buf(std::env::current_dir()?).unwrap();

// 2. Format into a System Prompt
let system_awareness_prompt = format!(
    "You are an autonomous coding agent.\n\
     Current Directory: {}\n\
     OS: {}\n\
     Python3 Available: {}\n\
     If any command fails, inspect the stderr output and fix the environment yourself.",
    current_dir,
    sysinfo::System::name().unwrap_or_default(),
    python_installed
);

// 3. Pass this string into your rig-rs or LLM agent definition on boot.
```

## References

- [https://lib.rs](https://lib.rs/crates/rustdoc-llms)
- [https://docs.rs](https://docs.rs/llm/latest/llm/)
- [https://docs.rig.rs](https://docs.rig.rs/)
- [https://crates.io/crates/llm-agent](https://crates.io/crates/llm-agent)
- [https://crates.io/crates/tree-sitter](https://crates.io/crates/tree-sitter)
- [https://crates.io/crates/git2](https://crates.io/crates/git2)
- [https://crates.io/crates/bollard](https://crates.io/crates/bollard)
- [https://crates.io/crates/sysinfo](https://crates.io/crates/sysinfo)
- [https://crates.io/crates/tracing](https://crates.io/crates/tracing)
- [https://crates.io/crates/camino](https://crates.io/crates/camino)
- [https://crates.io/crates/portable-pty](https://crates.io/crates/portable-pty)
