# True State Preservation

> Persistent pseudo-terminal (PTY) shell state preservation with `portable-pty` and containerized isolation with `bollard`
> Source: Multi-step shell sessions, environment variables, and Python venv state across LLM tool invocations

---

To preserve state (like changing directories with `cd`, setting environment variables, or activating a Python virtual environment) across multiple separate LLM tool invocations, standard `std::process::Command` won't work because it spawns a fresh shell process every time and destroys it immediately after completion.

To fix this, we can use the **portable-pty** crate. It allows you to open a persistent pseudo-terminal (PTY) master/slave pair. The shell process stays alive continuously in the background, and your AI agent interacts with it by writing to its standard input (stdin) and reading from its standard output (stdout) asynchronously.

---

## Step 1: True State Preservation with portable-pty

This implementation creates a persistent background shell session wrapped in a thread-safe container. The AI can run `cd my-cloned-repo`, followed by `pip install -r requirements.txt`, followed by `python main.py`, and the shell will perfectly remember its state between each independent LLM tool call.

Add these dependencies to your `Cargo.toml`:

```toml
[dependencies]
portable-pty = "0.8"
tokio = { version = "1.0", features = ["full"] }
parking_lot = "0.12"
```

Here is the implementation of a persistent shell manager:

```rust
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::sync::Arc;
use std::thread;
use parking_lot::Mutex;

#[derive(Clone)]
pub struct PersistentShell {
    // We use parking_lot::Mutex for safe, fast synchronous access across async tasks
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    output_buffer: Arc<Mutex<String>>,
}

impl PersistentShell {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let pty_system = NativePtySystem::default();

        // Open a pseudo-terminal with standard sizing
        let pair = pty_system.open_pty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // Spawn your host shell inside the PTY (bash on Linux/macOS, powershell/cmd on Windows)
        #[cfg(target_os = "windows")]
        let cmd = CommandBuilder::new("powershell.exe");
        #[cfg(not(target_os = "windows"))]
        let cmd = CommandBuilder::new("bash");

        let mut child = pair.slave.spawn_command(cmd)?;

        // Drop the slave side in this thread so EOF can be detected properly later
        drop(pair.slave);

        let writer = pair.master.take_writer()?;
        let mut reader = pair.master.try_clone_reader()?;

        let output_buffer = Arc::new(Mutex::new(String::new()));
        let output_buffer_clone = output_buffer.clone();

        // Spawn a dedicated background thread to constantly read streaming terminal bytes
        thread::spawn(move || {
            let mut buf = [0u32; 1024]; // Using u32 buffer or raw u8 bytes depending on your stream
            let mut byte_buf = [0u8; 4096];

            while let Ok(n) = reader.read(&mut byte_buf) {
                if n == 0 { break; } // Shell exited

                // Convert raw terminal bytes (including ANSI escape codes) into a string
                if let Ok(text) = std::str::from_utf8(&byte_buf[..n]) {
                    // Print to host console so the human operator can watch the agent work live
                    print!("{}", text);
                    std::io::stdout().flush().unwrap();

                    // Append raw data to our shared memory ring-buffer
                    output_buffer_clone.lock().push_str(text);
                }
            }
        });

        Ok(Self {
            writer: Arc::new(Mutex::new(writer)),
            output_buffer,
        })
    }

    /// Sends a command execution request to the live background shell
    pub async fn execute(&self, command: &str) -> Result<String, Box<dyn std::error::Error>> {
        // 1. Clear out previous buffer memory so we only return the output of THIS specific command
        self.output_buffer.lock().clear();

        // 2. Generate a unique, recognizable boundary marker (token delimiter)
        // This tells our parser exactly when the requested command has finished running.
        let delimiter = format!("__AGENT_END_{}__", tokio::time::Instant::now().elapsed().as_nanos());

        // 3. Craft the raw command string injection
        // We append a carriage return (\n) and echo out our distinct structural delimiter token.
        let formatted_input = format!("{}\necho {}\n", command, delimiter);

        // 4. Ship the raw bytes down into the active PTY stdin channel
        {
            let mut writer = self.writer.lock();
            writer.write_all(formatted_input.as_bytes())?;
            writer.flush()?;
        }

        // 5. Asynchronously poll our shared background memory buffer until the delimiter token appears
        let timeout_duration = std::time::Duration::from_secs(60);
        let start_time = std::time::Instant::now();

        loop {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;

            let buffer_content = self.output_buffer.lock().clone();

            if buffer_content.contains(&delimiter) {
                // Strip out ANSI escape characters (colors, cursor movements) and our custom delimiter token
                let clean_output = clean_terminal_output(&buffer_content, &delimiter);
                return Ok(clean_output);
            }

            if start_time.elapsed() > timeout_duration {
                return Err("Command execution timed out inside the persistent PTY environment.".into());
            }
        }
    }
}

/// Helper function to strip terminal color codes and parse execution blocks cleanly
fn clean_terminal_output(raw_output: &str, delimiter: &str) -> String {
    // Basic regex or replacement logic to strip out shell prompts and metadata
    let lines: Vec<&str> = raw_output.lines().collect();
    let mut filtered_lines = Vec::new();

    for line in lines {
        if line.contains(delimiter) {
            break; // Stop parsing when we hit our completion boundary token
        }
        // Optional: Run an ANSI-strip sequence here if your LLM doesn't like color escape bytes
        filtered_lines.push(line);
    }

    filtered_lines.join("\n")
}
```

