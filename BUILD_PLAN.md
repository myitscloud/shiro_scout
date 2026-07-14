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
| 1 | Scaffold & Toolchain | 🟡 | Closeout items only (see below) |
| 2 | Scaffold & Toolchain (initial pass) | ✅ | |
| 3 | Docker Orchestration | ✅ | Sandbox + axum bridge live |
| 4 | AgentKit Runtime | 🔲 | **Next major wave** |
| 5 | Core UI — Design System & Components | ✅ | 14 components shipped |
| 6 | LLM Integration | 🟡 | Only 6.7 streaming open (+ drift re-verify) |
| 7 | Security Hardening & HITL | 🔲 | |
| 8 | Distribution & Release | 🔲 | |

**Current priority order:** Wave 1 closeout → Wave 6.7 → Wave 4 → Wave 7 → Wave 8.

## 2. Wave 1 — Scaffold & Toolchain closeout 🟡

| Item | Task | Owner | Deps | Status |
|------|------|-------|------|:------:|
| 1.A | Baseline: run full DONE.md gate sequence on current tree, record report | QA | — | 🔲 |
| 1.B | `.gitattributes` (`* text=auto eol=lf` + binary entries); convert stray CRLF files | Architect | — | 🔲 |
| 1.C | Complete `cargo-deny` config (licenses, advisories, bans); add to gates | Security | 1.A | 🔲 |
| 1.D | `git init`, initial commit, GitHub remote, push | DevOps | 1.B | 🔲 |
| 1.E | Tauri shell IPC completion check — every stub in `lib.rs` implemented or `// BLOCKED: BLK-n` | Architect | 1.A | 🔲 |

## 3. Wave 6 — LLM Integration 🟡

| Item | Task | Owner | Deps | Status |
|------|------|-------|------|:------:|
| 6.1–6.3 | Provider crates (async-openai, DeepSeek, OpenAI) | Architect | — | ✅ |
| 6.4 | Settings > LLM Providers UI (3-role pattern) | Frontend | — | ✅ (re-verify 6.V) |
| 6.5 | API key management (Windows Credential Manager via keyring) | Architect | — | ✅ (re-verify 6.V) |
| 6.6 | Token usage tracking + cost estimation | Frontend | — | ✅ |
| 6.7 | Streaming responses: Rust `emit` → IPC events → StreamingText | Frontend (+Architect, C3) | 1.A | 🔄 |
| 6.8 | Provider health check + failover | QA + Reviewer | — | ✅ (re-verify 6.V) |
| 6.V | Drift re-verification of 6.4/6.5/6.8 with DONE.md reports | QA | 1.A | 🔲 |

## 4. Wave 4 — AgentKit Runtime 🔲

| Item | Task | Owner | Deps | Status |
|------|------|-------|------|:------:|
| 4.1 | Agent state machine (idle → thinking → tool → done) — finish from Wave 0 | Architect | 6.7 | 🔲 |
| 4.2 | Tool execution bridge (Rust → Docker exec via bollard) | Architect | 4.1 | 🔲 |
| 4.3 | Persistent PTY shell sessions (see docs/true-state-preservation.md) | Architect | 4.2 | 🔲 |
| 4.4 | Agent state persistence across app restarts | Architect | 4.1 | 🔲 |
| 4.5 | MCP server discovery per ADR-006 | Architect | 4.2 | 🔲 |
| 4.6 | Runtime test suite: state transitions + bridge failure paths | QA | 4.2 | 🔲 |

## 5. Wave 7 — Security Hardening & HITL 🔲

| Item | Task | Owner | Deps | Status |
|------|------|-------|------|:------:|
| 7.1 | HITL confirmation flow for dangerous operations (P0) | Frontend + Security | 4.2 | 🔲 |
| 7.2 | Air-gapped mode (no-network container profile) | Security | 4.2 | 🔲 |
| 7.3 | Threat model refresh vs. shipped Wave 4 surface | Security | 4.x | 🔲 |
| 7.4 | Secret-scan CI step (gitleaks) | DevOps | 1.D | 🔲 |
| 7.5 | Capabilities re-audit (minimal perms vs. actual usage) | Security | 4.x | 🔲 |

## 6. Wave 8 — Distribution & Release 🔲

| Item | Task | Owner | Deps | Status |
|------|------|-------|------|:------:|
| 8.1 | Windows packaging (MSI/NSIS via Tauri bundler) | DevOps | 7.x | 🔲 |
| 8.2 | Code signing pipeline | DevOps | 8.1 | 🔲 |
| 8.3 | Tauri self-updater + release channel | DevOps | 8.2 | 🔲 |
| 8.4 | ARM64 target build + smoke test | DevOps | 8.1 | 🔲 |
| 8.5 | Ring 2 (Windows) authoritative full-gate release run | QA + DevOps | 8.1 | 🔲 |

## 7. Housekeeping backlog (schedule into any batch with spare slots)

| Item | Task | Owner | Status |
|------|------|-------|:------:|
| H.1 | De-duplicate ADR numbering (two ADR-001s, ADR-002s, etc. in docs/adr/) — renumber files, keep content, add index | Doc Engineer | 🔲 |
| H.2 | Convert all CRLF markdown to LF after 1.B lands | Architect | 🔲 |
| H.3 | GLOSSARY.md: add Batch Loop, STOP/ASK, ROUTE line, WIP limit terms | Doc Engineer | 🔲 |
| H.4 | Validate Ring 1 ↔ Ring 2 bridge (Docker + mount + chat + IPC serialization) | Architect | ✅ VERIFIED |

---

*Maintained by the Orchestrator. Full-file rewrites only.*
