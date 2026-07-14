# MSPEC-016 · Sandbox Control Dropdown Menu

**Status:** Draft · **Owner:** Frontend Engineer · **Ring:** 2

## Goal
Add a chevron `▼` button next to the sandbox pill in the Navbar that opens a dropdown menu with sandbox control actions (Start, Restart, Stop) and status information.

## Files in Scope
| File | Change |
|------|--------|
| `src/components/Layout/Navbar.tsx` | Add chevron button + dropdown menu with controls |
| `src/components/Layout/Navbar.module.css` | Add dropdown styling, positioning, animation |

## Layout

```
[ 🟢 Sandbox ▼ ]
          ┌──────────────────────┐
          │ ├── Container: aegis-sandbox │
          │ ├── Status: running         │
          │ ├── Image: aegis-sandbox:latest │
          │ ├── Uptime: 12m             │
          │ ────────────────────── │
          │ │ ▶ Start               │
          │ │ 🔄 Restart             │
          │ │ ⏹ Stop                │
          └──────────────────────┘
```

The chevron `▼` is a small arrow icon inside the pill or immediately adjacent to it. When clicked, it opens a dropdown menu below the pill. The pill itself (the main body) still shows status and can be hovered for tooltip as before.

## Props

No new props needed — `sandboxStatus`, `sandboxVersion`, and `sandboxLabel` are already available. Add three new callback props:

```typescript
interface NavbarProps {
  // ... existing props
  onStartSandbox?: () => void;
  onRestartSandbox?: () => void;
  onStopSandbox?: () => void;
}
```

## Buehler's Layout Decision

Option C was chosen over A (inline buttons) and B (dropdown on pill click).

## State Management

- Local state `showDropdown: boolean` in Navbar.tsx
- Clicking the chevron toggles the dropdown
- Clicking an action button calls the callback and closes the dropdown
- Clicking outside the dropdown closes it (useEffect with mousedown listener)

## Styling (Navbar.module.css)

| Element | CSS |
|---------|-----|
| Chevron button | `btn icon ghost` (matching other icon buttons in the Navbar) |
| Dropdown container | Absolutely positioned below the pill. Background: glass-morphism layer matching Neo-Glass-Terminus. Border: `1px solid rgba(255,255,255,0.1)`. Border-radius: `8px`. Padding: `8px`. Min-width: `220px`. |
| Status info lines | Small, muted text showing container name, status, image, uptime |
| Action buttons | Full-width rows with icon + label. Hover: accent background. Each button calls the corresponding `invoke` directly or through the callback. |

## Actions (Rust backend integration)

Each action calls the corresponding Tauri command via `invoke()`:

| Button | invoke call |
|--------|-------------|
| ▶ Start | `invoke('start_sandbox', { containerId: 'aegis-sandbox' })` |
| 🔄 Restart | `invoke('stop_sandbox', { containerId: 'aegis-sandbox' })` then `invoke('start_sandbox', { containerId: 'aegis-sandbox' })` |
| ⏹ Stop | `invoke('stop_sandbox', { containerId: 'aegis-sandbox' })` |

## Acceptance
1. Chevron ▼ appears next to the sandbox pill on the right side.
2. Clicking the chevron opens a dropdown below the pill.
3. The dropdown shows: container name, status, image, uptime (when available).
4. The dropdown shows Start, Restart, Stop action buttons.
5. Clicking an action calls the corresponding `invoke` and closes the dropdown.
6. Clicking outside the dropdown closes it.
7. `npx tsc --noEmit` passes exit 0.
8. `pnpm build` passes exit 0.
9. No new dependencies.
