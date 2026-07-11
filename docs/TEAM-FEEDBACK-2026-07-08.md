# Project Aegis — Full Dev Team Analysis Report

> **Date:** 2026-07-08
> **Status:** Feedback collection complete — awaiting owner review
> **Reference Documents Analyzed:**
> - `/a0/usr/projects/shiro_scout/docs/Arch_Design/Agent Zero Architecture and PRD.md`
> - `/a0/usr/projects/shiro_scout/docs/Arch_Design/AEGIS-DESIGN-GUIDE.md` (715 lines)
> - `/a0/usr/projects/shiro_scout/docs/Arch_Design/MERGE-PRD-ROUGH-DRAFT.md`
> - `/a0/usr/projects/shiro_scout/docs/mockup/aegis-neo-glass-terminus-mockup.html` (1169 lines)
> - `/a0/usr/projects/shiro_scout/docs/mockup/aegis-mockup.png`
> - `/a0/usr/projects/shiro_scout/docs/agent-profiles/` (8 role files)
> - `/a0/usr/projects/shiro_scout/docs/mimicking-agent-zero/` (AgentKit starter + code-assistant-agent)

---

## 🏛️ 1. Windows Systems Architect — 5 Ideas

| # | Idea | Rationale & Reference |
|---|------|----------------------|
| 1 | **MicroVM Future-Proofing** — Architect the sandbox abstraction behind a `SandboxProvider` trait so we can swap Docker (via bollard) for Firecracker/gVisor without rewriting the orchestration layer. The PRD already flags Docker's shared-kernel risk. A trait with `start()`, `exec()`, `stop()`, `stream_logs()` methods keeps the API clean. | PRD §34-38 discusses Docker vs microVM tradeoffs. Failure trap: locking into Docker-only architecture then needing Firecracker for enterprise deployments. |
| 2 | **Shared-Container Model with Per-Agent `fork()`** — Instead of `child_process.fork()`, use Rust's `std::process::Command` to spawn AgentKit agent processes inside the container with separate PID namespaces. This gives better process isolation and lets Rust enforce the command allowlist before anything reaches Docker exec. | Merge PRD §3.2-3.4; Security Engineer's abuse-case catalog §2 "Tampered script on disk". Windows Architect owns Rust/Tauri backend per routing table. |
| 3 | **Stop/Rollback Quota System** — Implement a bollard-based commit/restore cycle: `docker commit` (snapshot) before each major tool execution, with a configurable rollback quota (e.g., 10 snapshots × 50MB each). Use Rust's async to manage the lifecycle without blocking the UI. | PRD §84 "Snapshot and Time Travel"; Merge PRD §6.5 "Workspace snapshot and restore". |
| 4 | **TCP-over-stdio IPC Bridge** — Tunnel agent communication over the container's stdin/stdout streams through bollard's `attach_container` instead of exposing an HTTP server inside the container. Rust reads/writes framed JSON messages. HTTP only for MCP servers. Eliminates a network listener inside the sandbox. | Security Engineer S3 "minimal capabilities". PRD §100-101 discusses log streaming via Docker API. |
| 5 | **ARM64 Cross-Compilation CI Matrix** — Set up a GitHub Actions matrix for x86_64 and aarch64 Windows, macOS (Intel + Apple Silicon), and Linux (.deb + AppImage). Use `cross` or `cargo-zigbuild` for cross-compilation. | Release Engineer D1 "CI runs on Windows runners"; PRD §83 "Cross-Platform Compilation". |

---

## 🎨 2. Frontend Engineer — 5 Ideas

