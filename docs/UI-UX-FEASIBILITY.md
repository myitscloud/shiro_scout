# Project Shiro Scout — UI/UX Feasibility Notes

> Produced by Frontend Engineer
> Date: 2026-07-07

---

## 1. CSS Framework / Styling Approach

**Recommendation: Vanilla CSS with Custom Properties + CSS Modules + selective Open Props**

| Approach | Binary cost | Flexibility | DX |
|---|---|---|---|
| Tailwind | ~300KB compiled + JIT | Low (utility lock-in) | High |
| Tailwind + heavy custom theme | ~400KB + purge overhead | Medium | High |
| CSS Modules + CSS custom properties | 0KB (native) | Maximum | Medium |
| styled-components | ~15KB runtime + bundle cost | High | Medium (RSC incompatible) |
| Vanilla Extract | 0KB runtime, ~0KB build output | High | Medium |
| Open Props (subset) | ~30KB minified, tree-shakeable | High | Medium |

**Why not Tailwind?**
- User explicitly finds it boring — the utility-class look is hard to override for a distinctive visual identity.
- Tailwind's purge-based CSS still carries a default aesthetic cost.
- For a 3-15MB app, every KB matters.

**The approach:**
1. **CSS Custom Properties** for design tokens (colors, spacing, typography, radii) → single `design-tokens.css` file.
2. **CSS Modules** for component-scoped styles with `composes:` for shared patterns.
3. **Open Props** selectively imported (just the `normalize` + a few animation tokens) — tree-shaken by Vite.
4. **`@container` queries** for panel-based responsive layout instead of viewport breakpoints.
5. **`@layer` cascade** : reset < design-tokens < components < utilities < overrides — keeps specificity flat.

**Seen-it-done-before:** Treat the theme as a first-class concept — define `--aegis-surface`, `--aegis-text`, `--aegis-accent` in CSS, switch them in `:root { color-scheme: dark }` / `[data-theme="light"]`. No framework lock-in.

**Binary estimate:** ~40KB gzipped for a hand-crafted token + component system with Open Props subset.

---

## 2. UI Libraries

**Recommendation: Radix UI Primitives (headless) + Floating UI + Framer Motion (limited scope)**

### What to bring in:

| Library | Cost (gzip) | Purpose |
|---|---|---|
| Radix UI primitives | ~25KB (tree-shaken) | Unstyled dialog, popover, dropdown, tabs, tooltip |
| Floating UI | ~8KB | Positioning for modals, popovers, tooltips (midpoint/edge detection) |
| Framer Motion | ~35KB (minimal subset) | Layout animations, page transitions, streaming text cursor |
| @tanstack/react-virtual | ~10KB | Virtual scrolling for agent chat threads |

### What to skip:
- **MUI, Ant Design, PrimeReact** — too heavy (200KB+), opinionated design = "boring" risk
- **shadcn/ui** — Tailwind-dependent; user finds Tailwind boring
- **React Aria** — great accessibility but 2× size of Radix for same primitives

### Implementation tactic:
All UI libs above are **headless** or style-agnostic — they own behavior, we own the look. Wrap each in a `components/ui/` module that applies our custom CSS Module + token classes.

```tsx
// components/ui/Dialog.tsx
import * as RadixDialog from '@radix-ui/react-dialog';
import styles from './Dialog.module.css';

export function Dialog({ children, ...props }) {
  return (
    <RadixDialog.Root {...props}>
      <RadixDialog.Portal>
        <RadixDialog.Overlay className={styles.overlay} />
        <RadixDialog.Content className={styles.content}>
          {children}
        </RadixDialog.Content>
      </RadixDialog.Portal>
    </RadixDialog.Root>
  );
}
```

**Binary estimate:** ~80KB gzipped for the library bundle (tree-shaken).

---

## 3. Font Strategy

**Recommendation: Bundled .woff2 variable fonts + `@font-face` via Tauri asset protocol**

### How to load in Tauri 2:

1. Place `.woff2` files in `src/assets/fonts/`
2. Import via Vite static asset reference:
```css
@font-face {
  font-family: 'Geist';
  src: url('./assets/fonts/GeistVariableVF.woff2') format('woff2-variations');
  font-weight: 100 900;
}
```
3. Vite auto-hashes and exposes them; Tauri's asset protocol serves from `tauri://localhost/`

### Font recommendations for a "distinctive non-boring look":

| Font | Type | Weight | Use | Size (woff2) |
|---|---|---|---|---|
| **Geist** | Variable sans-serif (Vercel) | 100-900 | UI text (system font replacement) | ~30KB |
| **JetBrains Mono** | Variable mono | 100-800 | Code blocks, terminal, tool output | ~40KB |
| **Instrument Sans** | Variable sans-serif | 400-700 | Headings, agent names | ~25KB |

