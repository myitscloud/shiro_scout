# Project Aegis — GLOSSARY.md

> **Normative definitions.** All terms used in UI copy, documentation, code identifiers, and agent prompts must match these definitions exactly. New user-facing terms require a glossary entry before merge.

---

## A

### Agent
An AI worker powered by AgentKit that runs inside the Docker sandbox. Each agent has its own state machine (idle → thinking → tool_exec → completing → done), conversation history, and tool access scope. Agents can communicate with each other via AgentKit's `createNetwork()`.

### Agent Lifecycle
State machine governing agent behavior: `idle` → `thinking` → `gathering_context` → `running_tool` → `reviewing_output` → `complete` (or `error`). Screens in UI: idle (steady green), thinking (purple glow pulse), waiting (blue glow).

### AgentKit
TypeScript framework for building multi-agent AI systems with deterministic routing, typed state machines, and native MCP protocol support. Developed by Inngest. Provides `createAgent()`, `createNetwork()`, and MCP client/server primitives.

### Air-Gapped
Network configuration where the Docker container has no network access (`--network none` or `network_mode: "none"` in Docker compose). All outbound communication (LLM API calls, web searches) is proxied through the Tauri host application. This prevents data exfiltration and lateral network attacks.

## B

### bollard
Async Rust crate providing native bindings for the Docker Engine API. Used for all container lifecycle operations: connect, pull images, create, start, stop, exec, stream logs. Replaces shell scripts for Docker management.

