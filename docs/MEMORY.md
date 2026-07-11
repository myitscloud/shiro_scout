# Memory — ShiroScout Project State & Reference

> **Purpose:** Single source of truth for project state, environment, version locks, paths, constraints.
> **Environment:** Agent Zero in Linux (Kali) Docker container · code targets Windows 11
> **Last updated:** 2026-07-10 · **Line endings:** LF (this file was CRLF before — fixed per FILEOPS-020)

---

## §1 Project State

| Field | Value |
|-------|-------|
| **Current focus** | Wave 1 closeout + Wave 6.7 streaming, then Wave 4 (AgentKit Runtime) |
| **Progress** | Waves 0, 2, 3, 5 ✅ · Wave 1 🟡 (closeout items) · Wave 6 🟡 (only 6.7 open + drift re-verify 6.V) · Waves 4, 7, 8 🔲 |
| **Last action** | 2026-07-10 — governance v2 installed: rebuilt BUILD_PLAN (was corrupted), new Batch Loop + STOP/ASK protocol, Linux-first FILEOPS, O10–O16 delegation rules |
| **Next task** | Batch 1: items 1.A → 1.B → 6.V → 6.7 (see BUILD_PLAN.md) |
| **Active branch** | N/A — git not yet initialized (item 1.D) |
| **Sprint goal** | Green baseline gates, LF everywhere, git initialized, streaming shipped |

## §2 Environment

| Property | Value |
|----------|-------|
| OS / Host | Kali Linux (Debian-based) in Docker · user root |
| Agent Zero root | `/a0/` |
| Project root | `/a0/usr/projects/shiro_scout/` |
| Scratch workdir | `/a0/usr/workdir/` |

**Agent Zero tools:** `code_execution_tool` (terminal/Python/Node, session-persistent) · `text_editor` (read/write/patch) · `document_query` · `search_engine` · `call_subordinate` (Ring 2 delegation) · `memory_load/save/forget/delete` (durable facts only) · `browser` · `vision_load` · `office_artifact` · `parallel` (independent tool calls, max 8 — reads only per O15).

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
/a0/usr/projects/shiro_scout/
├── KICKOFF_PROMPT.md · LOOP_PROMPT.md · SESSION_PROTOCOL.md
├── MEMORY.md (this file) · DECISIONS.md · THREAT_MODEL.md
├── docs/
│   ├── AGENTS.md (charter + role cards + routing)
│   ├── BUILD_PLAN.md · DONE.md · FILEOPS.md · GLOSSARY.md · FEATURES.md
│   ├── agent-profiles/ (9 full role profiles)
│   ├── adr/ (⚠ duplicate numbering — housekeeping item H.1)
│   ├── mini-specs/ (MSPEC-*.md)
│   └── Arch_Design/ (AEGIS-DESIGN-GUIDE.md, PRDs)
├── src/ (React: components/, styles/design-tokens.css, App.tsx, main.tsx)
└── src-tauri/ (Cargo.toml, tauri.conf.json, src/{main,lib,sandbox}.rs)
```

## §5 Architecture Summary

Tauri 2 desktop app → React 18 + TS (Vite, CSS Modules + design-tokens.css) → typed IPC → Rust backend (Tauri commands, bollard, LLM proxy — sandbox has **no network**) → Docker sandbox (read-only rootfs, `network_mode: none`, cap_drop ALL, user 1000:1000, tmpfs /tmp 256M) running AgentKit (Node.js HTTP bridge).

**Design language — Neo-Glass Terminus:** deep bg `#0D0D0F` · glass `rgba(26,26,36,0.85)` + blur(8px) · accent `#8B5CF6` · fonts Geist / Instrument Sans / JetBrains Mono · dark-first (`[data-theme="light"]` override only).

## §6 Key Constraints

| # | Constraint |
|---|------------|
| 1 | Code targets **Windows 11 only** — never make it cross-platform to silence errors |
| 2 | This container is Linux — cannot execute Windows-native code (Ring 1 = static checks + msvc target) |
| 3 | No host filesystem access from the container |
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
| 15 | Agent Zero memory tools = durable facts only; MEMORY.md + charter win conflicts |
| **16** | **WIP limit: max 2 concurrent subordinates** (hardware caps ~4 agents total) — O12 |

## §7 File Editing Quick Reference (Linux-first — full contract in docs/FILEOPS.md)

```bash
text_editor action:read  path:/a0/usr/projects/shiro_scout/FILE.md line_from:1 line_to:50
text_editor action:write path:/a0/usr/projects/shiro_scout/FILE.md content:"..."
text_editor action:patch path:FILE.md old_text:"exact unique text" new_text:"replacement"
grep -rn "pattern" docs/            # search
find . -name "*.md" -type f         # locate
sed 's/old/new/g' file.md           # REHEARSE without -i first (FILEOPS-044)
sed -i 's/old/new/g' file.md        # then apply
wc -l file.md && grep -n "sentinel" file.md   # verify after every write (FILEOPS-043)
```

## §8 Recent History

| Date | Action |
|------|--------|
| 2026-07-07 | Project analysis; BUILD_PLAN v1 (8 waves) |
| 2026-07-08 | SESSION_PROTOCOL, KICKOFF_PROMPT, MEMORY, path-controls, state-preservation docs |
| 2026-07-09 | Wave 6 LLM types fixed — Rust 0 errors; TS fix in progress |
| 2026-07-10 | **Governance v2:** BUILD_PLAN rebuilt after corruption; Batch Loop + STOP/ASK; Linux-first FILEOPS; O10–O16; pnpm standardized; DONE.md dual-ring |

## §9 End Task Ritual (per batch close)

Sync §1 + §8 → update TODO tables → batch report → kill stray processes → (once 1.D lands) commit + push.

---

*Maintained by the Documentation Engineer. §1 and §8 updated at every batch close.*
