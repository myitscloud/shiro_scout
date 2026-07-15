# ShiroScout (codename: Project Aegis)

> **AI-powered engineering orchestrator** — a Tauri 2 desktop application that runs an autonomous AI agent inside a secure Docker sandbox, with a Neo-Glass Terminus UI.

---

## Overview

ShiroScout is a desktop application for Windows 11 that combines a Docker-isolated AI sandbox with a rich React frontend. It enables autonomous AI agents to execute code, run shell commands, perform file operations, and orchestrate multi-step engineering tasks — all within a secure, network-isolated container environment.

**Key capabilities:**
- **Secure sandbox** — Docker container with `network_mode: none`, `cap_drop ALL`, read-only rootfs
- **Multi-provider LLM** — DeepSeek, OpenAI, Groq, Together, Ollama, LiteLLM via Rig 0.39.0
- **3-role LLM config** — Separate chat, utility, and embedding model configurations
- **Streaming responses** — Real-time token streaming via IPC events to `StreamingText` UI
- **Tool execution bridge** — Rust → Docker exec via HTTP bridge for terminal, file, and code tools
- **Agent state machine** — Idle → Thinking (streaming) → Tool (bridge invoke) → Done
- **Persistent PTY sessions** — Long-running shell sessions inside the sandbox
- **MCP server discovery** — Auto-detect Model Context Protocol servers on localhost
- **Agent state persistence** — Auto-save/restore conversation history across restarts
- **HITL confirmation** — Human-in-the-loop approval for dangerous operations
- **Windows Credential Manager** — Secure API key storage via Win32 `CredWriteW`/`CredReadW`
- **Neo-Glass Terminus UI** — Dark glassmorphism design system with 14+ components

---

## Architecture

```
 WebView (React 18 / TypeScript / Vite)
      ↓ IPC (typed invoke via @tauri-apps/api)
 Rust Backend (Tauri 2 commands, bollard Docker API)
      ↓ HTTP/WebSocket bridge
 Docker Sandbox (debian:bookworm-slim — no network, read-only rootfs)
      ↓ WinRM / CIM (planned)
 Remote Windows Targets
```

### Key Layers

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **Frontend** | React 18, TypeScript, Vite, CSS Modules | UI components, settings, streaming display |
| **Backend** | Rust, Tauri 2, rig 0.39.0, bollard 0.18 | IPC commands, LLM orchestration, Docker management |
| **Sandbox** | Docker (debian:bookworm-slim), axum HTTP bridge | Isolated agent execution environment |
| **Security** | Win32 Credential Manager, air-gapped mode, HITL | Key storage, network isolation, human oversight |

---

## Prerequisites

