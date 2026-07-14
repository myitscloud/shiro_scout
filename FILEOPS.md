# File Operations — Agent Rules & Tool Contract (Linux-first)

> **Purpose:** How agents edit files on this project. **Primary world: the Linux container** (`/a0/usr/projects/shiro_scout/`). **Secondary world: the Windows 11 host** via `*_remote` tools. PowerShell is orchestration-only fallback (DEC-001).
> **Rule IDs are stable** — FILEOPS-001…030 retained from the Windows-era doc; 040+ appended for Linux.

---

## §0 Environment Map — know which world you're in

| World | Where | Paths | Edit tools (priority order) |
|-------|-------|-------|------------------------------|
| **Ring 1 — Container (primary)** | Agent Zero, Kali Linux | `/a0/usr/projects/shiro_scout/…` | `text_editor` → shell `sed`/heredoc → never PowerShell |
| **Ring 2 — Windows host (secondary)** | Remote execution | `C:/…` forward slashes | `text_editor_remote` → Git Bash tools → PowerShell orchestration only |

Emitting a `C:/` path from the container shell, or an `/a0/` path to a remote tool, is always a bug (FILEOPS-040).

## §1 Container Operations (primary — 95% of edits happen here)

### Tool priority

| Priority | Task | Tool | Notes |
|:--:|------|------|-------|
| 1 | Read | `text_editor action:read path:… [line_from/line_to]` | 1-based line numbers, LF-normalized view |
| 2 | Create/overwrite ≤ 250 lines | `text_editor action:write` | The workhorse. Creates parent dirs. |
| 3 | Exact change in a large file | `text_editor action:patch old_text/new_text` | `old_text` unique + verbatim from latest read |
| 4 | Context-anchored change | `text_editor action:patch patch_text` (unified diff) | For inserts near anchors |
| 5 | Search & replace in one file | `sed -i 's/old/new/g' path` | Dry-run without `-i` first (FILEOPS-044) |
| 6 | Multi-file find & replace | `find . -name "*.ts" -exec sed -i 's/X/Y/g' {} +` | Dry-run with `grep -rn` first |
| 7 | Bulk file write from shell | heredoc: `cat > path <<'EOF' … EOF` | Quote the delimiter (FILEOPS-041) |

### Verify-after-write (always)

```bash
wc -l path/to/file            # sane line count?
grep -n "sentinel from new content" path/to/file   # change actually landed?
```

## §2 Core Rules (stable IDs)

| ID | Rule |
|:--:|------|
| FILEOPS-001 | Files ≤ 250 lines: full-file `write`. No surgical edits. |
| FILEOPS-002 | Reserve `patch`/`str_replace`-style edits for files too large to rewrite cheaply. |
| FILEOPS-010 | Read the target immediately before any edit. Never edit from memory. |
| FILEOPS-011 | At most one surgical edit per file per turn, then re-read before the next. |
| FILEOPS-012 | Full-file writes output **every line**. No `// ...` or `# rest unchanged` placeholders. |
| FILEOPS-013 | `old_text` must be unique in the file and copied verbatim from the latest read. |
| FILEOPS-014 | After every write, re-read the changed span or run the compiler/linter to confirm. |
| FILEOPS-020 | All source files: UTF-8 no BOM, **LF** endings. Enforced by `.gitattributes` (`* text=auto eol=lf`). |
| FILEOPS-021 | Forward slashes in all paths, both worlds (`C:/proj/src/main.rs`). |
| FILEOPS-022 | (Windows world) Enable long paths for deep `node_modules`/`target`. |
| FILEOPS-023 | (Windows world) A sharing-violation write = a lock (Defender, editor watcher). Close the running `.exe` first. |
| FILEOPS-030 | On a failed patch, never retry the same string. Read the near-miss, fix exact bytes, or fall back to full `write`. |
| **FILEOPS-040** | Ring 1 edits use container paths only (`/a0/…`); Ring 2 tools get `C:/…` only. Cross-world paths are bugs. |
| **FILEOPS-041** | Shell heredocs quote the delimiter — `<<'EOF'` — so `$`, backticks, and `\` pass through literally. |
| **FILEOPS-042** | No interactive editors or pagers in agent sessions (`vim`, `nano`, bare `less`, `git diff` without `--no-pager`) — they hang the tool call. |
| **FILEOPS-043** | Every write is followed by `wc -l` + a `grep -n` sentinel check before the item proceeds. |
| **FILEOPS-044** | `sed` runs are rehearsed: run the expression without `-i` (or `grep -rn` the pattern) to preview matches, then apply. Multi-file sed additionally reports the file list first. |

## §3 Windows Host Operations (secondary — deploy/verify on target)

`text_editor_remote` contract (behavior identical to §1's editor):

- `action:read` — 1-based line numbers, LF-normalized view, BOM stripped.
- `action:write` — full overwrite, creates parents, writes UTF-8 no BOM + LF, never errors on "exists".
- `action:patch old_text/new_text` — matching order: exact → line-ending-agnostic → indent-insensitive. Failures are recoverable: 0 matches returns the closest region + detected EOL/encoding; >1 match returns count + surrounding lines.
- `action:patch patch_text` — unified diff with context anchors.

Shell fallback on Windows is **Git Bash**, invoked from PowerShell only as a launcher:

```powershell
& "C:/Program Files/Git/bin/sed.exe"  -i 's/old/new/g' path/file.md
& "C:/Program Files/Git/bin/grep.exe" -rn "pattern" src/
& "C:/Program Files/Git/bin/find.exe" . -name "*.ts" -type f
```

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
