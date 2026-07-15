# ShiroScout — BUILD_PLAN.md

> **Purpose:** Execution plan — waves, items, owners, order. Scope lives in `FEATURES.md`; this file says *when* and *who*.
> **Rebuilt 2026-07-10** — previous file was corrupted (single row duplicated ~250×, a PowerShell file-write casualty; see DEC-005).
> **⚠ Corruption guard:** this file is ≤ 250 lines and is ONLY updated by full-file rewrite (FILEOPS-001). Never patch it with sed loops or append scripts.

Status: ✅ done · 🟡 in progress · 🔲 not started · ⏸️ blocked

---

## 1. Wave Overview

| Wave | Title | Status | Notes |
|:----:|-------|:------:|-------|
| 0 | Orchestrator Agent Core | ✅ | Docs + Rust core complete |
| 1 | Scaffold & Toolchain | ✅ | All items complete (1.B CRLF cleanup verified done) |
| 2 | Scaffold & Toolchain (initial pass) | ✅ | |
| 3 | Docker Orchestration | ✅ | Sandbox + axum bridge live |
| 4 | AgentKit Runtime | ✅ | Code exists + docs synced (FEATURES.md, README.md) |
| 5 | Core UI — Design System & Components | ✅ | 14+ components, CodeMirrorInput, Shiki highlighter |
| 6 | LLM Integration | ✅ | All items verified (6.V drift check done) |
| 7 | Security Hardening & HITL | ✅ | All items complete (7.1–7.5 verified) |
| 8 | Distribution & Release | 🟡 | 8.1 packaging done, 8.3 updater partial, 8.2/8.4 open |
| 9 | Rig Core + Cleanup | ✅ | All items verified complete; reqwest kept for health checks |

**Current priority order:** Wave 6.V (drift verify) → Wave 4 doc sync → Wave 7.3–7.5 → Wave 8.

## 2. Wave 1 — Scaffold & Toolchain ✅

| Item | Task | Owner | Deps | Status | Notes |
|------|------|-------|------|:------:|-------|
| 1.A | Baseline: run full DONE.md gate sequence on current tree, record report | QA | — | ✅ | Verified 2026-07-15: G0–G5 all pass (2 minors fixed) |
| 1.B | `.gitattributes` (`* text=auto eol=lf` + binary entries); convert stray CRLF files | Architect | — | ✅ | `.gitattributes` has `* text=auto eol=lf`; all 4 docs files LF; renormalized index |
| 1.C | Complete `cargo-deny` config (licenses, advisories, bans); add to gates | Security | 1.A | ✅ | `cargo deny check` passes all checks |
| 1.D | `git init`, initial commit, GitHub remote, push | DevOps | 1.B | ✅ | 6 commits, pushed to `github.com/myitscloud/shiro_scout.git` |
| 1.E | Tauri shell IPC completion check — every stub in `lib.rs` implemented or `// BLOCKED: BLK-n` | Architect | 1.A | ✅ | All commands registered in `generate_handler!`, no stubs |

## 3. Wave 6 — LLM Integration 🟡

| Item | Task | Owner | Deps | Status | Notes |
|------|------|-------|------|:------:|-------|
| 6.1–6.3 | Provider crates (async-openai, DeepSeek, OpenAI) | Architect | — | ✅ | Using `rig` 0.39.0 providers exclusively |
| 6.4 | Settings > LLM Providers UI (3-role pattern) | Frontend | — | ✅ | (re-verify 6.V) |
| 6.5 | API key management (Windows Credential Manager via keyring) | Architect | — | ✅ | (re-verify 6.V) |
| 6.6 | Token usage tracking + cost estimation | Frontend | — | ✅ | |
| 6.7 | Streaming responses: Rust `emit` → IPC events → StreamingText | Frontend (+Architect, C3) | 1.A | ✅ | Rig-native streaming in `agent.rs::think_with_streaming()`; `useStreamingLlm.ts` hooks properly wired; `AppContext.tsx` listens for `llm-token` events |
| 6.8 | Provider health check + failover | QA + Reviewer | — | ✅ | (re-verify 6.V) — 1 test fixed |
| 6.V | Drift re-verification of 6.4/6.5/6.8 with DONE.md reports | QA | 1.A | ✅ | Verified 2026-07-15: All three items properly implemented, IPC-wired, and passing G0–G5 gates. Completion report filed in session record. |

## 4. Wave 4 — AgentKit Runtime ✅

| Item | Task | Owner | Deps | Status | Notes |
|------|------|-------|------|:------:|-------|
| 4.1 | Agent state machine (idle → thinking → tool → done) | Architect | — | ✅ | `agent/state.rs`, `agent/context.rs`, `agent/agent.rs` all implemented |
| 4.2 | Tool execution bridge (Rust → Docker exec via bollard) | Architect | 4.1 | ✅ | `bridge_client.rs` with `ToolExecBridge` |
| 4.3 | Persistent PTY shell sessions | Architect | 4.2 | ✅ | `pty/mod.rs` with session management |
| 4.4 | Agent state persistence across app restarts | Architect | 4.1 | ✅ | `agent/persistence.rs` with save/load/clear commands |
| 4.5 | MCP server discovery per ADR-006 | Architect | 4.2 | ✅ | `mcp/mod.rs` with registry + discovery |
| 4.6 | Runtime test suite: state transitions + bridge failure paths | QA | 4.2 | ✅ | 104 tests passing across all modules |

## 5. Wave 7 — Security Hardening & HITL 🟡