---

## Step 2: Containerized Isolation with bollard

While the PTY preserves state beautifully, if your AI agent clones an unvetted repo from GitHub and executes code locally, it can still maliciously read your personal host machine documents. To achieve true sandboxing, we combine our logic with Docker containers via the **bollard** crate.

Add this dependency to your `Cargo.toml`:

```toml
[dependencies]
bollard = "0.15"  # Core Docker client library for Rust
```

Here is how you initialize a containerized environment, map your workspace folder directly into it, and control it safely via Rust code:

```rust
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, LogOutput};
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::Docker;
use futures_util::StreamExt;  // For processing live async docker stdout stream blocks

pub struct DockerSandbox {
    docker: Docker,
    container_name: String,
}

impl DockerSandbox {
    pub async fn connect_and_spawn(host_workspace_dir: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Connect directly to the local system's background Docker engine socket
        let docker = Docker::connect_with_local_defaults()?;
        let container_name = format!("agent_sandbox_{}", tokio::time::Instant::now().elapsed().as_secs());

        // Configure your container blueprint
        let config = Config {
            image: Some("python:3.11-slim"),  // Light image equipped for software engineering agents
            tty: Some(true),                   // Keep container open persistently waiting for input loops
            attach_stdin: Some(true),
            attach_stdout: Some(true),
            // Bind-mount the exact target path into an isolated directory structure inside the container
            host_config: Some(bollard::service::HostConfig {
                binds: Some(vec![format!("{}:/workspace", host_workspace_dir)]),
                ..Default::default()
            }),
            working_dir: Some("/workspace"),
            ..Default::default()
        };

        // Spin up and initialize the isolated machine
        docker.create_container(Some(CreateContainerOptions { name: &container_name, platform: None }), config).await?;
        docker.start_container(&container_name, None::<StartContainerOptions<String>>).await?;

        Ok(Self { docker, container_name })
    }

    /// Safely executes an isolated command string inside the secure Docker ecosystem
    pub async fn run_command(&self, command_str: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Construct execution command settings
        let exec_config = CreateExecOptions {
            cmd: Some(vec!["bash", "-c", command_str]),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            working_dir: Some("/workspace"),  // Enforces the workspace root as execution default
            ..Default::default()
        };

        // Register execution handle inside the container architecture
        let exec_id = self.docker.create_exec(&self.container_name, exec_config).await?.id;

        // Fire off execution and stream live data vectors straight out of the isolated kernel
        let mut output_accumulator = String::new();
        if let StartExecResults::Attached { mut output, .. } = self.docker.start_exec(&exec_id, None).await? {
            while let Some(Ok(log_chunk)) = output.next().await {
                match log_chunk {
                    LogOutput::StdOut { message } => {
                        let text = String::from_utf8_lossy(&message);
                        println!("[Docker Stdout]: {}", text);
                        output_accumulator.push_str(&text);
                    }
                    LogOutput::StdErr { message } => {
                        let text = String::from_utf8_lossy(&message);
                        println!("[Docker Stderr]: {}", text);
                        output_accumulator.push_str(&format!("ERROR: {}", text));
                    }
                    _ => {}
                }
            }
        }

        Ok(output_accumulator)
    }
}
```

---

## Step 3: The Complete Production Pipeline

When designing an AI Agent platform like Agent Zero in Rust, you combine both techniques into your final operational agent pipeline:

1. **Boot Loop Awareness:** On start, your Rust app runs `DockerSandbox::connect_and_spawn("/home/user/my_project")`.
2. **Git Clones & Inputs:** The AI runs its initial git clone directly into the bound volume space safely inside the Docker container via `sandbox.run_command("git clone <repo> .")`.
3. **Prompt Loop & Tool Routing:** When the LLM decides to type terminal operations, it triggers your tool. The tool passes the raw input parameters directly down to `sandbox.run_command(cmd)`.
4. **Self-Healing Interception:** If the execution block results in terminal crash states, the `output_accumulator` grabs the failure log data and passes it straight back into the LLM system prompt context, allowing the model to analyze and fix its script safely without putting your physical operating system at risk.

Would you like to see how to wrap either of these modules inside an explicit `rig-rs` Custom Tool trait structure so your AI can discover and execute these terminal functions natively?
