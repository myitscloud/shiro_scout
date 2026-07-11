# ADR-005: Rust bollard Crate for Docker Orchestration

**Date:** 2026-07-08
**Status:** Accepted

## Context

Project Aegis requires programmatic management of Docker containers for the AI agent sandbox. The orchestration layer needs to: connect to the local Docker daemon, pull images with progress streaming, manage container lifecycle with security constraints, execute commands inside containers, and stream logs in real-time.

Key constraints:
- Cross-platform (Windows Docker Desktop, macOS Orbstack/Colima, Linux Docker Engine)
- Must be reliable — no fragile shell scripting
- Must support async operations (tokio)
- Must support Unix socket (Linux/macOS) and named pipe (Windows) connections

## Decision

**Adopt the `bollard` crate** (version 0.18) as the sole Docker API client for Rust. Bollard provides fully asynchronous, native Rust bindings for the Docker Engine API over the local socket/pipe.

## Consequences

**Positive:**
- Deterministic, type-safe container management
- Progress streaming for image pulls via event stream
- Clean async/await patterns with tokio
- No shell injection vectors
- Works with Docker Desktop, Orbstack, Colima, Podman (Docker-compatible mode)
- Real-world precedent: clawpier, dockyard use bollard

**Negative:**
- Async-first crate requires tokio runtime in Tauri commands
- API differences between bollard versions (some breaking changes)
- Podman compatibility requires Docker-compatible API mode
- Adds ~50KB to binary size
- Learning curve for developers unfamiliar with Docker Engine API

## Alternatives Considered

- **Shell script invocations (`std::process::Command`)** — Simpler to start but fragile, non-deterministic, high maintenance, shell injection risk. Rejected.
- **Podman API via REST client** — Better for Linux-only but less cross-platform. Rejected.
- **Dockerode (Node.js sidecar)** — Would require bundling Node.js. Rejected.
- **docker_api crate** — Alternative to bollard but less maintained. Rejected.