- **Windows 11** (x64 or ARM64) — this is a Windows-only application
- **Docker Desktop** (with WSL2 backend) — for the sandbox container
- **pnpm** — package manager (npm is not used; see DEC-004)
- **Rust** 1.96+ — via [rustup](https://rustup.rs/)
- **Node.js** 22+ — via [nvm-windows](https://github.com/coreybutler/nvm-windows) or installer

---

## Quick Start

```bash
# 1. Install dependencies
pnpm install

# 2. Start Tauri dev server (frontend + backend)
pnpm tauri dev

# 3. Build for production
pnpm tauri build
```

### Verification Commands

```bash
# Frontend
npx tsc --noEmit        # TypeScript check
pnpm build              # Production build

# Rust backend
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml

# Supply chain
cargo deny check --manifest-path src-tauri/Cargo.toml
pnpm audit
```

---

## Project Structure

```
c:/shiro_scout/
├── src/                          # React frontend
│   ├── components/               # UI components (Button, ChatMessage, CodeBlock, etc.)
│   │   ├── AgentCard/
│   │   ├── Button/
│   │   ├── ChatMessage/
│   │   ├── CodeBlock/
│   │   ├── CodeMirrorInput/
│   │   ├── Layout/               # Navbar, Sidebar, BottomDrawer
│   │   ├── Overlay/              # Modal, Toast, ConfirmationDialog
│   │   ├── RightPanel/
│   │   ├── Settings/             # LLMProviderSettings, Settings
│   │   ├── StreamingText/
│   │   ├── ToolCallAccordion/
│   │   ├── UsageMetrics/
│   │   └── Wizard/               # FirstRunWizard
│   ├── context/                  # AppContext (global state)
│   ├── hooks/                    # useStreamingLlm, useShikiHighlighter
│   ├── styles/                   # design-tokens.css, resets.css
│   ├── App.tsx                   # Root component
│   ├── main.tsx                  # Entry point
│   └── tauri-commands.ts         # Typed IPC invoke wrappers
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── agent/                # Agent state machine, context, history, persistence
│   │   ├── llm/                  # LLM config, credential manager, keychain, health checks
│   │   ├── mcp/                  # MCP server discovery + registry
│   │   ├── pty/                  # Persistent PTY session management
│   │   ├── sandbox/              # Sandbox configuration
│   │   ├── tools/                # Tool registry + execution
│   │   ├── bridge_client.rs      # ToolExecBridge (HTTP client to sandbox)
│   │   ├── container.rs          # Docker container lifecycle commands
│   │   ├── docker_client.rs      # Docker daemon health check
│   │   ├── error.rs              # Typed error types (C7)
│   │   ├── hitl.rs               # HITL confirmation sessions
│   │   ├── prompts/              # Prompt store + persona
│   │   ├── settings.rs           # App settings + LLM settings + API key commands
│   │   ├── lib.rs                # Tauri command registration + agent orchestration
│   │   └── main.rs               # Entry point
│   ├── docker/                   # Dockerfile.sandbox, entrypoint.sh, bridge binary
│   └── capabilities/             # Tauri 2 capabilities (minimal permissions)
├── docs/
│   ├── adr/                      # Architecture Decision Records (ADR-001 through ADR-011)
│   ├── agent-profiles/           # Role profiles for 9 specialist agents
│   ├── Arch_Design/              # AEGIS-DESIGN-GUIDE.md (Neo-Glass Terminus)
│   ├── mini-specs/               # Component specs (MSPEC-001 through MSPEC-021)
│   ├── threats/                  # THREAT_MODEL.md (STRIDE)
│   └── xterm-mspec/              # Terminal integration spec
├── scripts/                      # auto-backup.ps1, build-release.ps1
├── AGENTS.md                     # Project charter, routing table, role cards
├── BUILD_PLAN.md                 # Execution plan (waves, items, status)
├── DECISIONS.md                  # Append-only decision log
├── DONE.md                       # Definition of Done (verification gates)
├── FEATURES.md                   # Feature inventory
├── GLOSSARY.md                   # Terminology
├── MEMORY.md                     # Project state & reference
└── SESSION_PROTOCOL.md           # Session lifecycle, batch loop, STOP/ASK
```

---

## Key Documents

| Document | Purpose |
|----------|---------|
| [`AGENTS.md`](AGENTS.md) | Project charter, prime directives, routing table, role cards |
| [`BUILD_PLAN.md`](BUILD_PLAN.md) | Wave execution plan with item-level status tracking |
| [`DONE.md`](DONE.md) | Verification gates (G0–G5), IPC wiring checks, completion report template |
| [`DECISIONS.md`](DECISIONS.md) | Append-only architectural and project decision log |
| [`FEATURES.md`](FEATURES.md) | Master feature inventory with priority, wave, and status |
| [`MEMORY.md`](MEMORY.md) | Single source of truth for project state, version locks, constraints |
| [`GLOSSARY.md`](GLOSSARY.md) | Project terminology reference |
| [`SESSION_PROTOCOL.md`](SESSION_PROTOCOL.md) | Session lifecycle, batch loop, STOP/ASK protocol |
| [`AEGIS-DESIGN-GUIDE.md`](docs/Arch_Design/AEGIS-DESIGN-GUIDE.md) | Neo-Glass Terminus design language specification |

---

## Tech Stack

| Category | Technology |
|----------|-----------|
| **Desktop framework** | Tauri 2.x |
| **Frontend** | React 18, TypeScript 5.x (strict), Vite 5.x |
| **Styling** | CSS Modules + custom properties (`design-tokens.css`) |
| **Icons** | Lucide React (tree-shaken) + inline SVGs |
| **LLM framework** | Rig 0.39.0 (providers: deepseek, openai, groq, together, ollama, litellm) |
| **Docker API** | bollard 0.18 |
| **Credential storage** | Windows Credential Manager via `windows-rs` (Win32 `CredWriteW`/`CredReadW`) |
| **Code highlighting** | Shiki (syntax highlighting in code blocks) |
| **Code input** | CodeMirror (via CodeMirrorInput component) |
| **Package manager** | pnpm (see DEC-004) |
| **Target** | Windows 11 (x64, ARM64) |

---

## License

MIT — see license in [`Cargo.toml`](src-tauri/Cargo.toml)
