# Memory — ShiroScout Project State & Reference

> **Purpose:** Single source of truth for project state, environment, version locks, paths, constraints.
> **Environment:** Windows 11 — Tauri 2 + React 18 + TypeScript + Vite + PowerShell 7
> **Last updated:** 2026-07-15 · **Line endings:** LF (FILEOPS-020)

---

## §1 Project State

| Field | Value |
|-------|-------|
| **Current focus** | Waves 0–7, 9 complete. Wave 8: 8.1 build verified (23.4 MB binary, 8.6 MB MSI), 8.3 updater configured. Remaining: code signing, Ring 2 release run. |
| **Progress** | Waves 0, 1, 2, 3, 4, 5, 6, 7, 9 ✅ · Wave 8 🟡 (8.1 packaging done, 8.3 updater partial, 8.2/8.4 open) |
| **Last action** | 2026-07-15 — Full session: Rig/CRLF audit, 6.V drift verify, Wave 4 doc sync, Wave 7.3 threat model refresh, Wave 7.4 gitleaks CI, Wave 7.5 capabilities audit, Wave 8 release workflow + build test. ARM64 dropped. All gates verified (104/104 tests). |
| **Next task** | Wave 8.2: Acquire code signing certificate, set GitHub secrets. 8.4: Ring 2 release run on Windows hardware. |
| **Active branch** | main — pushed to `origin/main` |
| **Sprint goal** | Wave 8 complete: code signing + Ring 2 release run |

## §2 Environment

| Property | Value |
|----------|-------|
| OS / Host | Windows 11 (x64, ARM64) |
| Project root | `c:/shiro_scout/` |
| Sandbox container | Debian Bookworm Slim (`debian:bookworm-slim`) |
| Package manager | pnpm (standardized — see DEC-004) |

## §3 Version Locks

| Tool | Version | | Tool | Version |
|------|---------|-|------|---------|
| Rust | 1.96.1 | | Tauri | 2.x |
| Node.js | 22.22.0 | | React | 18.x |
| **pnpm** | latest for Node 22 (standardized, DEC-004) | | TypeScript | 5.x strict |
| Python | 3.x (venv) | | Vite | 5.x |

