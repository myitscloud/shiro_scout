# Mockup Adherence Criteria — Neo-Glass Terminus Design Implementation

> **Purpose:** Strict criteria for UI/UX designers and React frontend engineers to ensure the implemented interface matches the mockup at `/a0/usr/projects/shiro_scout/docs/mockup/aegis-neo-glass-terminus-mockup.html` and `aegis-mockup.png`.
> **Status:** 🔴 = FAIL (deviation not allowed) | 🟡 = WARN (acceptable with justification) | ✅ = PASS (matches mockup)
> **Target:** The final Tauri 2 app must visually and behaviorally match the mockup within 95%+ accuracy. Every component, spacing, color, animation, and interaction must be compared against the reference.

---

## 0. General Principles

| # | Criteria | Threshold |
|---|----------|-----------|
| G1 | **The mockup is the single source of truth** for visual design. AEGIS-DESIGN-GUIDE.md is secondary. When they conflict, mockup wins. | 🔴 |
| G2 | **Pixel parity** — measured by screenshot overlay comparison. No component may deviate by more than 4px in any dimension from the mockup. | 🔴 |
| G3 | **Behavior parity** — every click, hover, focus, input, animation, and transition in the mockup HTML must exist in the React implementation. | 🔴 |
| G4 | **No speculative design** — do not add UI elements, icons, colors, layouts, or interactions not present in the mockup without written approval. | 🔴 |
| G5 | **Responsive breakpoints** must match the mockup CSS: 1180px (hide right panel), 880px (sidebar collapses to icon rail). | 🔴 |

---

## 1. Design Tokens & CSS

### 1.1 Color Tokens

Every CSS custom property value must match the mockup `:root` block exactly:

| Token | Mockup Value | Check |
|-------|-------------|-------|
| `--bg-deep` | `#0D0D0F` | 🔴 |
| `--bg-glass` | `rgba(26, 26, 36, 0.85)` | 🔴 |
| `--bg-glass-elevated` | `rgba(30, 30, 42, 0.92)` | 🔴 |
| `--bg-glass-hover` | `rgba(36, 36, 48, 0.92)` | 🔴 |
| `--border-glass` | `rgba(42, 42, 58, 0.5)` | 🔴 |
| `--border-glass-light` | `rgba(58, 58, 78, 0.6)` | 🔴 |
| `--text-primary` | `#E4E4E7` | 🔴 |
| `--text-secondary` | `#A1A1AA` | 🔴 |
| `--text-muted` | `#6B6B7A` | 🔴 |
| `--text-code` | `#C8C8D8` | 🔴 |
| `--accent-purple` | `#8B5CF6` | 🔴 |
| `--accent-purple-soft` | `#6D28D9` | 🔴 |
| `--accent-purple-glow` | `#A78BFA` | 🔴 |
| `--status-online` | `#22C55E` | 🔴 |
| `--status-thinking` | `#8B5CF6` | 🔴 |
| `--status-warning` | `#F59E0B` | 🔴 |
| `--status-error` | `#EF4444` | 🔴 |
| `--status-neutral` | `#6B7280` | 🔴 |
| `--status-human-wait` | `#3B82F6` | 🔴 |

### 1.2 Typography Tokens

| Token | Mockup Value | Check |
|-------|-------------|-------|
| `--font-ui` | `'Geist', -apple-system, 'Segoe UI', sans-serif` | 🔴 |
| `--font-head` | `'Instrument Sans', 'Geist', sans-serif` | 🔴 |
| `--font-mono` | `'JetBrains Mono', 'Cascadia Code', 'Fira Code', monospace` | 🔴 |
| Body text | 14px, `--font-ui` | 🔴 |
| Chat message text | 13.5px, `--font-mono` | 🔴 |
| Code block text | 12.5px, `--font-mono` | 🔴 |
| Log/terminal text | 12px, `--font-mono` | 🔴 |
| Section labels (uppercase) | 10.5px, 0.12em letter-spacing, `--font-head` | 🔴 |

### 1.3 Glass Effect Tokens