| # | Idea | Rationale & Reference |
|---|------|----------------------|
| 1 | **Monaco Editor Live Sync** — Implement a WebSocket-backed file watcher that pushes changes from the sandbox workspace to Monaco's editor buffer. When the agent writes output, the UI updates in real-time. xterm.js terminal should share this same websocket channel. | Mockup shows Monaco + xterm.js; Design Guide §3.2 Bottom Drawer; AgentKit examples have `useChatStream` hook. Must follow F5 (cleanup listeners) and F4 (loading/error states). |
| 2 | **Phase Indicator State Machine** — Implement a React context that reads agent phase from IPC events and drives ALL visual states from a single source of truth: navbar dot, sidebar avatar glow, chat phase strip, right panel status. One event to update all. | Design Guide §5.2 Agent Lifecycle UI Mapping (8 states with icons). Mockup `.phase-strip` and avatar `.thinking` animation. |
| 3 | **Virtualized Chat with Auto-Pause** — Use `@tanstack/react-virtual` for the message thread with an "auto-pause" feature: when the user scrolls up to read history, the viewport stops auto-scrolling. New messages get a "↓ New messages" badge. | Design Guide §9.1: `@tanstack/react-virtual (~10KB)` in budget. F10 "Large data is virtualized". |
| 4 | **Dark/Light Theme as CSS Custom Properties Swap** — Single React context toggle switching a `data-theme` attribute on `<html>`. No runtime CSS-in-JS, no FOUC. Respect `prefers-color-scheme` for initial detection, manual override in Tauri store. | Design Guide §1.3 Dark-First Mandate, §2.1 Light Palette, §7.4 Reduced Motion. Mockup lines 57-75 show complete light palette swap. |
| 5 | **AgentKit Chat Components as Submodule** — The AgentKit starter has `Chat.tsx`, `ChatMessage.tsx`, `ChatSidebar.tsx`, `ChatHeader.tsx`, and `useChatStream` hook. Import these and wrap in Tauri's IPC layer instead of rebuilding from scratch. | Merge PRD §4: "use `@inngest/use-agent` React hooks directly, or build a custom state layer?" The answer: use AgentKit hooks, wrap in Tauri IPC. |

---

## ✅ 3. QA Test Engineer — 5 Ideas

| # | Idea | Rationale & Reference |
|---|------|----------------------|
| 1 | **Contract Test Fixtures: Rust ↔ TypeScript ↔ AgentKit** — Create golden JSON fixtures for each of the 3 IPC boundaries (Rust↔WebView, Rust↔Docker, AgentKit↔MCP). Rust tests deserialize through serde; TypeScript tests validate against IPC types. | Q3: "Contract tests are the crown jewels. Golden JSON fixtures captured from real runs, verified in two directions." |
| 2 | **Sandbox Health Probe as Integration Test** — Script that pulls image, starts container with full security constraints, runs known tool, checks output, tears down. Run on CI schedule per Q1 isolation rules. | Q1: Layer isolation by default. Q11: E2E is a thin smoke layer. |
| 3 | **Flaky-Test Dashboard** — CI check that tracks test run history and auto-quarantines tests failing more than 2 of the last 10 runs. Each quarantine generates an issue with `flaky-test` label. | Q9: "A flaky test is quarantined with an issue the same day and fixed or deleted within two change sets." |
| 4 | **Streaming UI Performance Test** — vitest + Playwright test: opens app, sends prompt to mock agent, measures time-to-first-token, frame rate during stream, scroll performance with 500+ messages. Thresholds: first token <500ms, 60fps, no jank. | F4: loading/empty/error/success states. Design Guide §6.3 streaming cursor. Required per Q4. |
| 5 | **Mutation Testing for Rust Security Boundary** — Systematically inject bugs (allowlist bypass, path traversal, capability escalation) and verify test suite catches them. | Security Engineer S1: mandatory review triggers. Q10: tests must be able to fail for a stated reason. |

---

## 📋 4. Code Reviewer — 5 Ideas

| # | Idea | Rationale & Reference |
|---|------|----------------------|
| 1 | **Three-Layer Hallucination Audit (R3++)** — Extend R3 to require every new API symbol has a grep hit showing prior art, pinned docs citation, OR unresolved `VERIFY:` marker. Dockerfiles: scrutinize `apt-get`, COPY, ENV with secrets risk. | R3: "Hallucination audit. Every API symbol gets one of: grep hit, doc citation, or `VERIFY:` marker." |
| 2 | **Mini-Spec → Diff Scope Enforcement (O7/R4)** — Every PR references its mini-spec. Reviewer checks: does this diff's file list match the spec's declared scope? Files outside scope get bounced. | R4: "Files changed must match the mini-spec's list." O7 from Orchestrator profile. |
| 3 | **IPC Contract Sync Gate (R5)** — Any PR touching one IPC layer must show all three layers changed together. Automated check: git diff matching `src-tauri/src/` but not `src/ipc.ts` auto-flags `needs-sync`. | R5: "Any diff touching one layer of the IPC contract shows all three layers changed together." F2: "Every Rust command has one typed wrapper in ipc.ts." |
| 4 | **VERIFY Marker Tracking Dashboard** — CI job extracts all `VERIFY:` markers from diff, adds new ones to a project board. Markers surviving 3 change sets without resolution escalate to Orchestrator. | R9: "VERIFY markers are resolved or explicitly accepted." |
| 5 | **Dockerfile + Compose Review Checklist** — Specific checklist: (a) No hardcoded secrets in ENV/ARG, (b) Base image pinned to digest, (c) Multi-stage build, (d) HEALTHCHECK defined, (e) USER non-root before final CMD. | Security Engineer S1: Docker/build triggers mandatory review. Merge PRD §3.1 "Create Dockerfile". |

