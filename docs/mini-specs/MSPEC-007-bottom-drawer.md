# Mini-Spec 007: BottomDrawer Component

**Task:** Create the BottomDrawer component — a collapsible bottom panel with three tabs: Logs, Terminal, and Telemetry, max 40vh height.

**Layers touched:** Frontend (React/TypeScript + CSS Modules)

**Owning agent:** Frontend Engineer

## Acceptance Criteria

1. Renders as a collapsible bottom panel spanning full width of the window below Main Panel

2. Max height: 40vh when expanded

3. Tab bar at top with 3 tabs:
   | Tab | Content |
   |-----|---------|
   | **Logs** | Filterable event stream (agent actions, tool calls, system events) |
   | **Terminal** | Embedded terminal into the Docker sandbox (xterm.js + websocket) |
   | **Telemetry** | Agent performance stats, token counts, tool timing, cost estimates |

4. Collapsed state: thin bar showing tab labels + expand arrow (▲)

5. Hotkey: `Ctrl+` ` (backtick) to toggle open/closed

6. Animated expand/collapse with `--ease-normal` (200ms ease-in-out)

7. Tab indicator shows active tab with accent purple bottom border

8. LogStream tab:
   - Scrollable log entries with timestamp, level badge [INFO] [TOOL] [ERROR]
   - Filter input at top to filter by text or level
   - Levels color-coded: INFO (neutral), TOOL (purple), ERROR (red), WARN (yellow)

9. Terminal tab (placeholder for v1 — actual xterm.js integration is Phase 2):
   - Display placeholder text: "Terminal connected to sandbox. Type a command or wait for agent output."

10. Telemetry tab (placeholder for v1):
    - Agent performance stats area

11. CSS Module scoped — elevated glass surface (`--elevation-base`) over `--bg-glass`

12. Empty states per design guide §8.1

13. Respects `prefers-reduced-motion: reduce` (instant expand/collapse)

14. TypeScript props: `activeTab`, `logs`, `onTabChange`, `isOpen`, `onToggle`

## Out of Scope

- Full xterm.js terminal integration (Wave 4.28)
- Live telemetry charts (Wave 4.30)
- Resizable drawer height

## Review Triggers Expected

- Code Reviewer
- A11y & UX Specialist