| Token | Mockup Value | Check |
|-------|-------------|-------|
| `--blur-base` | `blur(8px)` | 🔴 |
| `--blur-raised` | `blur(12px)` | 🔴 |
| `--blur-overlay` | `blur(16px)` | 🔴 |
| `--blur-tooltip` | `blur(20px)` | 🔴 |
| Glass fallback (`@supports not`) | `background: #16161e` | 🔴 |

### 1.4 Animation Timing Tokens

| Token | Mockup Value | Check |
|-------|-------------|-------|
| `--ease-fast` | `150ms ease-out` | 🔴 |
| `--ease-normal` | `200ms ease-in-out` | 🔴 |
| `--ease-slow` | `300ms ease-out` | 🔴 |
| `prefers-reduced-motion` | All animations disabled via `animation: none !important; transition: none !important` | 🔴 |
| `.reduce-motion` class | Same as above, for in-app toggle | 🔴 |

### 1.5 Border Radius Tokens

| Token | Mockup Value | Check |
|-------|-------------|-------|
| `--r-sm` | `6px` | 🔴 |
| `--r-md` | `8px` | 🔴 |
| `--r-lg` | `12px` | 🔴 |
| Pills/status dots | `border-radius: 50%` (circle) | 🔴 |
| Toast border | `var(--r-md)` | 🔴 |

### 1.6 Light Palette

The light theme (`body.light` / `[data-theme="light"]`) must exist with ALL of these overrides:
- `--bg-deep: #F8F9FA`, `--bg-glass: #FFFFFF`
- `--text-primary: #1F2937`, `--text-secondary: #6B7280`
- `--accent-purple: #7C3AED` (slightly darker for contrast)
- All `backdrop-filter` properties set to `none`
- All `.glow` and `.avatar.thinking` box-shadows set to `none !important`
- `.backdrop` element hidden (`display: none`)

---

## 2. Layout & Component Structure

### 2.1 App Shell Grid

The top-level layout must be a CSS Grid:

```css
.app {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  height: 100vh;
}
```

With the workspace split inside:

```css
.workspace {
  display: flex;
  flex: 1;
  min-height: 0;
}
```

### 2.2 Navbar / Titlebar (36px)

| Element | Requirement | Check |
|---------|-------------|-------|
| Height | Exactly **36px**, `min-height: 36px` | 🔴 |
| Background | `var(--bg-glass)` with `backdrop-filter: var(--blur-raised)` | 🔴 |
| Bottom border | `1px solid var(--border-glass)` | 🔴 |
| Drag region | `data-tauri-drag-region` on the titlebar element | 🔴 |
| Brand logo | `▲` character, `var(--accent-purple)` color, `drop-shadow(0 0 6px rgba(139,92,246,.6))` | 🔴 |
| Brand text | "Aegis", `var(--font-head)`, 13px, 700 weight | 🔴 |
| Pill buttons | Inline-flex, height 24px, `border-radius: 99px`, 12px font, `var(--text-secondary)` | 🔴 |
| Status dots | 8px × 8px, `border-radius: 50%`, colored + box-shadow glow | 🔴 |
| Agent name in pill | `<strong>` with 600 weight, `var(--text-primary)` | 🔴 |
| Window controls | 44px wide each, right-aligned, `.close:hover` = `var(--status-error)` bg | 🔴 |

### 2.3 Sidebar (236px / 48px collapsible)

| Element | Requirement | Check |
|---------|-------------|-------|
| Width | **236px** (`min-width: 236px`), collapses to **48px** via `body.rail` class | 🔴 |
| Background | `var(--bg-glass)` with `backdrop-filter: var(--blur-base)` | 🔴 |
| Right border | `1px solid var(--border-glass)` | 🔴 |
| Transition | `width var(--ease-normal), min-width var(--ease-normal)` | 🔴 |
| Section label | 10.5px, uppercase, 0.12em spacing, `var(--text-muted)`, `var(--font-head)` | 🔴 |
| Agent slots | Flex row, 26px avatar (8px border-radius), name (13px 500 weight), phase icon | 🔴 |
| Avatar dot | `.st` — 9px × 9px, absolute positioned at `right: -2px; bottom: -2px`, 2px border `var(--bg-deep)` | 🔴 |
| Active agent | `border-color: rgba(139,92,246,.35)` | 🔴 |
| Thinking animation | `@keyframes agent-think` — pulse glow 2s ease-in-out | 🔴 |
| Session groups | "Today", "Yesterday", "This Week" — 10.5px uppercase, `var(--text-muted)` | 🔴 |
| Active session | `border-left: 2px solid var(--accent-purple)`, dot with `box-shadow: 0 0 6px rgba(139,92,246,.9)` | 🔴 |
| Collapsed state | `body.rail` — `.sb-label`, `.agent-name`, `.agent-phase`, `.sessions`, `.new-session span` all `display: none` | 🔴 |
| Footer | 10px padding, `border-top: 1px solid var(--border-glass)`, `.new-session` button + `.railToggle` button | 🔴 |
| Responsive 880px | Same as rail collapse, applied automatically | 🔴 |

