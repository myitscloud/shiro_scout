# Definition of Done — DONE.md

> **Purpose:** Verification gates, Tauri IPC wiring checks, blocker protocol, completion report template. Every task passes these before ✅.
> **v2 changes (2026-07-10):** dual-environment gate table (Ring 1 container / Ring 2 Windows), pnpm standardized. Rule IDs unchanged.

---

## 1. Environment Assumptions

Gates run **where the toolchain lives**:

| | Ring 1 — Linux container (every change set) | Ring 2 — Windows 11 host (authoritative, pre-release + runtime claims) |
|---|---|---|
| Shell | bash — chain with `&&`, exit code via `$?` | PowerShell 7 — chain with `&&`, exit code via `$LASTEXITCODE` |
| Rust | `cargo check/clippy --target x86_64-pc-windows-msvc` + `cargo test` (host-runnable tests) | full `cargo test` + runtime behavior |
| Layout | Frontend at repo root (`package.json`, `src/`); Rust at `src-tauri/` | same |
| Package manager | **pnpm** | **pnpm** |

Anything only provable on Windows goes under **UNVERIFIED-RUNTIME** (DONE-042) when reported from Ring 1.

## 2. Verification Gates — run in order, stop at first failure (DONE-010 … DONE-016)

| Gate | ID | Rule |
|:----:|:--:|------|
| 0 | **DONE-010** | **Stub scan** — regex over every changed file (case-insensitive): `todo!()\|unimplemented!()\|not.?implemented\|\bTODO\b\|\bFIXME\b\|\bHACK\b`. Zero new matches outside `tests/` and docs. Record the count. |
| 1 | **DONE-011** | **Types** — `npx tsc --noEmit` → exit 0 |
| 2 | **DONE-012** | **Frontend build** — `pnpm build` → exit 0 |
| 3 | **DONE-013** | **Rust check** — `cargo check --manifest-path src-tauri/Cargo.toml` (Ring 1 adds `--target x86_64-pc-windows-msvc`) → exit 0 |
| 4 | **DONE-014** | **Lint** — `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` → exit 0 |
| 4.5 | **DONE-014b** | **Supply chain** — `cargo deny check` + `pnpm audit` → exit 0 (advisories triaged, not ignored) |
| 5 | **DONE-015** | **Tests** — `cargo test --manifest-path src-tauri/Cargo.toml` → exit 0 if a suite exists; else record "no test suite". Never delete, weaken, `#[ignore]`, or rewrite assertions to force a pass. |
| 6 | **DONE-016** | **Exit codes** — record the code after each gate (`$?` / `$LASTEXITCODE`). A gate without a recorded exit code did not happen. |

### Gate Enforcement Rules

| ID | Rule |
|:--:|------|
| DONE-001 | DONE means: every gate ran **after the final edit** and exited 0, every wiring check is search-verified, and the DONE-040 report is included. Anything less is not done. |
| DONE-002 | Command output is the only proof of state. Inspection, expectation, or memory of an earlier run proves nothing. |
| DONE-003 | Any edit voids all earlier gate results. Re-run from Gate 0. |
| DONE-004 | Scope is frontend + backend + wiring. One side only is PARTIAL. |
| DONE-005 | Stubs are forbidden in delivered work unless tagged `// BLOCKED: BLK-n`. |
| DONE-006 | A todo item flips ✅ only after its acceptance check ran green **this session**. |

## 3. Tauri IPC Wiring — search-verified, never assumed (DONE-020 … DONE-027)

| ID | Rule |
|:--:|------|
| DONE-020 | Every new/renamed `#[tauri::command]` appears in `generate_handler![…]`. Verify by search. |
| DONE-021 | Every frontend `invoke("name")` string exactly matches a registered command. Both compilers pass while this is broken — only search proves it. |
| DONE-022 | Argument casing: Tauri 2 exposes Rust `snake_case` args as `camelCase` to JS. Per command: pass `camelCase` from TS or declare `#[tauri::command(rename_all = "snake_case")]`. |
| DONE-023 | Type parity: serde struct ↔ TS interface — names, optionality, nesting. |
| DONE-024 | Every `State<T>` parameter has a matching `.manage(…)` on the builder. |
| DONE-025 | Event parity: every `emit("x")` has a `listen("x")` or a documented external consumer. |
| DONE-026 | New plugin/core-API usage has a matching entry in `src-tauri/capabilities/`. |
| DONE-027 | Wiring answers are YES / NO / N-A, each backed by a search result. "Should match" is a NO. |

## 4. Blocker Protocol (DONE-030 … DONE-034)

| ID | Rule |
|:--:|------|
| DONE-030 | After two materially different approaches fail, STOP retrying. Convert to BLOCKED, continue with remaining objectives. (Fires SESSION_PROTOCOL STOP-4.) |
| DONE-031 | A blocker entry contains: `BLK-n`; the objective; exact error output; approaches tried; hypothesis; what unblocks it. |
| DONE-032 | Any stub left behind compiles safely and carries `// BLOCKED: BLK-n`. |
| DONE-033 | BLOCKED ≠ DONE. Status becomes `PARTIAL (n BLOCKED)`; gates still pass on everything else. |
| DONE-034 | Never silently drop an objective. Every plan item ends COMPLETE or BLOCKED. |

## 5. Completion Report Template (DONE-040 … DONE-044)

| ID | Rule |
|:--:|------|
| DONE-040 | `attempt_completion` includes the template below, fully filled. No line omitted. |
| DONE-041 | Banned for functional claims: "should work", "should now", "likely", "probably", "I believe". Every claim cites evidence. |
| DONE-042 | Behavior no Ring-1 gate can prove (WMI calls, window management, registry/filesystem effects) is listed under UNVERIFIED-RUNTIME. |
| DONE-043 | Report new compiler/linter warnings. Target: 0. |
| DONE-044 | List every changed file with a one-line purpose. |

```
## COMPLETION REPORT
Status: COMPLETE | PARTIAL (n BLOCKED)
Ring: 1 (container) | 2 (Windows)

Gates
- G0 stub scan: <n> new matches
- G1 tsc --noEmit: exit <code>
- G2 pnpm build: exit <code>
- G3 cargo check [--target x86_64-pc-windows-msvc]: exit <code>
- G4 cargo clippy -D warnings: exit <code>
- G4.5 cargo deny + pnpm audit: exit <code>
- G5 cargo test: <n> passed / <n> failed | no test suite

Wiring
- generate_handler registration: YES/NO/N-A — commands: <list>
- invoke ↔ command pairs: <"name" ↔ fn_name, …>
- argument casing per command: <camelCase | rename_all snake_case>
- type parity: <RustStruct ↔ TsInterface, …>
- State .manage(): YES/N-A
- event emit/listen parity: YES/N-A
- capabilities entries: YES/N-A

Files changed
- <path> — <purpose>

Dependencies added
- none | <name@version — reason (C12 approval ref)>

Blockers
- none | BLK-1: <one-line summary>

UNVERIFIED-RUNTIME
- none | <item — why Ring-1 gates cannot prove it>

Warnings delta
- 0 new | <list>
```

## 6. Scope Control (DONE-050 … DONE-052)

| ID | Rule |
|:--:|------|
| DONE-050 | Touch only files the plan names plus files strictly required to compile and wire. Extras must be justified in the report. |
| DONE-051 | No new dependencies without a report entry (name, version, reason) and C12 approval. |
| DONE-052 | No drive-by refactors, renames, or formatting churn. Never "fix" a gate failure by rewriting unrelated code. |

---

*Maintained by the Documentation Engineer. Update when gates change.*