---

## 📝 5. Documentation Engineer — 5 Ideas

| # | Idea | Rationale & Reference |
|---|------|----------------------|
| 1 | **Create ADR-001 through ADR-006 Immediately** — Formalize decisions: (001) Shared-container model, (002) HTTP bridge vs TCP-over-stdio, (003) Neo-Glass Terminus, (004) React 18 + TypeScript strict, (005) bollard vs Podman API, (006) MCP integration. Store at `/docs/adr/`. | T3: "Every architectural decision gets an ADR, numbered, append-only." |
| 2 | **Create DECISIONS.md Entry** — Per T5 format, append entry documenting this feedback session with outcomes, agents involved, pending action items. | T5: "DECISIONS.md is the team's working memory. Append-only, newest first." |
| 3 | **Create GLOSSARY.md** — Define: Sandbox, Agent, Network, MCP, HITL, bollard, AgentKit, Tauri IPC, Capability Drop, Air-Gapped, Neo-Glass Terminus. Store at `/docs/GLOSSARY.md`. | T7: "GLOSSARY.md is normative. Terms defined once and used identically in UI copy, docs, and code identifiers." |
| 4 | **Expand User Personas** — Merge PRD §3 defines 4 personas in 2-5 words each. Expand each to 10+ lines with name, role, goals, pain points, success scenario, environment. | T6: "Audience discipline. User docs written for L1-L3 technicians: task-oriented, plain language." |
| 5 | **In-App Help Content File** — Create `/docs/in-app/help-content.md` read by frontend via Tauri IPC for: wizard steps, tooltip text, empty-state copy, error templates. | T2: "One source of truth; link, don't copy." Mockup shows wizard, empty states, toasts. |

---

## 🔒 6. Security Engineer — 5 Ideas

| # | Idea | Rationale & Reference |
|---|------|----------------------|
| 1 | **Prompt Injection Defense Layer** — Rust middleware scanning all tool inputs for: path traversal (`../`, `~`), command injection (`;`, `|`, `$(...)`), known jailbreak patterns. Log and block before reaching Docker exec. | S2: Threat model per feature. Abuse case #3 "Hostile input." |
| 2 | **MCP Server Trust Boundary Documentation** — `THREAT_MODEL.md` entry for MCP: assets, entry points, threats (data exfiltration, prompt injection, supply chain), mitigations (separate containers, rate limits, audit trail). | S2: "New features get a STRIDE pass recorded in THREAT_MODEL.md." |
| 3 | **Workspace Audit Trail with Immutable Logs** — Every file change logged with: timestamp, agent ID, tool name, path, operation, SHA-256 checksum. Logs on read-only volume agent cannot modify. | S6: Log-redaction policy. Merge PRD §6.4 "Session recording and audit trail." |
| 4 | **Cryptographic Attestation of Container State** — Hash of container config + image digest + security params verified in first IPC message. Prevents compromised Docker daemon from running insecure container. | S3: "Capabilities are minimal and per-window." PRD §38: "enforce capability drops." |
| 5 | **Automated `cargo audit` + `npm audit` + Secret Scan Gate** — CI fails on any vulnerability. Secret scan on full diff including Dockerfiles and docs. A hit is a BLOCKER and a rotation event. | S4: "cargo audit, cargo deny check, npm audit on every change set; new advisories block release." S5: "Secret scan hit = blocker + rotation event." |

