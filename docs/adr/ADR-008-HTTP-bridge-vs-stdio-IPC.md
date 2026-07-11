# ADR-008: HTTP Bridge Inside Sandbox Container

**Date:** 2026-07-08
**Status:** Accepted

## Context

AgentKit agents running inside the Docker sandbox need a communication channel to the Rust Tauri backend. Options: (1) tunnel over container stdin/stdout via bollard `attach_container`, or (2) run an HTTP/WebSocket server inside the container.

Key constraints:
- Container is air-gapped (`network_mode: "none"`)
- Minimal attack surface
- Support for streaming responses
- Debuggability in development

## Decision

**Adopt HTTP/WebSocket server inside the Docker container** (Idea B). An Express/Fastify server listens on localhost:8080. Rust communicates via reqwest (HTTP) and tokio-tungstenite (WebSocket). MCP servers also bind to localhost ports.

## Consequences

**Positive:**
- HTTP is standard, debuggable with curl and browser DevTools
- WebSocket provides native streaming
- AgentKit examples map directly (agentkit-starter uses API routes)
- Keeps AgentKit in native Node.js environment
- No need to bundle Node.js in the desktop app
- Rust reqwest provides clean async HTTP client

**Negative:**
- Exposes an HTTP listener (mitigated by air-gap and non-root user)
- Slightly higher latency than raw stdio
- Requires health check endpoint on port 8080
- HTTP server adds ~5-10MB to container image

## Alternatives Considered

- **TCP-over-stdio (Idea A)** — Tunnel framed JSON through container stdin/stdout. No network listener but harder to debug, no streaming primitives, custom framing protocol required. Rejected for development velocity.
- **napi-rs bindings** — Embed AgentKit in Rust via native bindings. Most performant but complex to maintain. Revisit for v2.
- **Tauri sidecar** — Bundle Node.js as sidecar binary. Increases binary size significantly. Rejected.
