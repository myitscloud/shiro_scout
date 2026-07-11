# ADR-006: MCP Server Integration Model

**Date:** 2026-07-08
**Status:** Accepted

## Context

AgentKit agents require Model Context Protocol (MCP) servers to extend capabilities with tools like filesystem access, web search, database queries, and GitHub integration. The integration model determines where MCP servers run, how they're discovered and loaded, and security boundaries between MCP servers and agents.

Key constraints:
- MCP servers run arbitrary code — they are a security boundary
- Some MCP servers require host access (filesystem, Docker socket)
- Users will want to add custom MCP servers
- MCP servers must be manageable from the UI with user approval

## Decision

**MCP servers run inside the Docker sandbox container** as separate child processes, alongside AgentKit agents. They listen on localhost ports. A tool registry in the Rust backend performs allowlist filtering before forwarding tool calls. Each new MCP server connection requires user approval via a HITL dialog.

Default bundled MCP servers (v1):
- **Filesystem** — Read/write within `/workspace` (bounded)
- **Text Editor** — Structured file editing operations
- **Shell** — Command execution with allowlist enforcement

Future / user-installable:
- **GitHub** — Repository operations via API
- **Web Search** — Search engine access (proxied through Tauri host)
- **Database** — SQL query execution with read-only enforcement

## Consequences

**Positive:**
- MCP servers are sandboxed inside the container (maintains air-gap)
- Container already has Node.js/Python for MCP runtime
- Separate processes provide process-level isolation
- Tool registry allowlist prevents unauthorized tool use
- User approval per connection provides accountability

**Negative:**
- MCP servers share container resources with agents
- Some MCP servers need careful capability scoping
- Users who want host-accessing servers need separate trust model
- MCP server crashes can affect container health

## Alternatives Considered

- **MCP servers on host** — More flexible but breaks air-gap security. Rejected for v1.
- **MCP servers in separate containers** — Strongest isolation but resource-heavy and complex orchestration. Revisit for enterprise v2.
- **No MCP (bundled tools only)** — Simplest but not extensible. Rejected — MCP is a key differentiator.
