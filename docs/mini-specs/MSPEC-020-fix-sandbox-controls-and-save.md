# MSPEC-020 · Fix Sandbox Controls and Workspace Save

**Owner:** Frontend Engineer · **Ring:** 2

## Problem 1: Navbar Sandbox Control Dropdown Buttons Don't Work

The sandbox control dropdown (▼ chevron) has Start/Restart/Stop buttons that call `onStartSandbox`, `onRestartSandbox`, `onStopSandbox` callbacks. But `App.tsx` does NOT pass these callbacks to `<Navbar>` — they are undefined, so clicking the buttons is a silent no-op.

## Problem 2: Settings "Apply & Restart Sandbox" Doesn't Save/Restart

Container is `Exited (1)` — crashed. The `handleApplyRestart` function needs to: stop → remove → create (with new workspace path) → start. It may fail if remove doesn't work on an exited container, or the create fails with 409 because a container with that name already exists.

## Fix 1: Wire Callbacks in App.tsx

In the `<Navbar>` rendering section (around line 374), add these three props:

```typescript
<Navbar
  // ... existing props
  onStartSandbox={() => invoke('start_sandbox', { containerId: 'aegis-sandbox' })}
  onRestartSandbox={async () => {
    await invoke('restart_sandbox', { containerId: 'aegis-sandbox' });
    await refreshDockerStatus();
  }}
  onStopSandbox={async () => {
    await invoke('stop_sandbox', { containerId: 'aegis-sandbox' });
    await refreshDockerStatus();
  }}
/>
```

Check if `restart_sandbox` exists as a Tauri command. If not, implement it manually as stop + start:
```typescript
onRestartSandbox={async () => {
  await invoke('stop_sandbox', { containerId: 'aegis-sandbox' });
  await invoke('start_sandbox', { containerId: 'aegis-sandbox' });
  await refreshDockerStatus();
}}
```

Make sure `invoke` is imported from `@tauri-apps/api/core` (already imported at line 18) and `refreshDockerStatus` is destructured from `useAppContext()` (already at line 167).

## Fix 2: Add `restart_sandbox` Rust Command (if missing)

Check `container.rs` for a `restart_sandbox` command. If missing, add it:
```rust
#[tauri::command]
pub async fn restart_sandbox(container_id: String) -> Result<(), String> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| format!("Failed to connect to Docker daemon: {}", e))?;
    docker.restart_container::<String>(&container_id, None)
        .await
        .map_err(|e| format!("Failed to restart container '{}': {}", container_id, e))?;
    Ok(())
}
```

And register it in `lib.rs` generate_handler!

## Fix 3: Settings Save — Ensure container removal + recreation works

The `handleApplyRestart` in Settings.tsx already does:
1. `stopContainer('aegis-sandbox')`
2. `removeContainer('aegis-sandbox')`
3. `createContainer({ workspace_path: workspacePath || '', ... })`
4. `startContainer('aegis-sandbox')`
5. `refreshDockerStatus()`

Add error logging for each step so we can see which step fails. If `stopContainer` fails on an already-exited container, wrap it in try/catch.

## Acceptance
1. Sandbox pill dropdown Start/Restart/Stop buttons actually call the Docker API.
2. Settings workspace path input saves and restarts the container with `--volume C:\Path:/workspace:rw`.
3. Container moves from `Exited (1)` to `Running`.
4. Agent can `ls /workspace` and see Windows files.
5. `npx tsc --noEmit` passes exit 0.
6. `pnpm build` passes exit 0.

## Files in Scope
| File | Change |
|------|--------|
| `src/App.tsx` | Add `onStartSandbox`, `onRestartSandbox`, `onStopSandbox` callbacks to `<Navbar>` |
| `src/components/Settings/Settings.tsx` | Add error logging to `handleApplyRestart` |
| `src-tauri/src/container.rs` | Add `restart_sandbox` command if missing |
| `src-tauri/src/lib.rs` | Register `restart_sandbox` in `generate_handler![]` if added |
