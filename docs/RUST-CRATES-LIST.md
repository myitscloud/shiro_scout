* **portable-pty**: A crate used to open and manage persistent pseudo-terminal (PTY) master/slave pairs, allowing a background shell process to stay alive continuously.
* **tokio**: An asynchronous runtime for Rust used to handle async tasks, sleep intervals, and processing live streams.
* **parking_lot**: Provides smaller, faster, and safer implementations of synchronization primitives like Mutex.
* **bollard**: An asynchronous Docker client library used to programmatically interface with the local system's background Docker engine socket.
* **camino**: Provides UTF-8 backed paths like Utf8Path and Utf8PathBuf, ensuring all file paths are clean, valid strings readable by an LLM.
* **sysinfo**: A system metrics crate used to gather operating system, hardware specifications, CPU usage, and memory diagnostics.
* **which**: A tool verification crate used to dynamically check if specific binaries (like python3, git, or docker) are installed on the host.
* **platforms**: Provides compile-time and runtime target triple definitions to help determine underlying binary architectures.
* **current_platform**: Complements target architecture sensing by resolving the exact current target triple at runtime.
* **dotenvy**: A well-maintained fork of the original dotenv crate used to cleanly load API keys and environment configuration files.
* **duct**: A crate for chaining heavy shell commands, building pipelines, and easily redirecting standard error streams into standard output.
* **tracing**: The industry-standard logging and diagnostic framework for recording structured async application events.
* **tracing-appender**: A utility crate providing a simple log-file rotating mechanism and non-blocking file writers for tracing layers.
* **miette**: An advanced diagnostic error reporting crate that structures internal Rust errors into readable markdown help text.
* **safe-path**: A utility library or concept used to sanitize file structures and explicitly prevent directory traversal attacks.