# Agent Zero Core Extract → Tauri/React/Rust Port Kit

Extracted 2026-07-09 from `agent0ai/agent-zero` (current `main`, MIT license).
Note: the repo was recently reorganized — the old `python/` folder is gone,
prompts moved out of `prompts/default/`, and memory became a plugin. Guides
written against older versions will have stale paths; this extract is current.

## What's in here

| Folder | From repo | What it is |
|---|---|---|
| `prompts/` | `/prompts` | The personality + all framework messages. `agent.system.main.*.md` = identity/communication/solving style. `fw.*.md` = the self-healing repair messages. `behaviour.*.md` = the self-modifying ruleset. |
| `agents/` | `/agents` | Per-profile personality overrides (agent0, developer, hacker, researcher…). Each has `agent.yaml` + a `prompts/` dir that shadows root prompts. This is A0's "personality system." |
| `extensions-python/` | `/extensions/python` | The hook system = the "self-awareness." Ordered `_NN_*.py` files per hook point. Key dirs: `system_prompt/` (composes the persona), `message_loop_prompts_after/` (injects datetime, agent info, skills, workdir every iteration). |
| `plugins-memory/` | `/plugins/_memory` | The whole memory subsystem: FAISS store, consolidation, quality scoring, and the recall/memorize extensions that hook `message_loop_prompts_after` and `monologue_end`. |
| `helpers/` | `/helpers` (selected) | `dirty_json.py` + `extract_tools.py` (lenient parsing — half of self-healing), `errors.py` (Repairable/Intervention/Handled taxonomy — the other half), `history.py` (compression/summarization), `extension.py` (hook loader), `files.py` (prompt reading + `{{var}}` substitution), `rate_limiter.py`, `call_llm.py`, `secrets.py` (error masking). |
| `core/` | repo root | `agent.py` (THE monologue loop — reference implementation) and `models.py` (LLM provider wiring). |
| `rust-skeleton/` | new | Rust structural port. Start at `src/agent.rs`. |
| `LICENSE` | repo root | MIT, © 2025 Agent Zero, s.r.o. |

Excluded: `*.dox.md` auto-docs (noise), `knowledge/`, `webui/`, instance data.
The per-directory `AGENTS.md` files are kept — they document conventions.

## How the self-healing actually works (30-second version)

1. Model must reply in JSON: `{"thoughts": [...], "tool_name": "...", "tool_args": {...}}`.
2. Output won't parse even after `dirty_json` repair → inject `fw.msg_misformat.md`, loop.
3. Output identical to previous turn → inject `fw.msg_repeat.md`, loop.
4. Tool raises `RepairableException` → inject `fw.error.md` **with the error text**, loop. The model reads its own stack trace and tries again.
5. Unknown tool name → inject `fw.tool_not_found.md`, loop.
6. User interrupts mid-run → inject `fw.intervention.md`, loop.
7. Only `HandledException`/critical errors stop the run.

That's the whole trick: a taxonomy of errors + prompt files fed back as user-role
messages. Everything else (memory, behaviour adjustment, summarization) is
ordered extension hooks around that loop.

## Porting map (Python → Rust)

| A0 piece | Rust skeleton | Notes |
|---|---|---|
| `helpers/errors.py` | `core.rs::AgentError` | Repairable = continue, Critical = stop |
| `agent.py::monologue()` | `agent.rs::monologue()` | Same control flow, section comments cite line numbers |
| `helpers/dirty_json.py` | `prompts.rs::json_parse_dirty` | Minimum viable; port the full tokenizer for parity |
| `helpers/files.py` prompt loading | `prompts.rs::PromptStore` | `include_dir!` embeds prompts; profile override chain implemented |
| `helpers/extension.py` + `_NN_` dirs | `core.rs::HookRegistry` | Filename number prefix → `order()` |
| `tools/*.py` | `core.rs::Tool` trait | `response` tool breaks the loop; return `Repairable` from failures |
| `plugins-memory` (FAISS) | not ported | Suggest: embeddings API + SQLite cosine, hook at PromptsAfter + MonologueEnd |
| `helpers/history.py` compression | stub | Uses `fw.topic_summary.*` / `fw.bulk_summary.*` prompts |

## License obligation

MIT: keep `LICENSE` and the copyright notice in your distribution. Prompts are
part of the licensed work — fine to modify/rebrand, keep the attribution file.
