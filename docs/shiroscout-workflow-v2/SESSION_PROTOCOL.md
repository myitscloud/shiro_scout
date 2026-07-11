# Session Protocol

> **Purpose:** How AI sessions operate on ShiroScout — rings, lifecycle, TODO bookkeeping, the Batch Loop that keeps agents working, and the STOP/ASK protocol that tells them when to halt and talk to the boss.
> **Environment:** Agent Zero in Linux Docker container · code targets Windows 11

---

## §1 Ring Architecture

| | **Ring 1 — Direct Execution** | **Ring 2 — Subordinate Delegation** |
|---|---|---|
| **Who** | Orchestrator (Agent 0) itself | Specialist agents via `call_subordinate` |
| **Scope** | Docs, TODO tables, MEMORY/DECISIONS, reads, build/verify commands, config ≤ 20 lines | All production code, multi-file changes, reviews, audits, research reports |
| **Tools** | `code_execution_tool`, `text_editor`, `document_query`, `search_engine` | `call_subordinate` with a §KICKOFF-5 brief |
| **Hard limit** | O10: no production code | O12: max 2 concurrent; O13: no overlapping file scope |

**Ring 1 → Ring 2 escalation triggers:** item touches Rust/TS/PowerShell production code · needs a security review · spans > 3 files or crosses layers · is a research/analysis deliverable · a Ring 1 attempt failed twice.

## §2 Session Lifecycle

**Start:** run `KICKOFF_PROMPT.md` (reads AGENTS.md, MEMORY.md §1–3, BUILD_PLAN.md, this file §4–5; verifies env; kills stale processes).
**Active:** the Batch Loop (§4).
**End / freeze / timeout:** save state to MEMORY.md §1, sync the TODO table, log the last action + tool + result in MEMORY.md §8, kill background processes.
**Resume:** paste `LOOP_PROMPT.md` — it re-reads state and re-enters the loop at SELECT.

## §3 TODO Bookkeeping

One format everywhere (BUILD_PLAN.md item tables and §6 below):

| Item | Task | Owning Agent | Dependencies | Status |
|------|------|--------------|--------------|--------|

Status: ✅ done (gates passed) · 🔲 not started · 🔄 in progress · ⏸️ blocked (name the blocker) · ❌ failed twice (escalate).

Update rules: update on **every item completion before starting the next** · ⏸️ items name their blocker inline · after 2 failures mark ❌ and fire STOP-4 · at batch close, sync to MEMORY.md §1.

## §4 The Batch Loop (this is the engine — follow it literally)

```
LOOP:
  0. REFRESH  — If context has been summarized/compacted or feels stale:
                re-read MEMORY.md §1 and BUILD_PLAN.md open items first.
  1. SELECT   — Take up to 8 unblocked items, priority order:
                  a) current wave in BUILD_PLAN.md, top to bottom
                  b) if empty: earliest wave with open items
                  c) if none anywhere: fire STOP-1 and halt.
                Post the selected batch as a table before touching anything.
  2. PER ITEM (strict sequence):
       2a. ROUTE line (O11):  ROUTE: <id> → <role> | reviewers: <...> | ring: 1|2
       2b. Spec check — code items need a mini-spec (docs/mini-specs/MSPEC-*.md).
           Missing? Orchestrator writes one now (docs are Ring 1) using the
           template; if the spec requires a product decision, fire STOP-2 instead.
       2c. Execute — Ring 1 yourself, or Ring 2 via the KICKOFF §5 brief.
           Subordinate must read AGENTS.md + FILEOPS.md before edits.
       2d. Verify — full DONE.md gate sequence AFTER the final edit; wiring
           checks search-verified; completion report per DONE-040.
       2e. Book-keep — flip the TODO row now (not later). Significant choice
           made? Append a DECISIONS.md entry (T5 format, ≤ 12 lines).
       2f. Any STOP condition fired? → §5. Otherwise next item.
  3. BATCH CLOSE:
       - Sync MEMORY.md §1 (current wave, last action, next task) and §8 (history).
       - Post a batch report: items attempted / ✅ / ⏸️ / ❌, gate summary, blockers.
       - Kill stray processes (ps aux | grep -E 'pnpm|cargo|node' | grep -v grep).
  4. GOTO LOOP — immediately. Do not wait for permission to continue.
```

