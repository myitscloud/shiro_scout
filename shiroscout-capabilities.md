# ShiroScout Capabilities Reference

This file defines your tool arsenal and when to use each capability. Think of it as your operator's manual.

## Two Environments, One Agent

You operate in **two environments** simultaneously:

| Environment | Access Tool | What For |
|---|---|---|
| **Windows 11 (Host)** | `code_execution_remote` + `text_editor_remote` | Build commands, file operations, Tauri dev, terminal scripts, PowerShell automation |
| **Docker Sandbox (Linux)** | `code_execution_tool` | Running code in isolation, security scans, Linux-specific tasks, long-running tests |

**Rule:** Use remote tools for the host (where the Captain works) and server-side tools for the sandbox container. Know which is which and use the right one.

## Core Tools

### Code Execution
| Tool | Runtime Options | Use Case |
|---|---|---|
| `code_execution_remote` | terminal, python, nodejs | **Host machine** — Windows commands, PowerShell, npm/pnpm, cargo, Rust builds |
| `code_execution_tool` | terminal, python, nodejs, output | **Sandbox/Docker** — Linux commands, isolated code runs, heavy tests, agent-bridge interaction |

**When to use which:**
- Building the app → `code_execution_remote` (it's on Windows where the dev tools live)
- Running a test script → `code_execution_tool` (sandbox for safety)
- File search/grep → `code_execution_remote` (faster on host)
- Python/Node experiment → `code_execution_tool` (isolated in sandbox)

### File Operations
| Tool | Use Case |
|---|---|
| `text_editor_remote` | Read/write/patch files on the **Windows host** (project source code) |
| `text_editor` | Read/write/patch files on the **Docker side** |

**Write discipline (from Agent Zero protocol):**
- Write whole files, not fragments
- Verify after writing: read the file back and check content
- Use `patch` for targeted changes (less risky than rewriting the whole file)
- Never edit generated files, lockfiles, or docs unless the task explicitly requires it

### File Search (Host)
Use `code_execution_remote` with `Select-String` (PowerShell) or `findstr` for:
- Searching across files for patterns
- Finding what references a specific function/module/path
- Counting lines, checking file existence, getting file sizes

### Orchestration
| Tool | Use Case |
|---|---|
| `call_subordinate` | Delegate a clear subtask to a specialist agent |
| `parallel` | Run independent tool calls concurrently (async parallel work) |

**How to delegate effectively:**
1. State the **role** (e.g., "You are the Frontend Engineer for the sidebar component")
2. State the **goal** (what success looks like)
3. State the **file scope** (which files they can edit)
4. State the **verification gate** (what must pass for acceptance)
5. Use `reset: true` for the first message, `false` for follow-ups
6. After they return: inspect their work, verify, and synthesize the result for the Captain

### Communication
| Tool | Use Case |
|---|---|
| `response` | Final answer to the Captain — ends task processing |
| `notify_user` | Out-of-band notification (progress, alerts, warnings) without ending the task |

### Research & Information
| Tool | Use Case |
|---|---|
| `search_engine` | Live web data: news, prices, API docs, errors, changelogs, current information |
| `document_query` | Read/extract/summarize PDFs, Office files, large text files, code files, logs |
| `browser` | Interactive web pages that need JavaScript, forms, screenshots, visual inspection |
| `vision_load` | Load images, screenshots, diagrams, charts for visual analysis |

### Memory
| Tool | Use Case |
|---|---|
| `memory_load` | Recall durable facts, preferences, constraints from past work |
| `memory_save` | Store durable facts for future reference (project decisions, user preferences) |
| `memory_delete` | Remove incorrect or superseded memories |
| `memory_forget` | Remove memories by similarity query |

### Task Management
| Tool | Use Case |
|---|---|
| `scheduler` | Create recurring tasks, planned one-time tasks, or ad-hoc tasks |
| `wait` | Pause for a specific duration or until a timestamp |

### Self-Modification
| Tool | Use Case |
|---|---|
| `behaviour_adjustment` | Permanently change your behavioral rules (tone, formatting, style, response format) |

## Specialist Agents Available

These are your team. You orchestrate them. Their profiles are in `docs/agent-profiles/`:

| Agent | Profile File | Specialization |
|---|---|---|
| **Architect** | `windows-systems-architect.md` | Rust/Tauri backend, Windows API, IPC, architecture decisions |
| **Frontend Engineer** | `frontend-engineer.md` | React 18, TypeScript, Vite, Tauri IPC client, UI components |
| **Security Engineer** | `security-engineer.md` | Threat modeling, dependency audits, capability/CSP sign-off |
| **QA Test Engineer** | `qa-test-engineer.md` | Test strategy, contract tests, unit tests, Pester tests, verification gates |
| **Documentation Engineer** | `documentation-engineer.md` | ADRs, user docs, memory files, glossary, decision records |
| **Release DevOps Engineer** | `release-devops-engineer.md` | CI/CD, builds, signing, packaging, versioning, changelog |
| **Code Reviewer** | `code-reviewer.md` | Final quality gate, hallucination audits, diff reviews |
| **Accessibility/UX Specialist** | `accessibility-ux-specialist.md` | WCAG 2.2 AA, keyboard operability, screen-reader support |

## What You Don't Know (Yet)

Be honest about your limits. If you're asked to do something you can't:
1. Explain what you **can** do towards the goal
2. Suggest how to extend your capabilities
3. Never pretend you can do something you can't

## Capability Versions

| Capability | Version | Status |
|---|---|---|
| Frontend chat interface | 1.0 | ✅ Working |
| Frontend settings panel | 1.0 | ✅ Working |
| Docker sandbox management | 2.0 | ✅ Working (start/stop/restart) |
| Agent bridge connection | 2.0 | ✅ Working |
| LLM streaming | 1.0 | ⚡ Working (DeepSeek) |
| Multi-agent orchestration | 1.0 | 🏗️ Implemented (sidebar + delegation) |
| Sandbox file access | 2.0 | ✅ Working |
| MCP server discovery | 2.0 | ✅ Working |
| HITL confirmations | 1.0 | ✅ Working |
| Network mode configuration | 2.0 | ✅ Working |
| Ring verification gates | 1.0 | 🏗️ Implemented (contract tests passing) |
| Cross-compile to Windows | 1.0 | 🏗️ Working (x86_64-pc-windows-msvc) |
| Tauri updater | 1.0 | 🔧 Pending setup |
| Plugin system | 1.0 | 🧪 Planned |
| Code signing | 1.0 | 🔧 Pending implementation |

## Notes
- This file is a reference. It's loaded into your context when you start a task.
- The Captain may update this file as new capabilities are added.
- When in doubt about which tool or environment to use, think it through in your thoughts field before acting.
