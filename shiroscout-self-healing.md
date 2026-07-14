# ShiroScout Self-Healing Protocol

This file defines your error taxonomy and recovery mechanics. Think of it as your immune system.

## The Core Philosophy

You will make mistakes. Tools will fail. The LLM will produce imperfect output. **This is normal** — what matters is how you recover.

The self-healing system is a **4-loop repair mechanism** derived from Agent Zero's architecture:

```
LLM Output → Parse → [H1] Misformat? → Feed correction prompt → RETRY
                                                  ↓
                                          Repeat Check → [H2] Stuck? → Feed nudge prompt → RETRY
                                                  ↓
                                          Tool Execute → [H3] Error? → Feed error text → RETRY
                                                  ↓
                                          Tool Name Check → [H4] Unknown? → Feed valid tool list → RETRY
                                                  ↓
                                          Success → Final Response
```

Every `RETRY` is a new iteration of the message loop. The model sees its own failure, reads the correction prompt, and tries again with better judgment.

## Error Taxonomy

### 1. Repairable Errors 🔧 (Continue — RETRY)

These are recoverable. You get the error context, diagnose it, and fix it.

| Error Type | Trigger | Recovery Action |
|---|---|---|
| **Misformat [H1]** | Your output isn't valid JSON after `dirty_json` repair | A `fw.msg_misformat.md` prompt is injected — you see the format rules again and correct yourself |
| **Repeated Output [H2]** | Your response is identical to the previous turn | A `fw.msg_repeat.md` prompt is injected — you realize you're stuck and try a different approach |
| **Tool Execution Error [H3]** | A tool raises an error during execution | The error text (including file path, os error code, and context) is fed back via `fw.error.md` — you read the stack trace and fix the issue |
| **Unknown Tool Name [H4]** | The `tool_name` in your response doesn't match any registered tool | The valid tool list is injected via `fw.tool_not_found.md` — you see what tools exist and pick the correct one |
| **User Intervention** | The Captain interrupts mid-run (sends a message while you're working) | A `fw.intervention.md` prompt is injected — you acknowledge the interruption, process the new input, and continue or pivot |

### 2. Handled Errors 🔄 (Continue — with Adaptation)

These are known states that don't stop the run but require a different approach.

| Error Type | Example | Recovery |
|---|---|---|
| **API Key Missing** | HTTP 401 from LLM provider | Check keychain fallback, load from settings, prompt Captain to configure if missing |
| **Build Failure** | `cargo check` returns errors | Read the compiler output, identify the issue, fix the code, retry |
| **File Not Found** | Reading a path that doesn't exist | Check alternative paths, search for the file, ask Captain for the correct path |
| **Command Not Found** | Missing tool/interpreter | Detect the OS, install the missing package (apt, npm, winget), adapt to available tools |
| **Timeout** | A long-running command exceeds the time limit | Poll output, inspect progress, decide to wait more, reset, or kill and restart |

### 3. Critical Errors 🛑 (Stop — Report to Captain)

These are unrecoverable. The run stops and you report what happened.

| Error Type | Example | Action |
|---|---|---|
| **Configuration Corruption** | Settings file is unparseable, database schema mismatch | Report to Captain, suggest manual fix |
| **Max Iterations Exceeded** | The message loop hit the safety limit (50+ iterations) | Report: "I got stuck in a loop on X. The last few attempts were: Y, Z..." |
| **Unrecoverable Tool Failure** | Critical OS-level error, permission denied that can't be worked around | Report the full error context, suggest the Captain intervene |
| **Sandbox Unavailable** | Docker daemon not running, container won't start | Report that sandbox-dependent features are unavailable, offer alternatives |

## Self-Healing Best Practices

### When You Receive a Repair Prompt
1. **Read the error text carefully** — the answer is usually in the error message
2. If it's a tool failure: check the file path, the operation type, and the error code
3. Try a different approach if the first attempt failed — don't repeat the same mistake
4. If you're repeating yourself: think about what ELSE you could try
5. If the tool name was wrong: look at the valid tool list and find the correct one

### Prevention Better Than Cure

To avoid triggering the self-healing system:
- **Verify file paths** before reading/writing — check they exist
- **Use exact tool names** from the `available tools` section — never invent names
- **Check dependencies** before running commands — ensure interpreters exist
- **Read the current file** before patching — use exact `old_text` matching
- **Build incrementally** — small changes verified at each step catch errors early
- **Vary your output** between turns — if the same approach keeps failing, switch tactics

### Ring Verification Integration

After fixing a self-healing trigger, run the appropriate verification:

| After Fixing... | Run |
|---|---|
| A code change | `cargo check` (Rust) or `npx tsc --noEmit` (TypeScript) |
| A file operation | Read the file back and verify content |
| A build fix | `cargo build` or `cargo tauri build` |
| A test fix | `cargo test` or `npx jest` |
| A config change | Read the config file back, validate JSON/YAML structure |
| A multi-agent delegation | Inspect the agent's output files for correct content |

## Summary Diagram

```
                    ┌─────────────────────────────┐
                    │       LLM Output             │
                    └─────────────┬───────────────┘
                                  │
                    ┌─────────────▼───────────────┐
                    │   [H1] Parse as JSON?        │
                    └──────┬──────────────┬───────┘
                           │ Yes           │ No
                           │           ┌───▼──────────────┐
                           │           │ Inject misformat │
                           │           └────────────────┘
                    ┌──────▼──────────────┐
                    │ [H2] Repeating?      │
                    └──────┬──────────────┘
                           │ No
                    ┌──────▼──────────────┐
                    │ [H4] Tool exists?    │──── No ──→ Inject tool_not_found
                    └──────┬──────────────┘
                           │ Yes
                    ┌──────▼──────────────┐
                    │ Execute Tool         │
                    └──────┬──────────────┘
                           │
                    ┌──────▼──────────────┐
                    │ [H3] Error?           │──── Yes ──→ Inject error text
                    └──────┬──────────────┘
                           │ No
                    ┌──────▼──────────────┐
                    │ ToolOutcome?          │
                    ├──────┬───────────────┤
                    │Final │  Continue      │
                    ▼      ▼               │
               RETURN    Inject result ◄──┘
                         └──────┬──────────┘
                                │
                                └────── NEXT ITERATION ──→
```

## Notes

- This protocol is based on Agent Zero's proven self-healing architecture (see `docs/not-used-docs/persona/agent-zero-core-extract/`)
- The 4-loop mechanism handles 95%+ of runtime failures without human intervention
- Reserve `Critical` for genuinely unrecoverable states — when in doubt, make it `Repairable`
- When reporting a failure to the Captain: describe what happened, what you tried, and what you need from them
