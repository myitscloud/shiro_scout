# MSPEC-018 · Fix Auto-Start 409 Container Name Conflict

**Owner:** Frontend Engineer · **Ring:** 2

## Problem

After the `create_sandbox` config fix, the auto-start `useEffect` now correctly passes config, but it calls `create_sandbox` unconditionally. If a container named `aegis-sandbox` already exists (from a prior launch), Docker returns 409 Conflict and the auto-start fails.

## Root Cause

The auto-start logic (in App.tsx auto-start useEffect) does:
```typescript
if (containerInfo.length === 0) {
  await invoke('create_sandbox', { config: { ... } });
}
await invoke('start_sandbox', { containerId: 'aegis-sandbox' });
```

This `containerInfo` comes from `invoke('get_sandbox_status')` which may not return the same set. The more reliable check is to look at `dockerInfo.containers` from the already-called `refreshDockerStatus()` or simply check if `get_sandbox_status` returns any containers.

However, the current code reads `containerInfo` from `get_sandbox_status` which seems to not find the running container, so it tries to create again.

## Files in Scope
| File | Change |
|------|--------|
| `src/App.tsx` | Fix auto-start logic: only create if container doesn't exist, start regardless |

## Fix

In the auto-start `useEffect`, change the logic to:

```typescript
// After t3 timeout fires at 1200ms and we call refreshDockerStatus()...
try {
  const daemonStatus = await invoke('check_docker_daemon');
  const status = (daemonStatus as { available: boolean; ... }).available ? 'available' : 'unavailable';
  
  // Try to start first — create only if start fails with 'not found'
  try {
    await invoke('start_sandbox', { containerId: 'aegis-sandbox' });
  } catch (startErr) {
    // Container doesn't exist yet — create then start
    await createSandbox({
      image: 'aegis-sandbox:latest',
      workspace_path: '',
      memory_mb: 2048,
      cpu_shares: 512,
      network_mode: 'none'
    });
    await invoke('start_sandbox', { containerId: 'aegis-sandbox' });
  }
  
  await refreshDockerStatus();
} catch (err) {
  console.error('Sandbox auto-start failed:', err);
} finally {
  setSandboxBootPhase(null);
}
```

The key change: **try start first**. If the container exists (created or stopped), it starts successfully. If it doesn't exist, the start fails with a "not found" error — then we create and start. This avoids the 409 conflict entirely.

## Acceptance
1. If container `aegis-sandbox` already exists and is stopped: start it, no error.
2. If container `aegis-sandbox` already exists and is running: start succeeds silently, no error.
3. If container `aegis-sandbox` does not exist: create it, then start it.
4. No 409 Conflict errors in console.
5. `npx tsc --noEmit` passes exit 0.
6. `pnpm build` passes exit 0.
