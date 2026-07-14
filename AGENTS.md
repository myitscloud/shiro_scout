# ShiroScout — AGENTS.md (Charter + Team)

> **Root charter for all agents.** Role cards below are the portable summary of the team; if a full profile exists at `docs/agent-profiles/<role>.md` it remains authoritative for that role's detailed ruleset. **If anything conflicts, this charter wins — flag it, don't resolve it silently.**

---

## 1. Project Identity

| Field | Value |
|-------|-------|
| **Project** | ShiroScout (codename: Project Aegis) |
| **Design Language** | Neo-Glass Terminus |
| **Stack** | Tauri 2 / Rust / React 18 / TypeScript / Vite / PowerShell 7 |
| **Target** | Windows 11 (x64, ARM64) |
| **Dev Environment** | Agent Zero in Linux (Kali) Docker container — production targets Windows via cross-compilation |
| **Package manager** | pnpm (standardized — see DEC-004) |

## 2. Prime Directives

1. **Security is blocking.** The Security Engineer's rejection is overridden only by the human owner in writing, recorded in `DECISIONS.md`.
2. **One source of truth.** Link to authoritative files; never copy normative text across documents.
3. **No code crosses platform to silence errors.** The container is Linux; the target is Windows 11. In-container checks use `--target x86_64-pc-windows-msvc` and static analysis only.
4. **Test the failure paths.** Every fallible operation has a typed, sanitized, user-visible error path.
5. **Every behavior change updates docs in the same change set.** "Docs later" is a Request Changes.
6. **Delegation is the default, not the exception.** The Orchestrator routes; specialists implement. (See O-rules.)

## 3. Layer Architecture

```
 WebView (React/TypeScript)
      ↓ IPC (typed invoke)
 Rust Backend (Tauri commands, bollard Docker API)
      ↓ HTTP/WebSocket bridge
 Docker Sandbox (AgentKit runtime)
      ↓ WinRM / CIM
 Remote Windows Targets
```

One agent per layer per change set (C3). IPC contract changes are owned by the Windows Systems Architect with the Frontend Engineer consulted.

## 4. Task Pipeline (fixed order)

```
Intake → ROUTE line → Mini-spec → Implement (owning agent) → Self-verify (DONE.md gates)
      → QA review → Security review (only if a trigger fired)
      → Code review → Docs check → Merge-ready → TODO ✅
```

## 5. Rule Numbering Convention

| Prefix | Owner | Prefix | Owner |
|--------|-------|--------|-------|
| C | Charter (cross-cutting) | Q | QA / Test Engineer |
| O | Orchestrator / Tech Lead | R | Code Reviewer |
| W | Windows Systems Architect | D | Release / DevOps |
| F | Frontend Engineer | T | Documentation Engineer |
| S | Security Engineer | A | Accessibility & UX |

Rules are stable — **never renumber. Deprecate and append.**

## 6. Cross-Cutting Rules (C-rules)

| # | Rule |
|---|------|
| **C1** | User-facing errors state: what happened → why (best known) → what to do next. Error codes appended. |
| **C2** | No secrets in logs, error strings, telemetry, or frontend state. |
| **C3** | Three-layer contract sync: an IPC change updates Rust → TypeScript → docs in one change set. |
| **C4** | Every API symbol new to the repo needs evidence: prior-art grep hit, pinned docs citation, or an explicit `VERIFY:` marker. Uncited novel symbols are blockers. |
| **C5** | Dependency upgrades are planned migrations, never casual bumps. `cargo audit` / `cargo deny` / `pnpm audit` run every change set. |
| **C6** | Every path from frontend-controlled input to process spawn, file path, registry key, or query is validated on the receiving side. |
| **C7** | Every fallible command returns typed errors, sanitized before crossing IPC. |
| **C8** | Dark mode is the default identity. Light mode is an accessibility toggle. |
| **C9** | Assume no admin rights, no English locale, no MAX_PATH, no domain join. Degrade gracefully. |
| **C10** | Sanitize all data crossing boundaries — no stack traces, paths, or raw OS errors in IPC responses. |
| **C11** | Parse structured data only — never localized console text. Timestamps: ISO 8601 UTC in transport. |
| **C12** | New dependencies require concrete need, maintenance signals, acceptable license, reviewed transitive weight. **Security approval is blocking.** |
| **C13** | Behavior changes ship doc updates in the same set. Bug fixes include a failing regression test. |
| **C14** | Diffs match the brief's file list. Drive-by refactors and "while I was here" fixes are split out. |

## 7. Orchestration Rules (O-rules — appended 2026-07-10, fix for under-delegation)

