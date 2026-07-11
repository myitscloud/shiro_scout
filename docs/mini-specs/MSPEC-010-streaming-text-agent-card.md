# Mini-Spec 010: StreamingText & AgentCard Components

**Task:** Create the StreamingText component (token-by-token rendering with breathing cursor) and AgentCard component (agent status summary with phase indicator, progress, session info).

**Layers touched:** Frontend (React/TypeScript + CSS Modules)

**Owning agent:** Frontend Engineer

## Acceptance Criteria

### StreamingText
1. Renders text tokens incrementally as they arrive (append-only stream)

2. Animated breathing cursor (`▎` character) at the end of streaming text:
```css
@keyframes cursor-breath {
  0%, 100% { opacity: 0.6; text-shadow: 0 0 4px var(--accent-purple-glow); }
  50%      { opacity: 1;   text-shadow: 0 0 8px var(--accent-purple-glow); }
}
```

3. Cursor disappears when streaming completes

4. Rate-limited rendering: caps at 60fps, batches tokens every 50ms to avoid jank on burst tokens

5. Uses `insertAdjacentText`-style fast DOM updates (not setting innerHTML each time)

6. `aria-busy="true"` while streaming, `aria-busy="false"` when complete

7. Supports markdown content rendered inline (plain text fallback for unsupported markup)

8. Respects `prefers-reduced-motion: reduce` — renders text instantly, no animated cursor

9. TypeScript props: `content`, `isStreaming`, `onComplete`, `streamSpeed`

10. Parent consumer gets the complete text via `onComplete` callback

### AgentCard
11. Renders a compact card for agent status overview:
```
┌──────────────────────────────────────────────┐
│  ◉ Alpha   ● Active   🤖 gpt-4o  ⚡ 1 tool  │
│  ──────────────────────────────────────────   │
│  ◐ Thinking: "Searching project files..."     │
│  ████████░░ 68%                               │
│  Last activity: 12s ago                        │
│  Session: "Refactor API routes"               │
└──────────────────────────────────────────────┘
```

12. Avatar with status glow dot (◉ online, ◐ thinking, ◎ gathering, ⚡ running tool, ⚠ error, ✋ awaiting human)

13. Agent name, status text, model badge

14. Phase indicator with icon per design guide §5.2 mapping:
    | AgentKit Phase | Phase Icon |
    |----------------|------------|
    | idle | ◉ |
    | thinking | ◐ |
    | gathering_context | ◎ |
    | running_tool | ⚡ |
    | reviewing_output | ◉ spinning inward |
    | error | ⚠ |
    | awaiting_human | ✋ |

15. Progress bar for thinking/execution phases (percentage)

16. Last activity timestamp

17. Current session name

18. Tool count badge (when tools are active)

19. CSS Module scoped — glass surface `--bg-glass-elevated` with `--elevation-base`

20. TypeScript props: `agent`, `status`, `phase`, `progress`, `lastActivity`, `sessionName`, `toolCount`, `onClick`

## Out of Scope

- Inline agent card actions (pause/kill/resume) — those are in the right panel
- Multiple agent cards in a list (AgentRoster handles that)

## Review Triggers Expected

- Code Reviewer
- A11y & UX Specialist