All three fit in ~100KB total. No system font fallback spaghetti.

**Why not system fonts?** Custom fonts are the single biggest lever for "distinctive look" — they make the app instantly recognizable.

**Fallback strategy:**
```css
--font-ui: 'Geist', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
--font-mono: 'JetBrains Mono', 'Cascadia Code', 'Fira Code', monospace;
--font-heading: 'Instrument Sans', 'Geist', sans-serif;
```

**Binary impact:** ~100KB — well within the 3-15MB target.

---

## 4. Icon Strategy

**Recommendation: Lucide icons as a tree-shaken subset + SVG sprite for app-specific icons**

| Option | Cost | DX |
|---|---|---|
| Lucide (tree-shaken, direct imports) | ~5KB per 30 icons | Star — named exports, TypeScript types |
| Phosphor (tree-shaken) | ~6KB per 30 icons | Great but heavier weight families |
| Custom SVG sprite | ~3KB for a sprite of 20 icons | Manual; needs asset pipeline |
| Inline SVGs | 0KB library but ~1KB/icon repeated | Tedious; no consistency |
| Heroicons | ~4KB tree-shaken | Good, only outline/solid |

**Recommendation:** Use **Lucide** for standard UI icons (chevron, x, search, settings, terminal, etc.) and a hand-crafted SVG sprite for app-specific icons (agent states, container, tool types).

```tsx
// components/ui/Icon.tsx
import { type LucideIcon, Terminal, Code, Bot } from 'lucide-react';

type IconProps = { name: string; size?: number };

const customIcons = {
  'agent-idle': './assets/icons/agent-idle.svg',
  'agent-thinking': './assets/icons/agent-thinking.svg',
  'sandbox': './assets/icons/sandbox.svg',
} as const;

export function Icon({ name, size = 16 }: IconProps) {
  // Load from Lucide or custom SVG sprite
}
```

**Binary impact:** ~5KB gzipped.

---

## 5. Layout Components (Single-Window)

**Recommendation: CSS Grid + Resizable panels + Custom titlebar (Tauri decorations: false)**

### Grid structure:

```css
.app-shell {
  display: grid;
  grid-template-areas:
    'titlebar  titlebar  titlebar'
    'sidebar   main      drawer-toggle'
    'sidebar   main      bottom-drawer';
  grid-template-columns: 48px 1fr auto;
  grid-template-rows: 36px 1fr auto;
  height: 100vh;
  overflow: hidden;
}

.titlebar    { grid-area: titlebar; }
.sidebar     { grid-area: sidebar; }
.main-panel  { grid-area: main; }
.drawer      { grid-area: bottom-drawer; max-height: 40vh; overflow-y: auto; }
.drawer-toggle { grid-area: drawer-toggle; writing-mode: vertical-lr; }
```

### Resizable panels:
Use `allotment` (a 4KB React split-pane library) for the main/sidebar/drawer resize handles. No need for `react-resizable-panels` (20KB).

### Custom titlebar (`window.decorations: false`):
- `tauri.conf.json`: `"decorations": false`
- `data-tauri-drag-region` on the titlebar element
- Windows: 36px height for standard minimize/maximize/close look
- macOS: respect `--apple-titlebar-height` dynamically via Tauri's `onResized` / theme detection
- Linux: KDE/GNOME titlebar buttons if needed

```tsx
<div className={styles.titlebar} data-tauri-drag-region>
  <div className={styles.titlebarButtons}>
    <button onClick={() => invoke('minimize_window')}>─</button>
    <button onClick={() => invoke('toggle_maximize_window')}>□</button>
    <button onClick={() => invoke('close_window')}>✕</button>
  </div>
</div>
```

**Binary impact:** <1KB for `allotment`.

---

## 6. Streaming Text Rendering

**Recommendation: Diff-based incremental updates with animated cursor**

### The problem:
- Directly setting `innerHTML` every frame causes layout thrash and flicker.
- React's `setState` batched updates don't play well with high-frequency streaming.

### Implementation:

```tsx
function useStreamingText(ref: RefObject<HTMLPreElement>) {
  const bufferRef = useRef('');
  const isAnimating = useRef(false);
  const frameId = useRef(0);

  const appendChunk = useCallback((chunk: string) => {
    bufferRef.current += chunk;
    if (!isAnimating.current) {
      isAnimating.current = true;
      const animate = () => {
        if (!ref.current) return;
        const existing = ref.current.textContent || '';
        // Only append what's new (diff-based)
        const newText = bufferRef.current;
        if (newText.startsWith(existing)) {
          const newPart = newText.slice(existing.length);
          if (newPart) {
            // Use insertAdjacentText — no HTML parsing, no re-render
            ref.current.insertAdjacentText('beforeend', newPart);
          }
        } else {
          // Reset on divergent streams (edge case)
          ref.current.textContent = newText;
        }
        isAnimating.current = false;
      };
      frameId.current = requestAnimationFrame(animate);
    }
  }, [ref]);

  useEffect(() => () => cancelAnimationFrame(frameId.current), []);

  return { appendChunk };
}
```

