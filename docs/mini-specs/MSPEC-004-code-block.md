# Mini-Spec 004: CodeBlock Component

**Task:** Create the CodeBlock component for displaying code snippets with syntax highlighting, file name header, copy/run actions, and line count.

**Layers touched:** Frontend (React/TypeScript + CSS Modules)

**Owning agent:** Frontend Engineer

## Acceptance Criteria

1. Renders a bordered container with:
```
┌──────────────────────────────────────────────┐
│ 📄 main.rs                    [Copy] [Run]   │
│ fn main() {                                  │
│     println!("Hello!");                      │
│ }                                             │
│ Line 1-10 ───                                 │
└──────────────────────────────────────────────┘
```

2. File name tag in header bar (optional — hidden when no filename provided)

3. Copy button copies code content to clipboard, shows brief "Copied!" confirmation

4. Run button (visual only for v1 — actual execution delegated to sandbox) shows loading state when clicked

5. Line count indicator at bottom ("Line 1-10")

6. Monospace font: `--font-mono` (JetBrains Mono), 14px

7. Syntax highlighting — lightweight tokenization (no Prism/Shiki for v1 — use CSS class-based highlighting with basic keyword/string/comment tokens)

8. Horizontal scroll for long lines (overflow-x: auto)

9. Copy button uses `role="button"` with `aria-label="Copy code"`

10. Code block uses `role="code"` for screen readers

11. CSS Module — dark glass surface, uses `design-tokens.css`

12. Horizontal rule style separator between header and code body

13. TypeScript props: `code`, `language`, `filename`, `showRun`, `onRun`

## Out of Scope

- Diff view / line highlighting
- Edit-in-place
- Full Prism/Shiki integration

## Review Triggers Expected

- Code Reviewer
