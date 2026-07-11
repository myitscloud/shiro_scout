# ADR Index — ShiroScout Architectural Decisions

> **Purpose:** Index of all Architecture Decision Records (ADRs) for the ShiroScout project.
> **Maintained by:** Documentation Engineer
> **Last updated:** 2026-07-10

| ADR | Title | Status | Description |
|:---:|-------|:------:|-------------|
| 001 | Document References Strategy | ✅ Accepted | Reference access strategy — REFERENCE_INDEX.md + read-on-demand vs. local copies |
| 002 | Docker Container Architecture | ✅ Accepted | Single shared Docker sandbox with hardening; HostConfig values, 7 security conditions |
| 003 | AgentKit Bridge Architecture | ✅ Accepted | HTTP bridge (axum) inside Docker container; static musl compilation for 3-5MB binary |
| 004 | CSS Architecture | ✅ Accepted | Global design-tokens.css + CSS Modules per component; no Tailwind, no CSS-in-JS |
| 005 | Rust bollard Crate for Docker Orchestration | ✅ Accepted | Async Docker Engine API via bollard 0.18; Unix socket and named pipe support |
| 006 | MCP Server Integration Model | ✅ Accepted | MCP servers as child processes inside Docker sandbox; tool registry allowlist + HITL approval |
| 007 | Shared-Container Model for Agent Execution | ✅ Accepted | One long-lived Docker container hosting multiple AgentKit agents; Docker exec with allowlist |
| 008 | HTTP Bridge Inside Sandbox Container | ✅ Accepted | HTTP/WebSocket server on localhost:8080; reqwest + tokio-tungstenite communication |
| 009 | Neo-Glass Terminus Design Language | ✅ Accepted | Dark-first glassmorphism + terminal typography; WCAG 2.2 AA, binary size under 15MB |
| 010 | React 18 + TypeScript + Vite Frontend Stack | ✅ Accepted | React 18.3 strict, Radix UI, CSS Modules; vite HMR, 600KB JS gzipped target |
| 011 | DeepSeek as First-Class LLM Provider | ✅ Accepted | Pluggable Provider trait; DeepSeek as reference implementation; keys in OS keychain |

## Cross-References

- ADR-001 documents reference strategy for Agent Zero material
- ADR-002 and ADR-007 together define the Docker sandbox architecture and container model
- ADR-003 and ADR-008 define the bridge between container and Tauri host
- ADR-004 and ADR-009 define the CSS architecture and visual design language
- ADR-005 and ADR-010 define the Docker orchestration and frontend stack
- ADR-006 defines MCP server integration
- ADR-011 defines the LLM provider abstraction

---

*Maintained by the Documentation Engineer. Append new ADRs at the bottom when created.*