### Cursor animation:
```css
@keyframes blink {
  50% { opacity: 0; }
}

.token-cursor::after {
  content: '▎';
  animation: blink 0.8s step-end infinite;
  color: var(--accent);
}
```

### Performance budget:
- Direct DOM manipulation via `insertAdjacentText` skips React reconciliation entirely
- No diff computation on the client — the stream is append-only
- `requestAnimationFrame` caps updates to 60fps even if chunks arrive faster

---

## 7. Agent State Indicators

**Recommendation: State-machine-driven phases with transitions, not spinners**

### AgentKit state machine (expected):

```typescript
type AgentPhase =
  | { type: 'idle' }
  | { type: 'thinking'; detail?: string }
  | { type: 'gathering_context'; tool?: string }
  | { type: 'running_tool'; tool: string; args: unknown }
  | { type: 'reviewing_output' }
  | { type: 'error'; message: string }
  | { type: 'awaiting_human'; reason: string };
```

### UI component:

```tsx
function AgentStatusIndicator({ agent }: { agent: AgentState }) {
  const phaseIcon = {
    idle: '●',
    thinking: '◐',
    gathering_context: '◎',
    running_tool: '⚡',
    reviewing_output: '◉',
    error: '⚠',
    awaiting_human: '✋',
  }[agent.phase.type];

  const phaseColor = {
    idle: 'var(--text-muted)',
    thinking: 'var(--accent)',
    running_tool: 'var(--warning)',
    error: 'var(--danger)',
    awaiting_human: 'var(--info)',
    default: 'var(--accent)',
  };

  return (
    <span className={styles.indicator} style={{ color: phaseColor[agent.phase.type] ?? phaseColor.default }}>
      <span className={styles.phaseIcon}>{phaseIcon}</span>
      <span className={styles.phaseLabel}>{formatPhaseLabel(agent.phase)}</span>
    </span>
  );
}
```

### Transitions:
- Use CSS `transition: opacity 0.2s, transform 0.2s` for phase change — no Framer Motion needed for simple fade.
- For tool execution progress, show an expanding/collapsing `<details>`-like panel that reveals `{ tool, args, elapsed }`.
- Use `@tanstack/react-virtual` for the agent list sidebar if there are more than 8 agents.

### Agent position mapping:
For the overview, use a **Kanban-style phase lane** (optional, v1.5):
- Columns: Idle | Working | Waiting | Done
- Drag an agent card between lanes? No — it's automation, not a task board. The system owns phases.

---

## 8. Dark Mode First

**Recommendation: Dark mode is the default. Light mode = optional accessibility toggle.**

### Design choices:

```css
:root {
  color-scheme: dark;
  --surface: #0e0e10;
  --surface-raised: #1a1a1e;
  --surface-overlay: #242428;
  --border: #2a2a30;
  --text-primary: #e4e4e7;
  --text-secondary: #a1a1aa;
  --text-muted: #71717a;
  --accent: #8b5cf6;       /* purple */
  --accent-soft: #6d28d9;
  --danger: #ef4444;
  --warning: #f59e0b;
  --success: #22c55e;
  --info: #3b82f6;
}

[data-theme="light"] {
  color-scheme: light;
  --surface: #ffffff;
  --surface-raised: #f9fafb;
  --surface-overlay: #f3f4f6;
  --border: #e5e7eb;
  --text-primary: #111827;
  --text-secondary: #4b5563;
  --text-muted: #9ca3af;
  --accent: #7c3aed;
  --accent-soft: #8b5cf6;
}
```

