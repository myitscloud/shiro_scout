# ADR-002: Docker Container Architecture

**Status:** Accepted
**Date:** 2026-07-07
**Deciders:** Windows Systems Architect, Security Engineer, Frontend Engineer

## Context
AI coding agents (AgentKit) require a secure, isolated sandbox for executing code and tool calls. The sandbox must balance security (requirements B1–B4), performance, and developer experience. A multi-role security audit identified 7 conditions that must be met before Phase 2 implementation can proceed.

## Decision Drivers
- Security requirements B1–B4: MCP servers inside container, API keys in OS keychain (never enter container), HITL with 60s timeout, container hardening (cap_drop: ALL, no-new-privileges, read-only rootfs, network: none, 2GB RAM, 0.5 CPU)
- Single container model preferred over per-agent containers for simplicity and resource efficiency
- Read-only rootfs to prevent persistent container tampering
- Network isolation to prevent data exfiltration from sandbox
- AgentKit uses child_process.fork() for process isolation within the shared container

## Considered Options
- **Option A:** One container per agent — full network/filesystem isolation, but complex networking, high memory overhead, multiple IPC connections
- **Option B:** Single long-lived container with AgentKit child_process.fork() — simpler lifecycle, shared tool environment, single IPC connection, uniform management
- **Option C:** VM-based sandbox (Firecracker/microVM) — stronger isolation boundary, but slower startup (500ms+), heavier resource profile, no Docker compose integration

## Decision
Chosen: **Option B — Single shared container with AgentKit process isolation**

One long-lived Docker sandbox with the following hardening: `read_only: true`, `network_mode: "none"`, `init: true`, `cap_drop: ALL`, `no-new-privileges: true`, `pids_limit: 256`, `mem_limit: 2g`, `cpus: 0.5`, non-root user (1000:1000). LLM calls proxied through Tauri host. Workspace via bind mount, temp data on hardened tmpfs (`noexec,nosuid`, 256M for /tmp, 64M for /run). Health check on localhost:8080. All 7 approval conditions satisfied.

All 7 conditions approved:
1. ✅ `network_mode: "none"` — LLM calls proxied through Tauri host
2. ✅ `init: true` — zombie reaper for AgentKit child processes
3. ✅ tmpfs `noexec,nosuid` on /tmp (256M) and /run (64M)
4. ✅ Non-root user (`1000:1000`). ✅ `pids_limit: 256` — fork-bomb protection
6. ✅ Health check (HTTP localhost:8080)
7. ✅ Typed `SandboxEvent` TypeScript contract for frontend status

## Consequences
- Positive: Strong security posture with multiple independent hardening layers
- Positive: Simple lifecycle managed by Tauri app (start/stop/restart via Docker API)
- Positive: Single IPC connection, instant agent switching, shared tool environment
- Positive: Docker Compose configuration is directly portable to CI and local dev
- Negative: All agents share the same container filesystem (workspace isolation only)
- Negative: Requires Tauri-to-Docker communication for all outbound LLM calls
- Negative: Health check requires HTTP bridge running inside container on port 8080

## Compliance
- All container instantiations must use the approved HostConfig values from the Rust/bollard implementation
- Security Engineer must sign off on any deviation from the approved configuration
- The Rust backend must enforce the allowlist before executing any docker exec command
- Frontend must consume typed SandboxEvent for container lifecycle state