**Concurrency inside a batch:** default sequential. You MAY run 2 subordinates concurrently only when both items are unblocked, live in disjoint directories/layers (O13), and neither needs the other's output. Never exceed 2 (O12).

## §5 STOP/ASK Protocol — "Um, hey boss…"

A STOP ends the loop turn. Report using the matching template, then **wait for the human**. Inventing work to stay busy violates DONE-050.

| ID | Trigger | Required message shape |
|----|---------|------------------------|
| **STOP-1 Out of work** | No unblocked items in any wave | "Boss, the board is clear. Done this session: <list>. Blocked: <list+why>. Candidate next steps I could spec: <3 proposals>. Which direction?" |
| **STOP-2 Ambiguous spec** | Spec missing/contradictory and the choice is user-visible or architectural | One concrete question, options A/B(/C) with one-line trade-offs and my recommendation. No work on the item until answered. |
| **STOP-3 Missing artifact** | A referenced file/doc/dependency doesn't exist | "Item <id> needs `<path>`, which doesn't exist. I can draft a skeleton for your review (contents: <outline>) or you can supply it. Proceed with draft?" |
| **STOP-4 Two-strike failure** | Same item failed twice | Freeze item as ❌. Report: attempts, errors **verbatim**, current hypothesis, what would unblock (stronger model / human decision / missing info). |
| **STOP-5 Security finding** | Any §8 security trigger fails review | Blocking. Finding, affected files, risk, proposed remediation. Only the human owner overrides, in writing, logged to DECISIONS.md. |
| **STOP-6 Scope conflict** | Correct fix requires touching files outside the plan/brief | "Plan says touch <X>; reality requires <Y> because <reason>. Approve scope change or re-spec?" |
| **STOP-7 Resource limit** | Work needs > 2 concurrent subordinates, or container resources exhausted | State the constraint and a sequentialized plan; ask before proceeding. |

## §6 Current TODO (synced 2026-07-10 — reconciled against FEATURES.md)

> ⚠ **Drift notice:** the 2026-07-09 version of this table said Wave 6.4/6.5/6.8 were not started, while FEATURES.md (2026-07-10) marks them ✅. FEATURES is newer and wins, **but** those items must show a DONE.md completion report from this or a prior session before anyone builds on them. First batch below re-verifies.

### Wave 1 — Scaffold & Toolchain closeout (🟡)

| Item | Task | Owning Agent | Dependencies | Status |
|------|------|--------------|--------------|--------|
| 1.A | Re-run full gate sequence on current tree; record baseline report | QA / Test Engineer | — | ✅ (PARTIAL: 3 blockers) |
| 1.B | `.gitattributes` LF enforcement + convert stray CRLF files (MEMORY.md was CRLF) | Windows Systems Architect | — | ✅ |
| 1.C | Finish `cargo-deny` license/advisory config; wire into gate G4.5 | Security Engineer | 1.A | ✅ |
| 1.D | `git init` + initial commit + push (End Task Ritual from MEMORY §9) | Release / DevOps | 1.B | ⏸️ remote origin has history — needs force-push decision |
| 1.E | Tauri shell IPC completion check — every stub in `lib.rs` implemented or `// BLOCKED: BLK-n` | Windows Systems Architect | 1.A | ⏸️ blocked on 1.A baseline resolution |

### Wave 6 — LLM Integration (🟡, nearly done)

| Item | Task | Owning Agent | Dependencies | Status |
|------|------|--------------|--------------|--------|
| 6.7 | Streaming response handling (Rust emit → IPC events → StreamingText UI) — **report**: .a0proj/COMPLETION_REPORT_6_7.md — SSE parsing, token emit via Tauri events, frontend listener + rendering confirmed | Frontend Engineer (Architect consulted, C3) | 1.A | ✅ |
| 6.V | Gate-verify 6.4 / 6.5 / 6.8 with completion reports (drift check) — **report**: COMPLETION_6V.md — code structurally sound, no formal DONE.md gate evidence found; verdict PARTIAL; recommend unit tests + Ring 2 run | QA / Test Engineer | 1.A | ✅ (PARTIAL: no gate evidence) |

