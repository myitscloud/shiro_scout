# MSPEC-014 · Wire Sandbox Pill with Real Docker Status

**Status:** Draft · **Owner:** Frontend Engineer · **Ring:** 2

## Goal
Replace the hardcoded "healthy" status and static blue SVG in the Navbar sandbox pill with real-time Docker container status from `AppContext.dockerInfo` (which comes from the Rust backend via `invoke('get_docker_info')`).

## Files in Scope
| File | Change |
|------|--------|
| `src/components/Layout/Navbar.tsx` | Accept `sandboxStatus` prop, use it for dynamic dot color, icon color, title text, and label text |
| `src/components/Layout/Navbar.module.css` | Add pill status color classes if needed |
| `src/App.tsx` | Pass `dockerInfo.status` to Navbar as `sandboxStatus`, add periodic `refreshDockerStatus` polling |

## Requirements

### Visual States

| Status | Dot Color | Icon Color | Title Text | Pill Label |
|--------|:---------:|:----------:|------------|------------|
| `available` | Green `#22c55e` with glow | `#22c55e` | "Sandbox: aegis-sbx (vX.Y.Z) · running" | "Sandbox" or "aegis-sbx" |
| `checking` | Yellow `#f59e0b` with pulse animation | `#f59e0b` | "Sandbox · checking..." | "Checking…" |
| `unavailable` | Red `#ef4444` | `#ef4444` | "Sandbox · unavailable — Docker daemon not found" | "No sandbox" |
| `error` | Red `#ef4444` | `#ef4444` | "Sandbox · error — {dockerInfo.error}" | "Error" |

### NavbarProps Interface Changes
```typescript
export interface NavbarProps {
  // ... existing props
  sandboxStatus?: 'checking' | 'available' | 'unavailable' | 'error';
  sandboxVersion?: string | null;
}
```

### App.tsx Changes
1. Pass `sandboxStatus={dockerInfo.status}` and `sandboxVersion={dockerInfo.version}` to `<Navbar>`.
2. Add a `useEffect` that polls `refreshDockerStatus()` every 30 seconds using `setInterval`. Clean up on unmount.
3. The existing `sandboxLabel` prop can be derived inside Navbar from status + version instead of passed down.

### CSS Changes (if needed)
Add status classes for `.pill` or a container inside the sandbox pill:
```css
.pill.online { --pill-dot: var(--status-online); }
.pill.offline { --pill-dot: var(--status-error); }
```

## Acceptance
1. Green dot + icon when Docker is available and container is running.
2. Red dot + icon when Docker is unavailable or errored.
3. Yellow dot + pulse animation when status is 'checking'.
4. Title text shows real status, not hardcoded 'healthy'.
5. Periodic 30-second polling pings `refreshDockerStatus()`.
6. `npx tsc --noEmit` passes with exit 0.
7. `pnpm build` passes with exit 0.

## Current State (lines 41-51 of Navbar.tsx)
```tsx
<button className={styles.pill} title={`Sandbox: ${sandboxLabel} · healthy`}>
  <svg width="13" height="13" viewBox="0 0 24 24" fill="currentColor" style={{color:'#4A9FE0'}}><path d="M4 11h2v2H4zm3 0h2v2H7zm3 0h2v2h-2zm-3-3h2v2H7zm3 0h2v2h-2zm0-3h2v2h-2zM2 14s.7 4.5 5.5 4.5c6.6 0 9.8-3.2 11-5 0 0 3 .4 3.5-1.5-1-.8-2.6-.6-2.6-.6s.2-1.5-1.4-2.4c-.9 1-.8 2.4-.8 2.4H2z"/></svg>
  <span className={`${styles.dot}`} style={{width:6,height:6}}></span> <span>Sandbox</span>
</button>
```

The `sandboxLabel` prop already carries the real label (e.g. "aegis-sbx (v29.6.1)" or "no sandbox") from App.tsx, but the dot, icon color, and title text are all static.

## Wiring Summary
1. Add `sandboxStatus` and `sandboxVersion` to NavbarProps.
2. In the sandbox pill: dot color + SVG fill = `var(--status-online)` for available, `var(--status-error)` for unavailable/error, `var(--status-thinking)` for checking.
3. Remove hardcoded `· healthy` from title; use status text like "running" / "unavailable" / "checking...".
4. In App.tsx: pass status + version as new props, add polling interval.
5. Verify compile + build.
