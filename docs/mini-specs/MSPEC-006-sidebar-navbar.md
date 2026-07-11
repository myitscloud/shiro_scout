# Mini-Spec 006: Sidebar & Navbar Components

**Task:** Create the Sidebar (48px expanded / icon-only collapsible) and Navbar (36px persistent) layout frame components for the App shell.

**Layers touched:** Frontend (React/TypeScript + CSS Modules)

**Owning agent:** Frontend Engineer

## Acceptance Criteria

### Navbar (36px height)
1. Renders persistent top bar with layout:
   | Element | Content | Interaction |
   |---------|---------|-------------|
   | **App icon + name** | ▲ Aegis (SVG logo, left-aligned) | Click → home/dashboard |
   | **Agent status** | ◉ Agent Name (with glow dot) | Click → agent switcher dropdown |
   | **Privacy badge** | 🔒/☁ tooltip: provider name + type | Info display |
   | **Sandbox status** | Docker icon + green/yellow/red dot | Click → sandbox details |
   | **Settings** | ⚙ gear icon | Click → settings drawer/overlay |
   | **Window controls** | — □ × | Custom minimize/maximize/close |

2. `data-tauri-drag-region` attribute for window dragging

3. Settings button opens Settings view in main panel

### Sidebar (48px expanded, icon-only minimum)
4. Renders vertical left panel with sections:
   - **Agent roster** — Icons with status glow dots for each agent
   - **Session timeline** — Chat sessions grouped by date (Today, Yesterday, This Week, Older)
   - **+ New** — Button to start new session
   - **Bottom** — Settings / Preferences icon

5. Click agent slot → switches active agent

6. Click session → loads that session's chat history

7. Sessions auto-saved in Tauri backend state

8. Right-click session → context menu: Rename, Delete, Export

9. Sessions grouped by date with labeled headers

10. Active session highlighted with accent left border

11. CSS Module scoped — uses `design-tokens.css` tokens

12. Keyboard navigable: arrow keys to move between items, Enter to select

13. `aria-label` on icon-only buttons

14. All status indicators use colored glow dots (not badges) per design principle "Status as Light"

15. TypeScript props for sidebar: `agents`, `sessions`, `activeSessionId`, `activeAgentId`, `onSelectSession`, `onSelectAgent`, `onNewSession`

## Out of Scope

- Drag-to-reorder agents or sessions
- Right Panel (Phase 2)

## Review Triggers Expected

- Code Reviewer
- A11y & UX Specialist