### 2.4 Main Panel — Chat

| Element | Requirement | Check |
|---------|-------------|-------|
| Chat header | 40px height, bottom border `1px solid var(--border-glass)`, bg `rgba(13,13,15,.35)` + `blur-base` | 🔴 |
| Chat title | `var(--font-head)`, 13.5px, 600 weight | 🔴 |
| Chat meta | 11.5px, `var(--text-muted)`, `var(--font-mono)` — session ID, workspace, duration | 🔴 |
| Thread | Flex column, gap 14px, padding 20px 20px 8px, `scroll-behavior: smooth` | 🔴 |
| User message | `align-self: flex-end`, left border `3px solid var(--status-neutral)`, `var(--font-mono)` 13.5px | 🔴 |
| Agent message | `align-self: flex-start`, left border `3px solid var(--accent-purple)`, `width: 100%`, box-shadow `-6px 0 18px -10px rgba(139,92,246,.5)` | 🔴 |
| System message | `align-self: center`, no background/border, `var(--text-muted)`, 11.5px, `var(--font-mono)` | 🔴 |
| Message padding | 12px 16px, `border-radius: var(--r-lg)`, max-width 860px | 🔴 |
| Message meta row | Flex, gap 8px, 11.5px, `var(--text-muted)`, `.who` = `var(--text-secondary)` 600 weight | 🔴 |

### 2.5 Code Block

