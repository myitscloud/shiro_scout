# GLOSSARY — ShiroScout Terminology

> **Purpose:** Defines key terms used across the project's documentation, planning, and codebase.
> **Created:** 2026-07-15
> **Maintained by:** Documentation Engineer

---

## A–C

| Term | Definition |
|------|------------|
| **ADR** | Architecture Decision Record — a document capturing a significant architectural decision with context, decision, and consequences. Stored in `docs/adr/`. |
| **Agent** | An AI worker running inside the Docker sandbox, powered by AgentKit. Each agent has a lifecycle (idle → thinking → tool → done). |
| **Agent Zero** | *(legacy)* Previous codename for the orchestrator AI agent framework. No longer used — see Orchestrator. |
| **Batch** | A group of up to 8 work items selected from BUILD_PLAN and processed in a single loop iteration. |
| **Batch Close** | The end-of-batch ritual: sync MEMORY.md §1 + §8, write batch report, kill stray processes, commit + push. |
| **Batch Loop** | The core execution engine (§4 of SESSION_PROTOCOL.md): SELECT → ROUTE → Execute → Verify → Book-keep → LOOP. |
| **Bridge** | The HTTP/WebSocket service inside the Docker sandbox that relays tool execution requests from the Tauri host. |
| **C-Rules** | Cross-cutting rules (C1–C14) defined in AGENTS.md that apply to all agents and layers. |
| **Cargo Deny** | Rust supply-chain auditing tool — checks licenses, advisories, and banned crates. |
| **CIM** | Common Information Model — Windows management interface used for remote target communication. |

## D–I

| Term | Definition |
|------|------------|
| **DONE.md** | The Definition of Done document — defines verification gates (G0–G5) and completion report template. |
| **Docker Sandbox** | The `debian:bookworm-slim` container isolating agent execution. Read-only rootfs, `network_mode: none`, cap_drop ALL. |
| **Drift** | Discrepancy between what planning documents claim and what code/reality shows. |
| **G0–G5** | Verification gates: Stub scan, TypeScript check, Frontend build, Rust check, Clippy lint, Supply chain audit, Tests. |
| **HITL** | Human-in-the-Loop — a security checkpoint requiring user approval before dangerous operations execute. |
| **Housekeeping (H-)** | Backlog items (H.1–H.7) that don't belong to any wave but need scheduling. |
| **IPC** | Inter-Process Communication — typed `invoke`/`emit` calls between the React frontend and Rust backend via Tauri. |

## K–R

| Term | Definition |
|------|------------|
| **MCP** | Model Context Protocol — a protocol for MCP servers that provide tools and context to AI agents. |
| **Mini-spec (MSPEC)** | A lightweight specification document for a single feature or component. Stored in `docs/mini-specs/`. |
| **Neo-Glass Terminus** | The project's design language — dark glassmorphism + terminal typography + electric purple accents. |
| **O-Rules** | Orchestration rules (O10–O16) defined in AGENTS.md that govern delegation, routing, and WIP limits. |
| **Orchestrator** | The Tech Lead agent (Agent 0 in legacy terms) that routes work, maintains plans, and never writes production code (O10). |
| **Prime Directives** | Six foundational rules in AGENTS.md §2 that override all other rules when they conflict. |
| **PTY** | Pseudo-terminal — persistent shell sessions inside the Docker sandbox for long-running commands. |
| **Ring 1** | Direct execution by the Orchestrator — docs, configs, reads, verification commands. |
| **Ring 2** | Delegated execution to specialist agents — all production code, multi-file changes, reviews, audits. |
| **ROUTE line** | Every work item starts with a written `ROUTE:` line naming the owner, reviewers, and ring (O11). |
| **Routing Table** | The assignment matrix in AGENTS.md §8 that maps task types to owning roles and mandatory reviewers. |

## S–Z

| Term | Definition |
|------|------------|
| **Sandbox** | The Docker container that isolates agent execution. See Docker Sandbox. |
| **Session** | A single chat conversation from first message to reset. Sessions are auto-saved and grouped by date. |
| **STOP** | A protocol condition that halts the Batch Loop and requires human input. Conditions: STOP-1 (out of work) through STOP-7 (resource limit). |
| **STRIDE** | Microsoft's threat modeling framework (Spoofing, Tampering, Repudiation, Information Disclosure, Denial of Service, Elevation of Privilege). |
| **Tauri** | The desktop application framework — Rust backend + system WebView. Version 2.x. |
| **Tool Call Accordion** | An expandable UI component showing tool input, output, and duration inline in the chat. |
| **Two-Strike Rule** | After two failures on the same approach, freeze and escalate. Never retry the same failing prompt. |
| **Wave** | A major work phase in BUILD_PLAN.md (Waves 0–9). Each wave contains multiple items. |
| **WIP Limit** | Maximum 2 concurrent subordinate agents (O12). Hardware cap is 4 total agents. |
| **WinRM** | Windows Remote Management — protocol for managing remote Windows targets from the sandbox. |

---

*Maintained by the Documentation Engineer. Append new terms as they are introduced.*