### Accessibility requirements:
- WCAG 2.1 AA: contrast ratio ≥ 4.5:1 for normal text, ≥ 3:1 for large text
- With dark default: purple `#8b5cf6` on `#0e0e10` = 5.2:1 ✅
- Light mode: purple `#7c3aed` on `#ffffff` = 5.7:1 ✅
- Test with [Accessible Color Matrix](https://toolness.github.io/accessible-color-matrix/) in CI
- Respect `prefers-color-scheme`:
```css
@media (prefers-color-scheme: light) {
  :root { /* opt-out: default stays dark unless user toggles */ }
}
```

### Toggle behavior:
- Store preference in `localStorage` or Tauri's `appHandle.state()`
- Sync with `window.matchMedia('(prefers-color-scheme: dark)')` for initial detection
- Provide a UI toggle in settings (gear icon in navbar)

---

## 9. Binary Size Management

**Recommendation: Track to 3-8MB target with a size budget per layer**

### Budget breakdown (gzipped, for Tauri bundle):

| Layer | Budget | Notes |
|---|---|---|
| Rust binary (release, LTO) | 3-5MB | MinGW/MSVC stripped |
| Frontend JS bundle (Vite) | 600KB | React 18 + Radix + Lucide + app code |
| Frontend CSS | 40KB | No Tailwind → native CSS + Open Props subset |
| WebView runtime (WebView2) | 0KB | Bundled with Windows; macOS/Linux system |
| Fonts | 100KB | 3 variable fonts in .woff2 |
| Icons | 5KB | Tree-shaken Lucide + custom sprite |
| Docker image (first download) | ~200MB | Not bundled — downloaded on demand |
| **Total Tauri bundle** | **~3.7-5.7MB** | Well under 15MB ceiling |

### Optimization rules:

1. **ESM tree-shaking** — Import only what you use (Vite does this automatically)
2. **Code splitting** — Async-load panels that aren't in view (drawer, settings, agent details) via `React.lazy()`
3. **NO Moment.js, lodash, date-fns** — Use native `Intl`, `Date`, or a <1KB helper
4. **NO chart libraries** — Use CSS + SVG for simple charts (tool timing, agent state history)
5. **Font subsetting** — If a font has many glyphs, subset to Latin + common Unicode ranges using `glyphhanger` or `pyftsubset`
6. **Rust side** — Enable LTO, `opt-level = "z"`, strip symbols:
   ```toml
   [profile.release]
   lto = true
   codegen-units = 1
   opt-level = "z"
   strip = true
   ```
7. **WebView2 fixed version** — Use the fixed Evergreen WebView2 bootstrapper (~2MB) not the full installer; or let Windows provide it

### What NOT to sacrifice:
- Do not strip accessibility (Radix is free; ARIA labels cost no KB)
- Do not use unreadable minified class names that break screen reader context

---

## Summary Table

| Concern | Recommendation | Binary Cost |
|---|---|---|
| CSS Framework | CSS Modules + Custom Properties + Open Props subset | ~40KB |
| UI Libraries | Radix UI + Floating UI + Framer Motion (subset) | ~80KB |
| Fonts | 3 variable .woff2 fonts (Geist, JetBrains Mono, Instrument Sans) | ~100KB |
| Icons | Lucide (tree-shaken) + custom SVG sprite | ~5KB |
| Layout | CSS Grid + `allotment` + custom titlebar | ~1KB |
| Streaming | Diff-based `insertAdjacentText` + rAF cursor | ~2KB |
| Agent Indicators | CSS transitions + phase icons | ~3KB |
| Dark mode | CSS custom properties, dark first | ~0KB (tokens) |
| Binary target | **~3.7-5.7MB** (well under 15MB ceiling) | — |

**Total estimated binary impact of frontend additions: ~231KB — leaving plenty of headroom.**

---

## Implementation Risk Matrix

| Risk | Impact | Likelihood | Mitigation |
|---|---|---|---|
| Custom titlebar window controls break on Linux | Medium | Low | Use `window.decorations: false` and Tauri's `onCloseRequested`; test on Gnome/KDE |
| Streaming text jank with rapid LLM token bursts | Low | Medium | RequestAnimationFrame caps at 60fps; fallback to batch updates every 50ms |
| `@container` queries not supported in older WebView2 | Medium | Low (WebView2 auto-updates) | Use `max-width` fallback in `@supports not (container-type: inline-size)` |
| Font flash (FOUT) on first load | Low | Low | `font-display: swap` + `preload` link headers via Tauri |
| Radix Dialog portal escapes Tauri window bounds | Medium | Low | Use Floating UI's `autoUpdate` to reposition on boundary collision |

---

## Action Items for Implementation

1. [ ] Create `src/styles/design-tokens.css` with custom properties for the dark theme
2. [ ] Install Radix UI primitives and wrap in `components/ui/` modules
3. [ ] Add `.woff2` fonts and `@font-face` declarations
4. [ ] Set up Lucide with tree-shaken imports
5. [ ] Build `AppShell` with CSS Grid + `allotment` for panel resize
6. [ ] Implement `useStreamingText` hook for LLM output
7. [ ] Create `AgentStatusIndicator` component with phase display
8. [ ] Add light mode `data-theme` toggle + `prefers-color-scheme` detection
9. [ ] Configure `tauri.conf.json` for `decorations: false` + custom titlebar
10. [ ] Run initial `npm run build` and verify bundle size with `vite-bundle-visualizer`

---
