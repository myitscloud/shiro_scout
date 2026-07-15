# File Operations — Agent Rules & Tool Contract

> **Purpose:** How agents edit files on this project. **Primary world: Windows 11 native** (`c:/shiro_scout/`). All edits use the `apply_diff` tool or direct file I/O. PowerShell is orchestration-only (DEC-001).
> **Rule IDs are stable** — FILEOPS-001…030 retained from the original doc; 040+ appended for Windows-native operations.

---

## §0 Environment Map

| World | Where | Paths | Edit tools |
|-------|-------|-------|------------|
| **Primary** | Windows 11 native | `c:/shiro_scout/…` (forward slashes) | `apply_diff` → `write_to_file` → PowerShell orchestration only |

All paths use forward slashes (`c:/shiro_scout/...`). Using backslashes in tool calls is a bug.

## §1 Primary Operations

### Tool priority

| Priority | Task | Tool | Notes |
|:--:|------|------|-------|
| 1 | Read | `read_file` | Slice or indentation mode |
| 2 | Precise edit | `apply_diff` | Search/replace with start_line anchors |
| 3 | Create/overwrite | `write_to_file` | Full-file writes, creates parent dirs |
| 4 | Search | `search_files` | Regex across files |
| 5 | Bulk find & replace | PowerShell `-replace` or `Select-String` | Preview changes first |

### Verify-after-write (always)

```powershell
# Sane line count?
Get-Content path/to/file | Measure-Object -Line
# Change actually landed?
Select-String -Path path/to/file -Pattern "sentinel from new content"
```

## §2 Core Rules (stable IDs)

| ID | Rule |
|:--:|------|
| FILEOPS-001 | Files ≤ 250 lines: full-file `write_to_file`. No surgical edits. |
| FILEOPS-002 | Reserve `apply_diff` for targeted edits in files too large to rewrite. |
| FILEOPS-010 | Read the target immediately before any edit. Never edit from memory. |
| FILEOPS-011 | At most one surgical edit per file per turn, then re-read before the next. |
| FILEOPS-012 | Full-file writes output **every line**. No `// ...` or `# rest unchanged` placeholders. |
| FILEOPS-013 | `old_text` must be unique in the file and copied verbatim from the latest read. |
| FILEOPS-014 | After every write, re-read the changed span or run the compiler/linter to confirm. |
| FILEOPS-020 | All source files: UTF-8 no BOM, **LF** endings. Enforced by `.gitattributes` (`* text=auto eol=lf`). |
| FILEOPS-021 | Forward slashes in all paths (`c:/shiro_scout/src/main.rs`). |
| FILEOPS-022 | Enable long paths for deep `node_modules`/`target` (Windows). |
| FILEOPS-023 | A sharing-violation write = a lock (Defender, editor watcher). Close the running `.exe` first. |
| FILEOPS-030 | On a failed patch, never retry the same string. Read the near-miss, fix exact bytes, or fall back to full `write_to_file`. |
| FILEOPS-040 | All paths use forward slashes (`c:/shiro_scout/...`). Backslashes in tool calls are bugs. |
| FILEOPS-041 | [Deprecated — Windows-native operations only] |
| FILEOPS-042 | No interactive editors or pagers (`vim`, `nano`, bare `less`, `git diff` without `--no-pager`) — they hang the tool call. |
| FILEOPS-043 | Every write is followed by a line count + sentinel check before the item proceeds. |
| FILEOPS-044 | Preview changes before bulk replacements: use `Select-String` first, apply after review. |

## §3 [Deprecated — previously Windows Host Operations, now merged into §1]

## §4 PowerShell Policy — orchestration only, never content

PowerShell escape rules (`$`, `@`, backticks, quote flavors, `-replace` regex) are incompatible with LLM-generated content commands. Documented casualties:

| Attempt | Result |
|---------|--------|
| `Set-Content -NoNewline` | Stripped every newline from BUILD_PLAN.md |
| Base64 → `Set-Content` | Corrupted Rust files, missing `#` on derives |
| `-replace` regex | Silent no-op replacements from `\`/`$` escaping |
| `@" … "@` here-strings | Broke on backticks/nested quotes in content |

**Allowed (orchestration):** `cargo build 2>&1 | Select-Object -Last 20` · `pnpm lint` / `pnpm typecheck` · `git status` / `git diff --no-pager` · `docker ps`.
**Forbidden:** any PowerShell command whose purpose is to create or modify file *content*. If you catch yourself typing one — stop; use the editor tool or `sed`.

## §5 Path Security Controls

| Tier | Access | Example |
|:----:|:------:|---------|
| 0 | Full read/write | Project workspace |
| 1 | Read-only | Reference directories |
| 2 | Blocked | Everything else |

**PT-001** Canonicalize all paths before authorization — strip `..`, `.`, symlinks, junctions.
**PT-002** Reject null bytes, control characters, NTFS alternate data streams.
**PT-003** Log every blocked path attempt with caller context.

---

*Maintained by the Documentation Engineer. Update when tool behavior changes.*