| Item | Task | Owner | Deps | Status | Notes |
|------|------|-------|------|:------:|-------|
| 7.1 | HITL confirmation flow for dangerous operations (P0) | Frontend + Security | — | ✅ | `hitl.rs` with session management + IPC commands |
| 7.2 | Air-gapped mode (no-network container profile) | Security | — | ✅ | `container.rs` `set_sandbox_network_mode` command |
| 7.3 | Threat model refresh vs. shipped Wave 4 surface | Security | 4.x | ✅ | Refresh completed 2026-07-15: existing model already accurate; all ❌ controls confirmed still unimplemented; document history and date updated |
| 7.4 | Secret-scan CI step (gitleaks) | DevOps | 1.D | ✅ | `secret-scan.yml` already configured with `gitleaks/gitleaks-action@v2` on push/PR to main. `gitleaks.toml` allowlist in place. Added local gitleaks step to `build-release.ps1`. |
| 7.5 | Capabilities re-audit (minimal perms vs. actual usage) | Security | 4.x | ✅ | Audit complete: 5 plugin permissions match actual usage (core, shell:allow-open, dialog:allow-open, updater). ~30 custom commands implicitly available to single main window — no scoping needed for current architecture. Minimal and correct. |

## 6. Wave 8 — Distribution & Release 🟡

| Item | Task | Owner | Deps | Status | Notes |
|------|------|-------|------|:------:|-------|
| 8.1 | Windows packaging (MSI/NSIS via Tauri bundler) | DevOps | 7.x | ✅ | Already configured in `tauri.conf.json`: `bundle.targets: ["msi", "nsis"]` with WiX and NSIS options. Release workflow created at `.github/workflows/release.yml`. |
| 8.2 | Code signing pipeline | DevOps | 8.1 | 🔲 | Requires code signing certificate (DigiCert/Sectigo). Set `TAURI_SIGNING_PRIVATE_KEY` + `TAURI_SIGNING_PASSPHRASE` secrets in GitHub repo. Uncomment signing step in `release.yml` when ready. |
| 8.3 | Tauri self-updater + release channel | DevOps | 8.2 | 🟡 | Updater configured with endpoint URL and placeholder pubkey. Real key needs generation via `cargo tauri sign --generate-keys`. Release workflow creates update artifacts. |
| 8.4 | Ring 2 (Windows) authoritative full-gate release run | QA + DevOps | 8.1 | 🔲 | Requires Windows VM/hardware for full runtime validation. Not yet scheduled. |

## 7. Wave 9 — Rig Core + Cleanup ✅

> All items verified complete during 2026-07-15 session.

| Item | Task | Owner | Deps | Status | Notes |
|------|------|-------|------|:------:|-------|
| 9.1 | Audit remaining custom serde/LLM code for Rig migration | Architect | — | ✅ | `agent.rs` uses `rig::providers::deepseek::Client` + `rig::providers::openai::Client` exclusively. No direct `reqwest` LLM calls. Streaming uses `rig::completion::CompletionModel::stream()`. |
| 9.2 | Replace hand-rolled SSE streaming with Rig's native streaming | Architect | 9.1 | ✅ | `think_with_streaming()` uses `rig::streaming::StreamedAssistantContent` with `completion_request(...).stream().await`. Full coverage verified. |
| 9.3 | Remove `reqwest` if no longer needed directly | Architect | 9.2 | ✅ N/A | `reqwest` still needed for `health_check.rs` (LLM provider health probes) and `mcp/mod.rs` (MCP server discovery). Both are non-LLM HTTP calls outside Rig's scope. |
| 9.4 | Delete dead `env/mod.rs` module | Architect | — | ✅ | Removed from git index (was orphaned — no `mod env;` in lib.rs) |
| 9.5 | Fix `.gitattributes` — add `* text=auto eol=lf` | Architect | — | ✅ | Already present: `* text=auto eol=lf` on line 2. Binary entries also declared. |
| 9.6 | Convert 4 CRLF files to LF | Architect | 9.5 | ✅ | BUILD_PLAN.md (0 CRLF), MEMORY.md (0 CRLF), DECISIONS.md (0 CRLF), SESSION_PROTOCOL.md (0 CRLF) — all already LF. |
| 9.7 | `pnpm audit` triage — record false-positive assessment in DECISIONS.md | Security | — | ✅ | DEC-009 recorded 2026-07-15: 4 vite 5.x advisories are false positives. |
| 9.8 | Verify CodeMirrorInput + Shiki highlighter integration | Frontend | — | ✅ | CodeMirrorInput exported from `components/index.ts` (line 42), imported in `App.tsx` (line 13). `useShikiHighlighter` hook exists at `src/hooks/useShikiHighlighter.ts`, consumed by `CodeBlock.tsx` (line 2). |

## 8. Housekeeping backlog

| Item | Task | Owner | Status |
|------|------|-------|:------:|
| H.1 | ADR numbering verified — all unique (ADR-001 through ADR-011); removed stale warning | Doc Engineer | ✅ |
| H.2 | Convert CRLF→LF on 4 files: BUILD_PLAN.md, MEMORY.md, DECISIONS.md, SESSION_PROTOCOL.md | Architect | ✅ |
| H.3 | GLOSSARY.md: add Batch Loop, STOP/ASK, ROUTE line, WIP limit terms | Doc Engineer | ✅ |
| H.4 | Validate Ring 1 ↔ Ring 2 bridge (Docker + mount + chat + IPC serialization) | Architect | ✅ |
| H.5 | Remove dead code: `src-tauri/src/env/mod.rs` is orphaned (no `mod env;` in lib.rs) | Architect | ✅ |
| H.6 | Expand `.gitattributes` — add `* text=auto eol=lf` rule | Architect | ✅ |
| H.7 | `pnpm audit` triage — 4 advisories (false positives for vite 5.x); record in DECISIONS.md | Security | ✅ |

---

*Maintained by the Orchestrator. Full-file rewrites only.*