| Element | Requirement | Check |
|---------|-------------|-------|
| Border | `1px solid var(--border-glass-light)`, `border-radius: var(--r-md)` | 🔴 |
| Header | `6px 8px 6px 12px` padding, `var(--bg-glass-hover)` bg, `var(--font-mono)` 11.5px | 🔴 |
| File icon | `📄` character before filename | 🔴 |
| Actions | Copy (`⧉ Copy`) + Run (`▶ Run`) buttons as `btn sm ghost` and `btn sm secondary` | 🔴 |
| Code | 12.5px `var(--font-mono)`, `line-height: 1.7`, `var(--text-code)` color | 🔴 |
| Syntax tokens | `.tk-kw` (purple glow), `.tk-fn` (#7DAFFF), `.tk-str` (#4ADE80), `.tk-cm` (muted italic), `.tk-num` (#FBBF24) | 🔴 |
| Diff lines | `.ln-add` = `rgba(34,197,94,.1)` bg, `.ln-del` = `rgba(239,68,68,.1)` bg + line-through | 🔴 |
| Footer | Lines + language + change count, 10.5px, `var(--text-muted)`, `var(--font-mono)` | 🔴 |

### 2.6 Tool Call Accordion

| Element | Requirement | Check |
|---------|-------------|-------|
| Border | `1px solid var(--border-glass)`, `border-left: 3px solid var(--status-warning)` | 🔴 |
| Status variants | `.tool.ok` = green left border, `.tool.fail` = red left border, `.tool.running` = purple left border | 🔴 |
| Header | Flex row, `8px 12px` padding, `var(--font-mono)` 12.5px, full-width button with `cursor: pointer` | 🔴 |
| Caret | `▶` character, rotates 90° when `.open`, `var(--text-muted)` 10px | 🔴 |
| Tool name | `.tname` with 600 weight | 🔴 |
| Duration badge | `.tdur` — 11px, `var(--text-muted)`, `border-radius: 99px`, `padding: 1px 8px`, `var(--bg-glass-hover)` bg | 🔴 |
| Body | 12px font, `line-height: 1.75`, `var(--text-secondary)`, `display: none` until `.open` | 🔴 |
| Error block | `.terr` — `#FCA5A5` text, `rgba(239,68,68,.08)` bg, `1px solid rgba(239,68,68,.25)` border | 🔴 |
| Progress bar | `.tprog` — 4px height, `border-radius: 99px`, `var(--bg-glass-hover)`, inner `<i>` with gradient + glow + `pulse-dot` animation | 🔴 |
| Retry button | `btn sm secondary` + `btn sm ghost` inside error body | 🔴 |

### 2.7 Agent Phase Strip

| Element | Requirement | Check |
|---------|-------------|-------|
| Location | Between thread and chat input, `margin: 2px 20px 10px` | 🔴 |
| Max-width | 860px (matching messages) | 🔴 |
| Background | `var(--bg-glass)` + `blur-base`, `1px solid rgba(139,92,246,.3)` border | 🔴 |
| Font | `var(--font-mono)` 12.5px, `var(--text-secondary)` | 🔴 |
| Phase icon | `var(--accent-purple-glow)`, `pulse-dot` animation | 🔴 |
| Progress bar | `max-width: 220px`, 5px height, inner `<i>` with purple gradient | 🔴 |
| Percentage | `.pct` — `var(--text-primary)`, 600 weight | 🔴 |

### 2.8 Chat Input

| Element | Requirement | Check |
|---------|-------------|-------|
| Container | `padding: 0 20px 16px` | 🔴 |
| Box | `max-width: 860px`, `border-radius: var(--r-lg)`, `var(--bg-glass-elevated)` bg, `blur-raised` | 🔴 |
| Focus state | Border `var(--accent-purple)`, box-shadow `0 0 0 1px rgba(139,92,246,.4), 0 0 18px rgba(139,92,246,.18)` | 🔴 |
| Textarea | `var(--font-mono)` 13.5px, `line-height: 1.6`, placeholder `var(--text-muted)`, `min-height: 40px`, `max-height: 160px` | 🔴 |
| Toolbar buttons | Attach `📎`, Slash `/`, Code block `{ }` — all `btn icon ghost sm` | 🔴 |
| Hint | `Ctrl+Enter` with `<kbd>` elements, character count in `var(--font-mono)`, `var(--text-muted)` | 🔴 |
| Send button | `btn primary` with "Send ↵" label | 🔴 |

### 2.9 Right Panel — Agent Details (264px)

| Element | Requirement | Check |
|---------|-------------|-------|
| Width | **264px**, `min-width: 264px` | 🔴 |
| Visibility | Controlled by `body.no-right` class (adds `display: none`) | 🔴 |
| Sections | Status, Context window, Recent tools, Cost estimate, Actions | 🔴 |
| Key-value rows | `.kv` — flex `justify-content: space-between`, 12.5px, `var(--text-secondary)`, value in `<b>` `var(--text-primary)` `var(--font-mono)` 12px | 🔴 |
| Tool list | `.rtool` — flex row, icon (✓/✗/⚡), name (`.rn` flex:1), duration | 🔴 |
| Token bar | `.tokbar` — 6px height, inner `<i>` width representing percentage | 🔴 |
| Actions section | `margin-top: auto`, `border-top: 1px solid var(--border-glass)` | 🔴 |
| Kill button | `.btn.danger` with "■ Kill agent" label | 🔴 |
| Responsive 1180px | `display: none` below 1180px (`@media (max-width: 1180px)`) | 🔴 |

### 2.10 Bottom Drawer (220px / collapsible)

| Element | Requirement | Check |
|---------|-------------|-------|
| Height | **220px** (`min-height: 220px`, `max-height: 40vh`) | 🔴 |
| Collapsed | `body.drawer-collapsed` → 34px (`min-height: 34px`), `.drawer-body: display: none` | 🔴 |
| Tab bar | 34px height, tabs as `.tab` class with `.active` state (bg hover + border) | 🔴 |
| Pane content | `.pane.active` controls visibility, each pane has unique content | 🔴 |
| Logs pane | Filter input + chip buttons (ALL/INFO/TOOL/WARN/ERR), log lines with timestamp + level + message | 🔴 |
| Terminal pane | `.term` class, green text (#B7F5C6), purple prompt, dimmed text, error lines (#FCA5A5), block cursor animation | 🔴 |
| Telemetry pane | Stats grid (`grid-template-columns: repeat(auto-fit, minmax(150px, 1fr))`), bar chart with labels | 🔴 |
| Drawer toggle | `btn icon ghost sm`, ▾/▴ arrow based on collapse state | 🔴 |

---

## 3. Interactive Components & Overlays

### 3.1 HITL Approval Dialog

| Element | Requirement | Check |
|---------|-------------|-------|
| Scrim | `rgba(8,8,12,.55)` + `backdrop-filter: blur(3px)`, z-index 50 | 🔴 |
| Modal | `width: min(480px, 94vw)`, `border-radius: 14px`, `var(--bg-glass-elevated)` + `blur-overlay` | 🔴 |
| Animation | `@keyframes modal-in` — opacity 0→1, translateY 10px→0, scale .98→1, .22s ease-out | 🔴 |
| Left accent | `3px solid var(--status-human-wait)` | 🔴 |
| Action quote | `.quote` — `var(--font-mono)` 13px, left border `3px solid var(--status-human-wait)` | 🔴 |
| Countdown | `⏱ Auto-denying in 60s` — `var(--status-warning)`, `var(--font-mono)` 12px | 🔴 |
| Buttons | Review files (secondary) + Deny (ghost) + Approve (primary) — in that order, right-aligned | 🔴 |
| Auto-deny | After 60s countdown: close dialog, toast warning "Action auto-denied" | 🔴 |

### 3.2 Settings Dialog

| Element | Requirement | Check |
|---------|-------------|-------|
| Appearance | Segmented toggle: Dark (default) / Light · high contrast | 🔴 |
| Motion | Checkbox: "Reduce motion — static indicators only" | 🔴 |
| LLM provider | Segmented toggle: Local / Cloud | 🔴 |
| API key | Password input with masked value `sk-••••••••••••••••`, note about OS keyring | 🔴 |
| Model | `<select>` with options: gpt-4o, llama3.3-70b (local), deepseek-v3, claude-sonnet | 🔴 |
| Sandbox | Checkboxes: Start on launch, Mount /workspace read-write | 🔴 |

### 3.3 Command Palette

| Element | Requirement | Check |
|---------|-------------|-------|
| Trigger | `Ctrl+K` or `Ctrl+,` (settings) | 🔴 |
| Alignment | `align-self: flex-start; margin-top: 12vh` | 🔴 |
| Input | `height: 40px`, 13.5px font | 🔴 |
| List | `max-height: 300px`, scrollable, each item with keyboard shortcut `<kbd>` right-aligned | 🔴 |
| Commands | New session, Toggle drawer, Switch agent, Settings, Copy code, Kill agent | 🔴 |

### 3.4 First-Run Wizard

| Element | Requirement | Check |
|---------|-------------|-------|
| Steps | 4-step progress bar with numbered circles, connecting lines | 🔴 |
| Step states | `.step.done` (green), `.step.cur` (purple glow), default (bordered) | 🔴 |
| Step 2 check | Docker Desktop detected + Sandbox image pulled — both with green ✓ | 🔴 |

### 3.5 Toast Notifications

| Element | Requirement | Check |
|---------|-------------|-------|
| Position | Fixed: `right: 16px; bottom: 16px; z-index: 80` | 🔴 |
| Width | 320px, stacked vertically with 8px gap | 🔴 |
| Animation | `@keyframes toast-in` — translateX 20px→0, .25s ease-out | 🔴 |
| Types | 4 variants with left border accent + icon: success (green), error (red), warning (yellow), info (blue) | 🔴 |
| Dismiss | ✕ button in header | 🔴 |
| Auto-dismiss | Success 4s, Warning 8s, Info 6s, Error: manual only | 🔴 |

---

## 4. Interactive Behaviors & JavaScript

All JavaScript behaviors from the mockup must have React equivalents:

| # | Behavior | Implementation | Check |
|---|----------|---------------|-------|
| B1 | **Send message** — Enter or Ctrl+Enter sends, appends user msg to thread, triggers agent thinking phase | React state + event handler | 🔴 |
| B2 | **Token streaming** — Agent response appears character-by-character (3 chars/28ms) with breathing cursor | `useEffect` with `setInterval`, cursor removed on completion | 🔴 |
| B3 | **Tool accordion** — Click caret to expand/collapse, aria-expanded, caret rotates 90° | React state toggle | 🔴 |
| B4 | **Phase lifecycle** — online → thinking → gathering → tool → streaming, each with icon/text/dot changes | State machine (useReducer recommended) | 🔴 |
| B5 | **Sidebar rail toggle** — ⇤/⇥ icon swaps, `body.rail` class controls visibility, transition animation | React state on body className | 🔴 |
| B6 | **Right panel toggle** — ▥ button, `body.no-right` class | React state on body className | 🔴 |
| B7 | **Drawer collapse** — ▾/▴ toggle, `body.drawer-collapsed` class | React state | 🔴 |
| B8 | **Drawer tabs** — Logs / Terminal / Telemetry, `.tab.active` + `.pane.active` | React state, activeTab | 🔴 |
| B9 | **Agent switching** — Click agent slot in sidebar switches active agent, updates navbar + right panel | React state | 🔴 |
| B10 | **Session selection** — Click session item, highlight with left border + purple dot | React state | 🔴 |
| B11 | **New session** — Clears thread, shows empty state with suggestion buttons | React state reset | 🔴 |
| B12 | **Kill agent** — Changes dot to red `.err`, toast "terminated" | React confirmation + state | 🔴 |
| B13 | **HITL dialog** — Fade in with countdown, approve/deny/review, auto-deny at 60s | React state + useEffect timer | 🔴 |
| B14 | **Settings modal** — Theme toggle (dark/light), reduce-motion checkbox | React state, body className | 🔴 |
| B15 | **Command palette** — Ctrl+K, search filter, click or keyboard navigate | React state + keyboard event | 🔴 |
| B16 | **Toasts** — Stacking, type variants, auto-dismiss, manual dismiss | React toast manager component | 🔴 |
| B17 | **Copy code** — Copies code block content to clipboard, shows success toast | `navigator.clipboard.writeText()` | 🔴 |
| B18 | **Log filtering** — Chip toggle (ALL/INFO/TOOL/WARN/ERR) + text search filter | React state, filter logic | 🔴 |
| B19 | **Theme toggle** — Light/Dark classes on body, all tokens recalculated via CSS custom properties | React state, CSS class | 🔴 |
| B20 | **Keyboard shortcuts** — Escape (close overlays), Ctrl+Enter (send), Ctrl+` (drawer), Ctrl+K (palette), Ctrl+, (settings) | `useEffect` with keydown listener | 🔴 |
| B21 | **Empty state** — When no active session, show centered hero text + suggestion chips | Conditional render | 🔴 |

---

## 5. Accessibility Requirements

| # | Requirement | Check |
|---|-------------|-------|
| A1 | All buttons have `aria-label` where icon-only (mockup has these on win-btn, icon buttons, tabs) | 🔴 |
| A2 | Skip-to-content link at top of page: `.skip` — positioned offscreen until focused, then `top: 8px` | 🔴 |
| A3 | `role="log"` and `aria-live="polite"` on chat thread for screen reader updates | 🔴 |
| A4 | `role="dialog"`, `aria-modal="true"`, `aria-labelledby` on all modals/dialogs | 🔴 |
| A5 | `aria-expanded` on tool accordion headers, toggled with open/close state | 🔴 |
| A6 | `aria-label` on sidebar and right panel `<aside>` elements | 🔴 |
| A7 | `role="list"` and `role="listitem"` on session list | 🔴 |
| A8 | `prefers-reduced-motion` media query disables ALL animations and transitions | 🔴 |
| A9 | In-app reduce-motion toggle via checkbox in settings | 🔴 |
| A10 | Focus-visible outlines: `2px solid var(--accent-purple)`, `outline-offset: 2px` on `.btn:focus-visible`, `.pill:focus-visible`, `.win-btn:focus-visible`, `.tab:focus-visible`, `.chip:focus-visible` | 🔴 |
| A11 | Color contrast: All text/background combinations must pass WCAG 2.2 AA (4.5:1 normal, 3:1 large) — light theme especially | 🔴 |
| A12 | `aria-live="assertive"` on phase strip for dynamic status updates | 🔴 |

---

## 6. Responsive Breakpoints

| Breakpoint | Behavior | Check |
|------------|----------|-------|
| >1180px | Full layout: sidebar (236px) + main (flex) + right panel (264px) + drawer | 🔴 |
| ≤1180px | Right panel hidden (`display: none`) | 🔴 |
| ≤880px | Sidebar collapses to icon rail (48px width), labels/sessions/names hidden | 🔴 |

---

## 7. Non-Functional Requirements

| # | Requirement | Check |
|---|-------------|-------|
| N1 | **CSS Modules** per component — no global styles except `design-tokens.css`, `resets.css`, `fonts.css` | 🔴 |
| N2 | **`@layer` cascade** — reset < design-tokens < components < utilities < overrides | 🔴 |
| N3 | **No Tailwind CSS** — all styling via CSS Modules + custom properties | 🔴 |
| N4 | **Lucide icons** (tree-shaken) for standard UI icons, inline SVG for app-specific icons | 🔴 |
| N5 | **Bundle size** — CSS ~40KB gzipped, no CSS-in-JS runtime cost | 🟡 |
| N6 | **Tauri 2 IPC** — all state persistence via typed `tauri-commands.ts` wrapper | 🔴 |
| N7 | **No placeholder/demo data** in production — all state comes from Rust backend | 🟡 |
| N8 | **Keyboard navigable** — all interactive elements reachable and operable via keyboard | 🔴 |
| N9 | **Custom titlebar** — `window.decorations: false` in `tauri.conf.json`, drag region via `data-tauri-drag-region` | 🔴 |

---

## 8. Implementation Priority

Components must be implemented in this order to ensure consistent visual testing against the mockup:

| Priority | Component | Mockup Reference |
|----------|-----------|-----------------|
| P0 | Design tokens (`design-tokens.css`) + CSS reset | `:root` block, `@layer`, scrollbars, glass classes |
| P1 | AppShell + Titlebar + Workspace split | Lines 85–126 (titlebar), `.workspace` |
| P2 | Sidebar (agents + sessions + footer) | Lines 139–187 |
| P3 | Main Panel + Chat header + Thread + Messages | Lines 190–230 |
| P4 | Chat Input + Send button + Toolbar | Lines 232–245 |
| P5 | Agent Phase Strip | Lines 229–231 |
| P6 | Code Block + Syntax highlighting | Lines 100–115 |
| P7 | Tool Call Accordion + Progress | Lines 118–133 |
| P8 | Right Panel — Agent Details | Lines 246–260 |
| P9 | Bottom Drawer — Logs/Terminal/Telemetry | Lines 262–285 |
| P10 | HITL Approval Dialog | Lines 288–305 |
| P11 | Settings Dialog | Lines 307–328 |
| P12 | Command Palette | Lines 330–341 |
| P13 | First-Run Wizard | Lines 343–358 |
| P14 | Toast Notifications | Lines 360–366 |
| P15 | Empty State | JavaScript section `#newSession` |
| P16 | All Interactive Behaviors (B1–B21) | JavaScript section |
| P17 | Accessibility pass (A1–A12) | Throughout |
| P18 | Responsive verification + Light theme | Media queries + `body.light` |

---

## 9. Review Gate

Every component PR must be checked against this criteria document before merging:

| Gate | Who | Criteria |
|------|-----|----------|
| Self-review | Frontend Engineer | All 🔴 items PASS for the component |
| Code review | Code Reviewer | Spot-check 5 random 🔴 items against mockup screenshots |
| Visual review | Orchestrator | Side-by-side comparison with mockup HTML open in browser vs Tauri app |
| A11y review | A11y Specialist | All A# items for new/changed components |
| Final gate | Orchestrator | G1–G5 all PASS, no 🔴 FAIL items remain |

---

*This document is the single source of truth for mockup adherence. Questions or disputes about design decisions are resolved by comparing against the mockup files at `/a0/usr/projects/shiro_scout/docs/mockup/`.*