---

## 🚀 7. Release/DevOps Engineer — 5 Ideas

| # | Idea | Rationale & Reference |
|---|------|----------------------|
| 1 | **Docker Image as CI Artifact** — Build Docker image on release, push to GitHub Container Registry, sign with cosign, generate CycloneDX SBOM. Desktop app checks signed digest before pulling. | D9: "SBOMs generated per release." D4: signing all artifacts. |
| 2 | **MSI Silent Install Test in CI** — Automate MSI install on clean Windows 11 VM via Windows runner + `msiexec /i app.msi /qn`. Fail pipeline if install fails or Aegis.exe doesn't launch. | D5: "MSI as enterprise-primary artifact (silent install verified: msiexec /i app.msi /qn)." |
| 3 | **Lockfile Policy Enforcement** — CI rejects any PR modifying `Cargo.lock`/`package-lock.json` without changing corresponding source manifest. Lockfile change without source change = suspicious. | D2: "npm ci never npm install. Cargo.lock committed and honored." |
| 4 | **Cross-Platform Version Sync Check** — GitHub Action extracts version from `tauri.conf.json`, `Cargo.toml`, `package.json`; fails if mismatched. Also checks Git tag exists matching `v*.*.*`. | D8: "tauri.conf.json, Cargo.tomp, package.json versions move together." |
| 5 | **First-Run Telemetry (opt-in)** — Anonymous startup telemetry: OS version, WebView2 version, Docker availability, sandbox startup time, first action latency. Critical for release quality decisions. | Merge PRD §7.5-7.6. Security Engineer S6: log-redaction (no credentials in telemetry). |

---

## ♿ 8. Accessibility & UX Specialist — 5 Ideas

| # | Idea | Rationale & Reference |
|---|------|----------------------|
| 1 | **WCAG AA Audit for Glass Effects** — Test text contrast AT ALL glass opacity levels (85%, 92%, 95%, 98%), focus indicator visibility through each elevation, `forced-colors: active` mode. | A3: Contrast minimums. A5: Windows high-contrast. Design Guide §7.1 ratios on solid backgrounds, not through glass. |
| 2 | **Keyboard-Only Power User Mode** — VIM-style command mode: `Ctrl+K` palette, `/` triggers commands, `j`/`k` navigates history. | A1: "Helpdesk technicians live on keyboards — treat keyboard UX as primary interface." A10: Shortcuts documented. |
| 3 | **Live Region Strategy for Streaming** — Tool call accordions need `aria-expanded`, progress bars need `role="progressbar"` + `aria-valuenow`, errors need `role="alert"` with assertive announcement. | A7: Screen readers. A12: Long-operation UX. Mockup has `.tprog` and `.terr` but no ARIA. |
| 4 | **Auto-Deny Timer for HITL Dialogs** — 45s countdown must be: visible, announced via live region, pausable on focus, configurable (some enterprises need 5-min windows). Default-deny is correct but UX must not frustrate. | A13: "Safe action as default focus." Mockup lines 442-456: timer + default-deny. |
| 5 | **Error-Message Copy Audit (A11)** — Every error must follow: what happened → why (best known) → what to do next. No blame, no stack traces, error codes appended only. Create `ERRORS.md` dictionary. | A11: error-message standard. Documentation Engineer T8: co-authored with A11y Specialist. |

---

## 🔍 9. Orchestrator / Tech Lead — 5 Ideas

