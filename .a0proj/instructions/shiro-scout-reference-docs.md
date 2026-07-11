# ShiroScout — Reference Documentation Mapping

## Location
All authoritative reference documentation lives at `/a0/usr/projects/shiro_scout/docs/`.

## Available Documents

### Agent Profiles (AEGIS Charter Extension)
Located in `/a0/usr/projects/shiro_scout/docs/agent-profiles/`

| Profile | File | Role |
|---------|------|------|
| Orchestrator / Tech Lead | `orchestrator.md` | Task intake, mini-specs, routing, sequencing, merge readiness — NEVER writes production code. Contains Agent Zero Mapping (call_subordinate delegation), mini-spec template, routing table, model routing, failure-loop handling, and Role DoD checklists. |
| Windows Systems Architect | `windows-systems-architect.md` | Rust/Tauri backend, Win32/COM/WMI, PowerShell, IPC contract ownership |
| Frontend Engineer | `frontend-engineer.md` | React 18/TypeScript/Vite, components, state, styled IPC client (`tauri-commands.ts`) |
| Security Engineer | `security-engineer.md` | Threat modeling, dependency audit, capabilities/CSP sign-off — blocking authority |
| QA / Test Engineer | `qa-test-engineer.md` | Test strategy, fixtures, coverage, flaky test policy |
| Code Reviewer | `code-reviewer.md` | Final quality gate, diff review, hallucination audit |
| Accessibility & UX Specialist | `accessibility-ux-specialist.md` | WCAG 2.2 AA, keyboard operability, screen-reader validation |
| Release / DevOps Engineer | `release-devops-engineer.md` | CI/CD, signing, packaging, updater, versioning |
| Documentation Engineer | `documentation-engineer.md` | ADRs, user docs, glossary, memory files, AGENTS.md custodianship |

### Architecture & Design
| File | Path | Description |
|------|------|-------------|
| AEGIS Design Guide | `a-zero-docs/Arch_Design/AEGIS-DESIGN-GUIDE.md` | Full design language (Neo-Glass Terminus), CSS tokens, component architecture, security posture |
| Agent Zero Architecture & PRD | `a-zero-docs/Arch_Design/Agent Zero Architecture and PRD.md` | System architecture overview |
| Agent Zero Merged PRD | `a-zero-docs/Arch_Design/MERGE-PRD-ROUGH-DRAFT.md` | Detailed product requirements |

### Mini-Specs
Located in `/a0/usr/projects/shiro_scout/docs/mini-specs/`

| File | Component |
|------|-----------|
| `MSPEC-001-css-design-tokens.md` | CSS Custom Properties token system (Phase 0) |

### ADRs (Architecture Decision Records)
Located in `/a0/usr/projects/shiro_scout/docs/adr/`

| File | Decision |
|------|----------|
| ADR-001 | Document References Strategy |
| ADR-001 | Shared-Container Model for Agent Execution |
| ADR-002 | Docker Container Architecture |
| ADR-002 | HTTP Bridge vs stdio IPC |
| ADR-003 | AgentKit Bridge Pattern |
| ADR-003 | Neo-Glass Terminus Design Language |
| ADR-004 | CSS Architecture |
| ADR-004 | React/TypeScript/Vite Frontend Stack |
| ADR-005 | Bollard Docker API Orchestration |
| ADR-005 | DeepSeek Provider |
| ADR-006 | MCP Server Integration Model |

### Supporting Documents
| File | Path | Description |
|------|------|-------------|
| GLOSSARY | `GLOSSARY.md` | Normative definitions for all project terms |
| User Personas | `user-personas.md` | Target user profiles |
| Rust Crates Reference | `rust-crates-reference.md` | Curated Rust crate catalog |
| Team Feedback | `team-feedback-2026-07-08.md` | Team feedback record |

## Process Notes
- Agent profiles all extend the root `AGENTS.md` charter profile. The charter rules are embedded in each profile's opening statements.
- The orchestrator.md file contains the definitive mini-spec template, routing table, model routing policy, failure-loop handling, and Role DoD checklists.
- Use `call_subordinate` to delegate tasks to specialist subordinates, passing role briefs referencing the agent profile at the above paths.
- For host-machine file access (e.g., to read the actual Agent Zero source code), use `text_editor_remote` — but the container does NOT have native access to the host; this requires the A0 CLI connector.
