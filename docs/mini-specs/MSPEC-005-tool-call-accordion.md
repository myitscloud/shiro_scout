# Mini-Spec 005: ToolCallAccordion Component

**Task:** Create the ToolCallAccordion component — an expandable inline component that shows tool call input, output, duration, and status in the chat thread.

**Layers touched:** Frontend (React/TypeScript + CSS Modules)

**Owning agent:** Frontend Engineer

## Acceptance Criteria

1. Renders as a collapsible accordion in the chat thread with the following visual structure:
```
── Tool Call: search_files ─────────────────────
▶ Input:  pattern="*.rs", root="/workspace"
▶ Output: [src/main.rs, src/lib.rs, ...] (3 files)
  Duration: 0.3s  Status: ✅ Success
────────────────────────────────────────────────
```

2. Collapsed by default — shows only the title bar ("── Tool Call: [name] ────")

3. Expand on click to reveal full input/output/duration

4. Input section shows tool arguments as formatted key-value pairs

5. Output section shows tool return value (truncated if > 500 chars with "Show more" link)

6. Duration badge (mini-badge, right-aligned): shows execution time

7. Status indicator with icon:
   - ✅ Success (green)
   - ❌ Failed (red) — with error message and retry button
   - ⏳ Running (animated spinner with inline mini progress bar 0%→100%)

8. Failed tool calls: red highlight + error message text + retry button

9. CSS Module scoped — uses `design-tokens.css` tokens

10. Yellow left border (`--status-warning`) when embedded in ChatMessage

11. Uses `role="region"` + `aria-label="Tool: [name]"` for screen readers

12. Animated expand/collapse with `--ease-normal` (200ms)

13. Respects `prefers-reduced-motion: reduce` (instant expand/collapse)

14. TypeScript props: `name`, `input`, `output`, `duration`, `status`, `error`, `onRetry`

## Out of Scope

- Live streaming tool args animation (v1.1)
- Nested tool calls

## Review Triggers Expected

- Code Reviewer
- A11y & UX Specialist