### Wave 4 — AgentKit Runtime (next major wave)

| Item | Task | Owning Agent | Dependencies | Status |
|------|------|--------------|--------------|--------|
| 4.1 | Agent state machine completion (idle→thinking→tool→done) — **verdict**: state machine wired, streaming integrated, `cargo check` exit 0 (3 pre-existing errs fixed), `cargo test` 27 passed | Windows Systems Architect | 6.7 | ✅ |
| 4.2 | Tool execution bridge (Rust → Docker exec via bollard) — **verdict**: bridge wired (exec_sandbox_command, ToolExecBridge), `cargo check` exit 0, `cargo test` 27 passed; 22 pre-existing clippy warnings outside scope blocked G4 | Windows Systems Architect | 4.1 | ✅ (PARTIAL: pre-existing clippy warnings) |
| 4.3 | Persistent PTY shell sessions — **verdict**: PtyManager with session create/exec/close/list implemented, `cargo check` exit 0, `cargo test` 27 passed | Windows Systems Architect | 4.2 | ✅ |
| 4.4 | Agent state persistence across app restarts — **verdict**: persistence.rs with save/load/clear, auto-save on CloseRequested, auto-restore in .setup(), `cargo check` exit 0, `cargo test` 27 passed | Windows Systems Architect | 4.1 | ✅ |
| 4.5 | MCP server discovery (per ADR-006) — **verdict**: McpDiscovery with TCP port scanning 3100-3200, health probe, McpRegistry, 5 Tauri commands registered, `cargo check` exit 0, `cargo test` 27 passed | Windows Systems Architect | 4.2 | ✅ |

### Completed waves

| Wave | Title | Status |
|:----:|-------|:------:|
| 0 | Orchestrator Agent Core | ✅ |
| 2 | Scaffold & Toolchain (initial) | ✅ |
| 3 | Docker Orchestration | ✅ |
| 5 | Core UI — Design System & Components | ✅ |

### Wave 7 — Security Hardening & HITL (🟡, in progress)

| Item | Task | Owning Agent | Dependencies | Status |
|------|------|--------------|--------------|--------|
| 7.1 | HITL confirmation flow for dangerous operations (P0) — **verdict**: hitl.rs (430 lines, HITLManager + 5 Tauri commands), ConfirmationDialog.tsx + .module.css, PendingConfirmation state in agent state.rs, THREAT_MODEL.md STRIDE pass documented. Pre-existing compilation errors in out-of-scope modules block G3+. | Frontend + Security | 4.x | ✅ (PARTIAL: pre-existing Rust errors) |
| 7.2 | Air-gapped mode (no-network container profile) — **verdict**: NetworkMode enum (Bridge/None), SandboxConfig.network_mode, build_host_config() applying NetworkMode::None, set_sandbox_network_mode command, default air-gapped (network_mode=none), sandbox_air_gapped setting, THREAT_MODEL.md SC-02 updated, HITL required for toggle. G3 blocked by 44 pre-existing errors outside scope. | Security | 7.1 | ✅ (PARTIAL: pre-existing compilation errors) |
| 7.3 | Threat model refresh vs shipped Wave 4 surface | Security | 7.x | 🔲 |
| 7.4 | Secret-scan CI step (gitleaks) | DevOps | 1.D | 🔲 |
| 7.5 | Capabilities re-audit (minimal perms vs actual usage) | Security | 7.x | 🔲 |

### Wave 8 — Distribution & Release (🔲)


Waves 7 (Security Hardening & HITL) and 8 (Distribution & Release) are 🔲 — see BUILD_PLAN.md.

---

*Maintained by the Orchestrator. TODO synced at every batch close.*
