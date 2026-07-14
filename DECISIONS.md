# DECISIONS.md — Append-Only Decision Log

> **Format (T5):** newest first · each entry ≤ 12 lines · `DEC-nnn · date · title` then **Context** (why it came up), **Decision** (what was chosen), **Consequences** (what changes / trade-offs), **Links**. Never edit past entries — supersede with a new one.

---

## DEC-008 · 2026-07-14 · Docker mount point + IPC schema fixed
**Context:** After workspace overhaul, LLM agent produced `missing field 'tool_call_id'` deserialization errors. Two custom code paths — `llm/mod.rs:151` (Tauri command) and `agent/agent.rs:195` (agent loop) — used `serde_json::json!()` which silently dropped `tool_call_id` and `name` fields from messages sent to DeepSeek API, violating its tool-call chain validation.
**Decision:** Replace `serde_json::json!()` with `serde_json::Map` + conditional field insertion in both code paths. Add keychain fallback for API key resolution in `think_with_streaming()`. Normalize non-standard roles (`'warning'`) to `'system'` in agent loop. Change `add_tool()` from `'tool'` to `'assistant'` role in history.rs. All changes compiled with 0 errors.
**Consequences:** Chat pipeline fully functional; Ring 1 ↔ Ring 2 bridge validated; agent can execute bash commands in Docker container and return output to Tauri. Automated backup via powershell/robocopy to X: drive configured. Git pushed to `github.com/myitscloud/shiro_scout.git`.
**Links:** llm/mod.rs, agent/agent.rs, history.rs, scripts/auto-backup.ps1

## DEC-005 · 2026-07-10 · BUILD_PLAN.md is full-rewrite-only
**Context:** BUILD_PLAN.md was found corrupted — one table row duplicated ~250 times, consistent with the PowerShell `Set-Content`/append failures cataloged in FILEOPS §4.
**Decision:** BUILD_PLAN.md stays ≤ 250 lines and is only ever updated by full-file `write` (FILEOPS-001). No sed loops, no appends, no PowerShell.
**Consequences:** Slightly more tokens per update; zero risk of incremental corruption. File rebuilt from FEATURES.md + MEMORY.md as of today.
**Links:** FILEOPS-001, FILEOPS §4, BUILD_PLAN.md header.

## DEC-004 · 2026-07-10 · pnpm standardized across all docs and gates
**Context:** MEMORY.md locked pnpm; DONE.md gates said npm. Agents alternated, producing mixed lockfile states.
**Decision:** pnpm everywhere — gates G2/G4.5 use `pnpm build` / `pnpm audit`.
**Consequences:** DONE.md, AGENTS.md, KICKOFF updated; any `package-lock.json` is deleted when found.
**Links:** MEMORY §3 constraint 9, DONE.md §1.

## DEC-003 · 2026-07-10 · WIP limit: max 2 concurrent subordinates
**Context:** Hardware supports ~4 concurrent agents before failures; the Orchestrator + `parallel` tool overhead consume headroom.
**Decision:** O12 — max 2 subordinates at once, sequential by default, concurrency only for disjoint-directory items (O13).
**Consequences:** Slower wall-clock on independent items; far fewer crashed runs and merge collisions. Revisit if hardware upgrades.
**Links:** AGENTS.md O12/O13, SESSION_PROTOCOL §4.

## DEC-002 · 2026-07-10 · AGENTS.md consolidates charter + role cards + routing
**Context:** Delegation required loading a separate orchestrator.md; routing rules were buried, and Agent 0 defaulted to doing work itself.
**Decision:** Single AGENTS.md holds the charter, new O10–O16 delegation rules, the Routing Table, and nine condensed role cards. Full per-role profiles in `docs/agent-profiles/` remain authoritative for role detail.
**Consequences:** Every subordinate brief points at one file; routing is enforceable (O11 ROUTE line). Charter still wins all conflicts.
**Links:** AGENTS.md §7–§9, KICKOFF §5.

## DEC-001 · 2026-07-10 · Linux-first file operations; PowerShell demoted to orchestration-only
**Context:** Repeated PowerShell content-write failures (newline stripping, Base64 corruption, silent regex no-ops); the model intuitively emits Linux commands; primary editing happens inside the Linux container anyway.
**Decision:** FILEOPS.md rewritten Linux-first: container `text_editor`/`sed`/heredoc are primary; `text_editor_remote` + Git Bash for the Windows host; PowerShell allowed only for orchestration (builds, git status, docker ps).
**Consequences:** Existing FILEOPS-001…030 IDs retained; FILEOPS-040…044 appended for Linux specifics. Supersedes the PowerShell-era priority table.
**Links:** FILEOPS.md, MEMORY §7.
