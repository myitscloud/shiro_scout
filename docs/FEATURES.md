# ShiroScout — Feature Inventory

> **Purpose:** Master feature list derived from PRDs (docs/Arch_Design/), user personas (docs/USER-PERSONAS.md), and component inventory. Tracks release scope, priority, and implementation status.
> **Source:** PRDs + code scan + design docs
> **Last updated:** 2026-07-10

---

## 1. Core Platform

| # | Feature | Priority | Wave | Status | Source |
|:-:|---------|:--------:|:----:|:------:|--------|
| 1 | Docker sandbox with bridge HTTP server | P0 | 3 | ✅ | PRD §3.2 |
| 2 | Tauri 2 desktop shell with IPC | P0 | 1 | 🟡 | PRD §2 |
| 3 | Multi-agent orchestration (Orchestrator + agents) | P0 | 0 | 🟡 | PRD §4.1 |
| 4 | Rust axum bridge (agent-bridge) | P0 | 3 | ✅ | PRD §3.3 |
| 5 | Cross-platform targeting (Windows 11 x64/ARM64) | P1 | 8 | 🔲 | AGENTS.md C9 |
| 6 | Self-updating (Tauri updater) | P1 | 8 | 🔲 | ADR pending |

## 2. UI Components (Wave 5)

| # | Component | Priority | Wave | Status |
|:-:|-----------|:--------:|:----:|:------:|
| 7 | Design tokens + CSS modules | P0 | 5 | ✅ |
| 8 | Button component | P0 | 5 | ✅ |
| 9 | ChatMessage (markdown, code, avatar) | P0 | 5 | ✅ |
| 10 | CodeBlock (syntax highlight, copy, run) | P0 | 5 | ✅ |
| 11 | ToolCallAccordion | P0 | 5 | ✅ |
| 12 | Navbar + Sidebar | P0 | 5 | ✅ |
| 13 | BottomDrawer (Logs, Terminal, Telemetry) | P0 | 5 | ✅ |
| 14 | Modal + Dialog + Toast | P0 | 5 | ✅ |
| 15 | RightPanel | P0 | 5 | ✅ |
| 16 | Settings panel | P1 | 5 | ✅ |
| 17 | Wizard (First-run setup) | P1 | 5 | ✅ |
| 18 | StreamingText component | P0 | 6 | ✅ |
| 19 | AgentCard (agent status visualization) | P1 | 5 | ✅ |
| 20 | UsageMetrics (token tracking display) | P0 | 6 | ✅ |

## 3. LLM Integration (Wave 6)

| # | Feature | Priority | Wave | Status |
|:-:|---------|:--------:|:----:|:------:|
| 21 | Provider selector (DeepSeek, OpenAI, Groq, Together, Ollama, LiteLLM) | P0 | 6.4 | ✅ |
| 22 | 3-role configuration (chat, utility, embedding) | P0 | 6.4 | ✅ |
| 23 | API key management (Windows Credential Manager) | P0 | 6.5 | ✅ |
| 24 | LLM settings persistence (tauri-commands) | P0 | 6.5 | ✅ |
| 25 | Token usage tracking and cost estimation | P0 | 6.6 | ✅ |
| 26 | Streaming response handling (IPC events → UI) | P0 | 6.7 | 🟡 |
| 27 | Provider health check + failover | P0 | 6.8 | ✅ |
| 28 | Connection test button in Settings | P1 | 6.4 | ✅ |

## 4. Security & Operations (Wave 7)

| # | Feature | Priority | Wave | Status |
|:-:|---------|:--------:|:----:|:------:|
| 29 | Air-gapped mode (no network container) | P1 | 7 | 🔲 |
| 30 | HITL confirmation for dangerous operations | P0 | 7 | 🔲 |
| 31 | Threat model (STRIDE) | P0 | 0.7 | ✅ |
| 32 | Tauri capabilities audit (minimal perms) | P0 | 1.7 | ✅ |
| 33 | cargo-deny license/advisory check | P0 | 1.4 | 🟡 |
| 34 | .gitattributes LF enforcement | P1 | 1 | 🔲 |

## 5. Agent Runtime (Wave 4)

| # | Feature | Priority | Wave | Status |
|:-:|---------|:--------:|:----:|:------:|
| 35 | Perslent PTY shell sessions | P1 | 4 | 🔲 |
| 36 | Agent state machine (idle → thinking → tool → done) | P0 | 0 | 🟡 |
| 37 | Tool execution bridge (Rust → Docker exec) | P0 | 4 | 🔲 |
| 38 | MCP server discovery (ADR-006) | P1 | 4 | 🔲 |
| 39 | Agent state persistence | P1 | 4 | 🔲 |

---

*Generated from: docs/USER-PERSONAS.md, docs/Arch_Design/Agent Zero Architecture and PRD.md, docs/Arch_Design/MERGE-PRD-ROUGH-DRAFT.md, codebase component scan, FEATURES.md*