| # | Idea | Rationale & Reference |
|---|------|----------------------|
| 1 | **Phase DAG with Parallelization** — Reorganize phases as DAG: Phase 0 (design tokens + component lib) prerequisite for UI. Phase 1.5 (Docker image) before Phase 2. Frontend/backend teams parallelize Phase 2-3. | O2: "No task without mini-spec." O3: "One agent per layer per change set." |
| 2 | **Create 10 Mini-Specs for Phase 0 Now** — Break token + component work into 10 specs (CSS tokens, Button, ChatMessage, CodeBlock, ToolCallAccordion, Sidebar, Navbar, BottomDrawer, Modal/Toast, Settings). Each specs acceptance criteria from mockup. | O2 mini-spec template. Routing table: Frontend Engineer owns all React components. |
| 3 | **AgentKit Bridge Decision Before Phase 3** — Document this now. Recommend Option B (HTTP server inside Docker container): keeps AgentKit in native Node.js, HTTP debuggable, Rust uses reqwest, avoids bundling Node. Create ADR-002. | Merge PRD §6.1 is a Phase 3 blocker. Security Engineer S1: affects security posture. |
| 4 | **Model Routing: Default to Local** — Recommend Ollama as default (🔒 privacy badge), cloud as optional upgrade. Privacy-first, no API key friction for first-time users, competitive local model performance. | Orchestrator model routing. Merge PRD §6.9. Design Guide Privacy Badge §3.2. |
| 5 | **Two-Strike Escalation Template** — GitHub Issue template for 2nd failed attempt: implementer fills what was tried, observed errors, hypothesis, requested escalation model. Logs result in DECISIONS.md. | O5: "Third attempts without escalation forbidden." Orchestrator §Failure-Loop Handling. |

---

## ⚡ 10. Dev Team Lead — 5 Ideas

| # | Idea | Rationale & Reference |
|---|------|----------------------|
| 1 | **Monorepo Structure** — `src-tauri/` (Rust), `src/` (React/TS), `sandbox/` (Dockerfile + AgentKit + MCP), `docs/` (ADRs, design), `scripts/`, `tests/`, `examples/`. | Merge PRD §5 architecture. Currently missing `sandbox/` and `docs/` dirs. |
| 2 | **Incremental Docker Image Strategy** — Three layers: `aegis-base` (Alpine + Node + Python, rarely changes), `aegis-agent` (base + AgentKit + MCP, per release), `aegis-workspace` (agent + workspace, per-session ephemeral). | Architecture PRD §113: Dockerfile. Merge PRD §3.1. |
| 3 | **Feature Flags for Incremental Delivery** — Rust-side `Feature` enum toggled via config. Phase 1 always on; Phase 2+ (VNC, plugins, multi-agent) behind flags. Ships MVP without branching nightmares. | Merge PRD §5 "Out of Scope (v1)". |
| 4 | **Ring-1 Verification for All Agents** — Aegis-specific SESSION_PROTOCOL.md: Ring-1 = local dev, Ring-2 = CI, Ring-3 = release. Each agent verifies before declaring done. | Orchestrator §Agent Zero Mapping: "verify with Ring-1 commands only." |
| 5 | **Weekly Sprint Loop** — Day 1: Plan (mini-specs → assign → estimate). Day 2-4: Build (implement → self-verify → submit). Day 5: Review + Retro (QA → Code Review → Security → Merge → Update DECISIONS.md → Retro). | Orchestrator §Task Pipeline. Merge PRD §9. |

---

## 📊 Summary: 50 Ideas by Phase Priority

| Phase | Ideas Count | Key Themes |
|-------|-------------|------------|
| **Phase 0 — Foundation** | 8 | Monorepo structure, ADR creation, GLOSSARY.md, design token mini-specs, theme system, persona expansion |
| **Phase 1 — Scaffold** | 6 | Tauri 2 setup, cross-platform CI, AgentKit bridge decision, lockfile policy, Docker image layering |
| **Phase 2 — Sandbox** | 7 | SandboxProvider trait, TCP-over-stdio IPC, health probe tests, cryptographic attestation, prompt injection defense |
| **Phase 3 — AgentKit** | 5 | AgentKit integration, Feature flags, Phase indicator state machine, MCP trust boundary docs |
| **Phase 4 — UI** | 7 | Monaco live sync, Virtualized chat, Keyboard power user mode, A11y audit protocol, Error-message audit |
| **Phase 5 — LLM** | 3 | Model routing decision, First-run telemetry (opt-in), Privacy badge UX |
| **Phase 6 — Security** | 6 | Workspace audit trail, Mutation testing, Contract fixtures, HITL timer, Secret scan gate, VERIFY tracking |
| **Phase 7 — Release** | 5 | MSI silent install CI test, Docker image as CI artifact, Version sync, SBOM, Release checklist automation |
| **Process** | 3 | Sprint loop adoption, Two-strike escalation, Phase DAG parallelization |

---

*Report generated by Orchestrator/A0 on 2026-07-08 after reviewing all 14 project documents, mockup image, and AgentKit examples.*