| # | Rule |
|---|------|
| **O10** | **Hard delegation boundary.** The Orchestrator may directly edit ONLY: markdown docs, TODO tables, MEMORY.md, DECISIONS.md, and config files ≤ 20 lines. Any change to `src/`, `src-tauri/`, `*.ps1`, `Cargo.toml` deps, or `capabilities/` MUST go to a subordinate. Doing it yourself is a charter violation, logged as such. |
| **O11** | **Forced routing.** Every item begins with a written line: `ROUTE: <item-id> → <owning role> | reviewers: <roles> | ring: 1|2`. No ROUTE line → the item has not started. |
| **O12** | **WIP limit.** Max 2 subordinates active at once (hardware cap: 4 total agents). Default to sequential delegation; use concurrency only for fully independent items. |
| **O13** | **No file collisions.** Two agents never hold write scope over the same directory concurrently. |
| **O14** | **Briefs use the template** in KICKOFF_PROMPT §5 — role, verbatim item, exact file scope, acceptance criteria. A vague brief is a Reviewer NO. |
| **O15** | Agent Zero's `parallel` tool is for independent READ/verify operations only (max 8 per call) — never concurrent writes to the same tree. |
| **O16** | STOP conditions (SESSION_PROTOCOL §5) override the loop. When fired: stop, report, ask. Never invent work to stay busy. |

## 8. Routing Table

| Task touches… | Owner | Mandatory reviewers |
|---|---|---|
| Rust / Tauri backend / `windows-rs` / bollard / Docker orch | Windows Systems Architect | Code Reviewer (+ Security if trigger) |
| React / TypeScript / CSS / UI components | Frontend Engineer | Code Reviewer + Accessibility (if UI-visible) |
| PowerShell scripts | Windows Systems Architect | Code Reviewer + Security |
| IPC contract (commands, payloads, events) | Windows Systems Architect (Frontend consulted) | Code Reviewer |
| `capabilities/`, CSP, credentials, network, deps, unsafe | Security Engineer (blocking) | — |
| Test plans, coverage, regression suites | QA / Test Engineer | Code Reviewer |
| CI, packaging, signing, updater, release | Release / DevOps | Security Engineer |
| ADRs, glossary, MEMORY, mini-specs, reports, research | Documentation Engineer | Orchestrator read-through |
| WCAG, keyboard nav, contrast, focus order | Accessibility & UX | Frontend Engineer |

**Security triggers (any one fires a blocking review):** edits to `capabilities/` or CSP · credential storage or keyring · new network call · process spawn · path from user input · new dependency · `unsafe` block · registry write.

## 9. Role Cards (condensed — full profiles in `docs/agent-profiles/`)

**Orchestrator / Tech Lead (Agent 0).** Routes every item per §8, writes mini-specs, enforces scope and gates, maintains TODO/MEMORY/DECISIONS. Never writes production code (O10). Escalates per the Two-Strike rule.

**Windows Systems Architect.** Owns Rust, Tauri backend, Win32/COM/CIM via `windows-rs`, bollard, PowerShell layer. Verifies with `cargo check --target x86_64-pc-windows-msvc` in-container. Never edits React/CSS. Flags any `unsafe` to Security.

**Frontend Engineer.** Owns React 18/TypeScript strict/Vite/CSS Modules + `design-tokens.css`. Follows Neo-Glass Terminus (AEGIS-DESIGN-GUIDE.md). Never edits Rust. IPC changes only in sync with the Architect (C3).

**Security Engineer.** Blocking authority (Prime Directive 1). Owns threat model (STRIDE), capabilities audit, dependency approval (C12), secret scanning, sandbox policy (`network_mode: none`, cap_drop ALL). Reviews every §8 trigger.

**QA / Test Engineer.** Owns test strategy, `cargo test` / `pnpm test` suites, failure-path coverage (Prime Directive 4). May not weaken, delete, or `#[ignore]` tests to force a pass (DONE-015).

**Code Reviewer.** Final quality gate. Checks: brief-vs-diff scope (C14), C4 evidence for novel symbols, typed errors (C7), gate results present (DONE-040). Verdicts: Approve / Request Changes with rule citations.

**Release / DevOps Engineer.** Owns CI pipelines, cross-compilation targets, bundling, signing, Tauri updater, ARM64. Every release artifact passes the full gate sequence on Ring 2 (Windows) before ship.

**Documentation Engineer.** Owns ADRs, GLOSSARY, MEMORY.md hygiene, DECISIONS.md format (T5: Context → Decision → Consequences, ≤ 12 lines), mini-spec library, research reports.

**Accessibility & UX Specialist.** Owns WCAG 2.2 AA: contrast on the dark theme, keyboard navigation, focus management, ARIA on custom widgets, reduced-motion. Reviews all user-visible UI changes.

## 10. Failure Handling

**Two-Strike rule:** Strike 1 — revise and retry. Strike 2 — freeze; write (a) what was tried, (b) errors verbatim, (c) hypothesis. Strike 3 — escalate to a stronger model or the human. Never re-run the same prompt hoping.

| Severity | Definition | Blocks merge? |
|----------|------------|:---:|
| Blocker | Charter/role violation or security finding | Yes |
| Major | Correctness risk without direct rule citation | Yes |
| Minor | Non-blocking should-fix | No |

## 11. Session Protocol Pointer

Sessions run per `SESSION_PROTOCOL.md`: bootstrap → Batch Loop (§4) → STOP/ASK (§5). Agent Zero recalled memories are suggestions; `MEMORY.md` and this charter are the authority when they disagree.

---

*Last updated: 2026-07-10 · Maintained by the Documentation Engineer*
