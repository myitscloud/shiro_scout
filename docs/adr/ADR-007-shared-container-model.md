# ADR-007: Shared-Container Model for Agent Execution

**Date:** 2026-07-08
**Status:** Accepted

## Context

Project Aegis needs a secure, sandboxed execution environment for AI coding agents. The two primary architectural options are: (1) one container per agent, or (2) one shared container hosting multiple agents via AgentKit's `createNetwork()`.

Key constraints:
- Cross-platform (Windows, macOS, Linux)
- Minimum resource footprint for a desktop app
- Fast agent switching (sub-second)
- Shared tool environment
- Security isolation between agents and host

## Decision

**Adopt a shared-container model:** one long-lived Docker sandbox container hosts multiple AgentKit agents as child processes. Docker exec is used for tool execution with a Rust-enforced command allowlist. The HTTP bridge runs inside the container for agent communication.

## Consequences

**Positive:**
- Lower memory and CPU overhead vs. per-agent containers
- Simpler Rust orchestration
- Faster agent switching (no container lifecycle per switch)
- Shared workspace and tool caches
- Single IPC connection

**Negative:**
- Reduced isolation between agents (shared rootfs)
- A compromised agent has easier lateral movement
- Container restart kills all agents; per-agent session persistence required
- Requires careful PID namespace management (mitigated via `init: true`)

## Alternatives Considered

- **One container per agent** — Stronger isolation but higher resource usage, slower switching. Rejected for desktop constraints.
- **No container (bare metal)** — Maximum performance but zero isolation. Rejected on security grounds.
