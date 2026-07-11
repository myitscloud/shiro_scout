# ADR-003: AgentKit Bridge Architecture

**Status:** Accepted
**Date:** 2026-07-07
**Deciders:** Windows Systems Architect, Tech Lead, Security Engineer

## Context
AgentKit agents running inside the Docker sandbox need a communication bridge to the Tauri host process for LLM API calls, file system coordination, and tool execution. The bridge must be lightweight, secure, and maintain a small binary footprint (target 3–15MB).

## Decision Drivers
- Binary size target 3–15MB — Node.js sidecar alone exceeds this
- Must work with `network_mode: "none"` — communication is localhost only within the container
- No cross-platform shims or native compilation requirements for npm packages
- Must support streaming responses from LLM calls
- Rust foundation preferred for performance and safety

## Considered Options
- **Option A:** Node.js sidecar process — simplest to prototype, but binary size exceeds 30MB with packed runtime, adds npm dependency chain
- **Option B:** HTTP bridge running inside the Docker container (Rust-based, e.g., actix-web or axum) — small binary (~5MB), native Rust, no external runtime, direct bollard integration
- **Option C:** napi-rs native addon — tight integration with Rust, but adds build complexity, requires separate compilation per platform, complicates cross-compilation for Windows ARM64

## Decision
Chosen: **Option B — HTTP bridge inside Docker container**

A lightweight Rust HTTP server (using axum) runs inside the Docker sandbox on port 8080. It listens for agent requests and proxies them to the Tauri host via a Unix domain socket or shared IPC mechanism that is accessible only from within the container. The bridge handles:
- LLM API calls proxied to Tauri host (which manages API keys securely)
- Tool execution requests forwarded to the Tauri backend for allowlist checking and docker exec dispatch
- Health check endpoint (GET /health) for container readiness
- Streaming response support via Server-Sent Events

Binary target for the bridge is ~3–5MB when compiled statically against musl.

## Consequences
- Positive: Small binary (~3–5MB) keeps overall package under 15MB target
- Positive: Rust ecosystem guarantees no loose npm dependencies
- Positive: Direct integration with bollard (Docker API crate) for Rust backend
- Positive: No host-side port exposure — bridge is localhost-only inside container
- Negative: Requires agent code inside container to make HTTP calls (library dependency)
- Negative: IPC between container bridge and Tauri host requires careful socket permission management
- Negative: Need separate bridge health monitoring in frontend SandboxEvent contract

## Compliance
- The bridge must be compiled statically (musl target) for minimal binary size
- All LLM API keys must remain in Tauri host keychain; the bridge never receives or stores keys
- The bridge must log all proxied requests for audit purposes
- Security Engineer must review the allowlist for all tool execution paths