### Bottom Drawer
Collapsible panel at the bottom of the app window (max 40vh) containing three tabs: Logs (filterable event stream), Terminal (xterm.js shell into sandbox), Telemetry (agent performance stats, token counts, tool timing). Toggle with `` Ctrl+` ``.

### Bridge (HTTP Bridge)
Communication channel between the Rust Tauri backend and the AgentKit agent runtime inside the Docker sandbox. An Express/Fastify HTTP/WebSocket server runs inside the container on localhost:8080.

### Batch Loop
The execution engine per SESSION_PROTOCOL §4. A cycle of REFRESH → SELECT (up to 8 items) → PER ITEM (ROUTE line → spec check → execute → verify → book-keep) → BATCH CLOSE → LOOP. Managed by the Orchestrator (Agent 0).

## C

### Capability Drop
Docker security configuration that removes Linux kernel capabilities from the container. Project Aegis drops ALL capabilities (`cap_drop: ["ALL"]`) to minimize the kernel attack surface. Prevents privilege escalation from a compromised agent.

### Command Allowlist
Rust-enforced list of shell commands that the agent is permitted to execute via `docker exec`. Commands outside the allowlist are blocked at the Rust backend level, before reaching Docker. Configurable per-session.

### Container Attestation
Cryptographic verification that the sandbox container is running with the expected security configuration. A hash of the container's config, image digest, and security parameters is computed before the first IPC message.

## D

### DECISIONS.md
Team's working memory file. Append-only, newest-first format documenting task outcomes, agents involved, escalations, and session summaries. Does not contain transcripts — only outcomes and actionable information.

### Docker Sandbox
See **Sandbox**.

## E

### Error-Message Standard (A11)
Format for all user-facing errors: **what happened** → **why (best known)** → **what to do next**. Error codes appended for escalation only. No blame-the-user phrasing, no stack traces, no credentials in error text.

## F

### Feature Flags
Rust-side `Feature` enum controlling which capabilities are available. Phase 1 features (basic chat, Docker detection) are always on. Phase 2+ features (VNC, plugin system, multi-agent networks) are behind flags.

## G

### Glass Elevation
Z-depth levels in the Neo-Glass Terminus design language. Communicated by opacity and blur, not box shadows: `base` (85%, 8px blur), `raised` (92%, 12px blur), `overlay` (95%, 16px blur), `tooltip` (98%, 20px blur).

### GLOSSARY.md
This file. The canonical source of term definitions for the project. New terms require an entry before being used in UI, docs, or code identifiers.

## H

### HITL (Human-in-the-Loop)
Security checkpoint where the system pauses before executing a potentially destructive action and requests user approval. Features: 60-second countdown timer, default-deny (auto-reject at timeout), emergency kill button, no "approve all future" toggle.

## I

### IPC (Inter-Process Communication)
Communication channel between the Tauri 2 WebView (React frontend) and the Rust backend. Uses typed Tauri `invoke()` calls with serde serialization. Every Rust command has exactly one typed wrapper in `ipc.ts`.

### IPC Contract
Type definitions for all IPC messages between the three layers: Rust ↔ WebView (typed invoke), Rust ↔ Docker sandbox (stdout/HTTP), AgentKit ↔ tools (MCP). Changes to any layer require updates to all three.

## M

### MCP (Model Context Protocol)
Open protocol that standardizes how AI agents connect to external tools and data sources. AgentKit provides native MCP client support. In Project Aegis, MCP servers run inside the Docker sandbox as separate child processes.

### MCP Server
A server implementing the Model Context Protocol that provides tools to agents. Bundled MCP servers: Filesystem (bounded to /workspace), Text Editor, and Shell. Future: GitHub, Web Search, Database.

### Mini-Spec
Short specification document that must exist before any implementation task begins. Format: Task description, Layers touched, Owning agent, Acceptance criteria, Non-functional checklist, Out of scope, Review triggers.

### Model Routing
Decision process for selecting which LLM model to use for a given agent task. Default: the primary cost-efficient model. Escalation to stronger models for: `unsafe` blocks, FFI, capability changes, cross-layer refactors, two failed attempts.

### Monorepo
Project repository structure with three main workspaces: `src-tauri/` (Rust backend), `src/` (React/TypeScript frontend), and `sandbox/` (Dockerfile + AgentKit runtime + MCP servers).

## N

### Neo-Glass Terminus
Design language for Project Aegis: fusion of glassmorphism (frosted glass surfaces), terminal-inspired typography (JetBrains Mono default), and cybernetic accent lighting (purple #8B5CF6). Dark-first with light mode as accessibility toggle.

### Network (AgentKit)
A collection of agents connected via AgentKit's `createNetwork()` that share state and can communicate. In the shared-container model, all agents in one container participate in one logical network.

## P

### Phase Indicator
Visual element showing an agent's current lifecycle state. Uses icons and glow effects: `◉` idle/online (green), `◐` thinking (purple), `◎` gathering context, `⚡` running tool, `◉` reviewing, `⚠` error (red), `✋` awaiting human (blue).

### Privacy Badge
Navbar element showing provider type: 🔒 = local provider (Ollama, LM Studio — no data leaves the machine), ☁ = cloud provider (OpenAI, Anthropic). Trust signal for the user.

### Prompt Injection Defense
Rust middleware layer that scans all tool inputs for path traversal (`../`, `~`), command injection (`;`, `|`, `$(...)`), and known jailbreak patterns before forwarding to Docker exec.

## R

### Ring-1/2/3 Verification
Verification levels: Ring-1 = local development verification (npm run dev, cargo build, docker ps test). Ring-2 = CI pipeline (all gates: lint, typecheck, test, audit). Ring-3 = release verification (signed, packaged, smoke-tested on clean OS).

### Rollback Quota
System of Docker commit snapshots taken before each major tool execution. Configurable limit (e.g., 10 snapshots × 50MB each). Allows workspace state to be reverted if the agent makes an unrecoverable error.

### ROUTE Line
Per O11 (AGENTS.md §7), every item begins with a written line: `ROUTE: <item-id> → <owning role> | reviewers: <roles> | ring: 1|2`. No ROUTE line means the item has not started.

## S

### Sandbox
The Docker container that provides an isolated execution environment for AI agents. Security configuration: read-only rootfs, no network, no capabilities, non-root user, resource limits, health check.

### Session
A single chat conversation from the user's first message to session reset or close. Sessions are auto-saved in the Tauri backend, grouped by date in the sidebar, and support rename, delete, and export.

### Shared-Container Model
Architecture pattern where one long-lived Docker sandbox container hosts multiple AgentKit agents as child processes, rather than one container per agent.

### STOP/ASK Protocol
A halt mechanism per SESSION_PROTOCOL §5. 7 triggers (STOP-1 through STOP-7) that pause the Batch Loop and return control to the human. Each has a defined message shape. Firing STOP conditions overrides the loop — never invent work to stay busy.

## T

### Tauri IPC
See **IPC**.

### Tool Call Accordion
Expandable UI component showing tool execution details: tool name, input parameters, output/result, duration, status (running/success/error/fail). Appears inline in the chat thread. Collapsed by default.

### Tool Registry
Backend system that maintains a registry of authorized MCP tools with allowlist filtering. Each tool call passes through the Rust backend for validation before execution.

## W

### Workspace
Persistent directory (`/workspace`) mounted into the Docker sandbox where agents store files, code artifacts, and session data. The only writable surface in an otherwise read-only rootfs. Persists across agent sessions.

### WIP Limit
Per O12 (AGENTS.md §7), maximum 2 concurrent subordinates at any time (hardware cap: 4 total agents). Default to sequential delegation; use concurrency only for fully independent items in disjoint directories (O13).
