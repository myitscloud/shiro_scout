# MSPEC-017 · Fix React StrictMode Boot Animation Stuck on RED

**Status:** Draft · **Owner:** Frontend Engineer · **Ring:** 2

## Problem

The sandbox boot animation (LED sequence: red → orange → yellow → green blink → null/solid green) is stuck permanently on the **booting-red** phase. The container is actually running fine — the issue is in the auto-start useEffect logic.

## Root Cause

In `src/App.tsx`, the auto-start useEffect uses a `hasAutoStarted` ref guard:

```typescript
const hasAutoStarted = useRef(false);
useEffect(() => {
  if (hasAutoStarted.current) return;  // <-- BUG
  hasAutoStarted.current = true;       // <-- BUG
  
  setSandboxBootPhase('booting-red');
  const t1 = setTimeout(() => setSandboxBootPhase('booting-orange'), 400);
  const t2 = setTimeout(() => setSandboxBootPhase('booting-yellow'), 800);
  const t3 = setTimeout(async () => { ... setSandboxBootPhase(null); }, 1200);
  
  return () => {
    clearTimeout(t1); clearTimeout(t2); clearTimeout(t3);
  };
}, []);
```

In React StrictMode, the effect runs twice:
1. **First mount**: `hasAutoStarted` → true, sets `booting-red`, starts timers
2. **Cleanup**: clears all timers (red is shown but never progresses)
3. **Second mount**: `hasAutoStarted` is already true → returns early (timers never restart)

Result: Boot phase is stuck on `booting-red` forever because:
- Timers from first mount were **cancelled by cleanup**
- Second mount **never restarts** timers because of the ref guard

## Fix

**Remove** the `hasAutoStarted` ref entirely. The cleanup function already handles StrictMode correctly — on the second mount, the effect re-runs and re-starts all timers fresh.

### Files in Scope
| File | Change |
|------|--------|
| `src/App.tsx` | Remove `hasAutoStarted` useRef + guard + setter |

### Before:
```typescript
const hasAutoStarted = useRef(false);
useEffect(() => {
  if (hasAutoStarted.current) return;
  hasAutoStarted.current = true;
  // ... timers ...
  return () => {
    clearTimeout(t1); clearTimeout(t2); clearTimeout(t3);
  };
}, []);
```

### After:
```typescript
useEffect(() => {
  setSandboxBootPhase('booting-red');
  const t1 = setTimeout(...);
  const t2 = setTimeout(...);
  const t3 = setTimeout(...);
  return () => {
    clearTimeout(t1); clearTimeout(t2); clearTimeout(t3);
  };
}, []);
```

Also remove the unused `useRef` import if `hasAutoStarted` was the only useRef in App.tsx (check first).

## Acceptance
1. Boot animation plays fully: red (0-400ms) → orange (400-800ms) → yellow (800-1200ms) → green blink → solid green (normal status).
2. `npx tsc --noEmit` passes exit 0.
3. `pnpm build` passes exit 0.