**Key crates:** tauri 2.x · tauri-plugin-shell 2.x · bollard 0.18 · serde/serde_json 1.x · tokio 1.x (full) · camino 1.x · parking_lot 0.12 · windows (latest) · futures 0.3 · keyring.
**Key npm:** @tauri-apps/api 2.x · react/react-dom 18.x · lucide-react (tree-shaken) · @radix-ui/* · allotment.

## §4 Key Paths

```
c:/shiro_scout/
├── AGENTS.md · BUILD_PLAN.md · DECISIONS.md · DONE.md · FEATURES.md
├── FILEOPS.md · KICKOFF_PROMPT.md · LOOP_PROMPT.md · MEMORY.md
├── SESSION_PROTOCOL.md
├── docs/
│   ├── adr/ (ADR-*.md)
│   ├── Arch_Design/ (AEGIS-DESIGN-GUIDE.md)
│   ├── agent-profiles/ (9 full role profiles)
│   ├── mini-specs/ (MSPEC-*.md)
│   ├── threats/ (THREAT_MODEL.md)
│   └── xterm-mspec/ (MSPEC-T1, AGENT-HANDOFF-xterm)
├── scripts/ (auto-backup.ps1, build-release.ps1)
├── src/ (React: components/, styles/, App.tsx, main.tsx, hooks/)
├── src-tauri/ (Cargo.toml, tauri.conf.json, capabilities/)
│   ├── src/ (main.rs, lib.rs, bridge_client, container, docker_client, …)
│   │   ├── agent/ · llm/ · mcp/ · prompts/ · pty/ · sandbox/ · tools/
│   └── docker/ (Dockerfile.sandbox, entrypoint.sh, bridge/)
```

## §5 Architecture Summary

Tauri 2 desktop app → React 18 + TS (Vite, CSS Modules + design-tokens.css) → typed IPC → Rust backend (Tauri commands, bollard, LLM proxy — sandbox has **no network**) → Docker sandbox (read-only rootfs, `network_mode: none`, cap_drop ALL, user 1000:1000, tmpfs /tmp 256M) running AgentKit (Node.js HTTP bridge).

**Design language — Neo-Glass Terminus:** deep bg `#0D0D0F` · glass `rgba(26,26,36,0.85)` + blur(8px) · accent `#8B5CF6` · fonts Geist / Instrument Sans / JetBrains Mono · dark-first (`[data-theme="light"]` override only).

## §6 Key Constraints

| # | Constraint |
|---|------------|
| 1 | Code targets **Windows 11 only** — never make it cross-platform to silence errors |
| 2 | Ring 1 static checks use `cargo check --target x86_64-pc-windows-msvc`; full runtime tests require Ring 2 (Windows) |
| 4 | Rust uses the `windows` crate for Win32; compile-check against `x86_64-pc-windows-msvc` |
| 5 | Tauri 2.x, not 1.x (ADR-004) |
| 6 | CSS Modules + custom properties — no Tailwind, no CSS-in-JS (ADR-004) |
| 7 | One `design-tokens.css` (MSPEC-001) |
| 8 | Lucide tree-shaken + inline SVGs only — binary size target 3–15 MB |
| 9 | **pnpm** everywhere (DEC-004) |
| 10 | Sandbox `network_mode: none` — LLM calls proxied through the Tauri host (ADR-002) |
| 11 | Ring 1 direct / Ring 2 delegated per SESSION_PROTOCOL §1; O10 delegation boundary applies |
| 12 | TODO updated on every item completion |
| 13 | DONE.md gates before any ✅ |
| 14 | Two-strike rule — escalate after 2 failures |
| 15 | MEMORY.md and AGENTS.md (charter) are authoritative; trust them over any recalled context |
| **16** | **WIP limit: max 2 concurrent subordinates** (hardware caps ~4 agents total) — O12 |

## §7 File Editing Quick Reference (Windows-native — full contract in `FILEOPS.md`)

Use the `apply_diff` tool for precise edits. For command-line operations:
```powershell
# Search
Select-String -Path "docs/*.md" -Pattern "pattern"

# Locate files
Get-ChildItem -Recurse -Filter "*.md"

# Verify after write
Get-Content FILE.md | Measure-Object -Line
Select-String -Path FILE.md -Pattern "sentinel"
```

## §8 Recent History

| Date | Action |
|------|--------|
| 2026-07-07 | Project analysis; BUILD_PLAN v1 (8 waves) |
| 2026-07-08 | SESSION_PROTOCOL, KICKOFF_PROMPT, MEMORY, path-controls, state-preservation docs |
| 2026-07-09 | Wave 6 LLM types fixed — Rust 0 errors; TS fix in progress |
| 2026-07-10 | **Governance v2:** BUILD_PLAN rebuilt after corruption; Batch Loop + STOP/ASK; Linux-first FILEOPS; O10–O16; pnpm standardized; DONE.md dual-ring |
| 2026-07-14 | **Infrastructure fixed + validated:** `tool_call_id` deserialization error resolved, keychain fallback, role normalization, automated backup, git push |
| 2026-07-15 | **Full verification audit (Phases A–F):** All files verified, cross-references checked, build gates run (G0–G5 pass with 3 fixes). TRUE status synced to BUILD_PLAN.md, MEMORY.md, SESSION_PROTOCOL.md. Rig 0.39.0 integration confirmed. 104/104 tests passing. |
| 2026-07-15 (late) | **Rig/CRLF audit session:** Verified all Wave 9 items complete (Rig migration, streaming, dead code, gitattributes, CRLF, pnpm audit, CodeMirrorInput+Shiki). Fixed stray `O` char in `lib.rs:1` causing compile error. Fixed unused var warning in `keychain.rs:307`. Removed stale `.a0proj/` and dead `env/mod.rs` from git index. Renormalized git line endings. Updated BUILD_PLAN.md and MEMORY.md to reflect true status. **Gates:** G0–G5 all pass (tsc, pnpm build, cargo check, clippy, 104/104 tests). |
| 2026-07-15 (end) | **Full sprint completion:** Wave 6.V drift verify completed (gates pass). Wave 4 doc sync (FEATURES.md, README.md updated). Wave 7.3 threat model refresh (confirmed accurate). Wave 7.4 gitleaks CI verified (already wired). Wave 7.5 capabilities audit (minimal and correct). Wave 8 release workflow created (`.github/workflows/release.yml`). Tauri build verified: 23.4 MB binary, 8.6 MB MSI. `_notes` config error fixed. ARM64 dropped. BUILD_PLAN.md fully updated. All remaining open items: Wave 8.2 (code signing) + Wave 8.4 (Ring 2 run). **End Task Ritual** performed — MEMORY synced, stray processes killed, git commit + push to `origin/main`. |

## §9 End Task Ritual (per batch close)

Sync §1 + §8 → update TODO tables → batch report → kill stray processes → commit + push.

---

*Maintained by the Documentation Engineer. §1 and §8 updated at every batch close.*
