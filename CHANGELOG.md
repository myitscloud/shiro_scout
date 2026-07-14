# Changelog

All notable changes to ShiroScout are documented in this file.

## [0.1.0] - 2026-07-13

### Wave 0 — Project Foundations
- Initialize Tauri 2 + React + TypeScript project scaffolding
- Configure Vite, ESLint, and TypeScript strict mode
- Establish Rust backend crate structure
- Set up pnpm workspace and lockfile

### Wave 1 — Docker Sandbox Backend
- Implement Docker daemon health check (bollard API)
- Create sandbox container lifecycle: create, start, stop, remove
- Build sandbox image definition (Aegis Sandbox)
- Implement `exec_sandbox_command` for container command execution
- Set up Docker bridge client for sandbox communication

### Wave 2 — Docker Bridge & Container Networking
- Implement `agent-bridge` binary for container-host IPC
- Set up HTTP bridge on port 8080 for agent-sandbox communication
- Add network isolation & security controls (`network_mode: none`)
- Configure container capabilities: memory 2GB, 0.5 CPU
- Add bridge client polling and health checks

### Wave 3 — LLM Integration & Agent Orchestration
- Integrate `rig` LLM client library for provider-agnostic completions
- Build streaming completion pipeline (`stream_llm_completion`)
- Implement LLM health check per provider
- Create ShiroScout agent orchestrator state machine
- Wire inline agent processing through Tauri IPC commands

### Wave 4 — Settings & State Persistence
- Implement settings system: load, save, reset LLM/api/workspace config
- Add secure API key storage via Windows Credential Manager (`windows-rs`)
- Implement agent state persistence (auto-save on close, restore on boot)
- Add persistent PTY session management
- Implement MCP server discovery and registration

### Wave 5 — Frontend Architecture & UI
- Build React chat interface with message history
- Implement LLM settings panel (provider, model, temperature)
- Add Docker sandbox management controls (start/stop/restart)
- Create HITL (Human-in-the-Loop) confirmation dialog
- Implement dark theme with Neo-Glass Terminus design

### Wave 6 — Security & Air-Gapped Mode
- Implement sandbox network mode enforcement (air-gapped toggle)
- Add HITL confirmation request/response IPC (Item 7.1)
- Add sandbox network mode setter IPC (Item 7.2)
- Set up secret-scan CI with gitleaks (D7 compliance)
- Add capability-based permission system (Tauri 2 capabilities)

### Wave 7 — Final Integration & Polish
- Code review and cleanup across all layers
- Fix workspace path setting synchronization
- Verify end-to-end: app boot → sandbox → agent → response
- Ensure `cargo check` passes in both debug and release modes
- Final git commit and push to origin/main

### Wave 8 — Distribution & Release
- Sync version across all manifests (0.1.0)
- Create CHANGELOG.md (this file)
- Add `tauri-plugin-updater` dependency and plugin registration
- Configure Tauri updater with signed manifest support (stub server URL)
- Add updater permissions to default capability
- Configure MSI/NSIS bundle targets for enterprise silent install
- Generate CycloneDX SBOM (via cargo-deny deny.toml skeleton)
- Add build CI workflow for automated verification
- Add local build verification script

---

*This changelog follows the [Keep a Changelog](https://keepachangelog.com/) format.*
