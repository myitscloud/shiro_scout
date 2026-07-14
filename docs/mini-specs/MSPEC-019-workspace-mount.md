# MSPEC-019 · Workspace Mount — Windows Files in Sandbox

**Owner:** Frontend Engineer · **Ring:** 2

## Goal
Mount a user-selected Windows folder (e.g. `C:\Projects`) into the Docker sandbox as `/workspace` so the agent inside the container can read and write the user's real files.

## Files in Scope
| File | Change |
|------|--------|
| `src/App.tsx` | Pass `workspace_path` from settings to `createSandbox()` call in auto-start |
| `src/tauri-commands.ts` | Add `workspace_path` to `createSandbox` function signature if not there |
| `src/components/Settings/Settings.tsx` | Add a "Workspace folder" text input + folder picker button |
| `src-tauri/src/container.rs` | Change `SandboxConfig::default()` to accept a runtime `workspace_path` |

> Scope rules per C14 / DONE-050: only the files listed above.

## Design

### Default Behavior
- On first launch, if no workspace path is configured, use `String::new()` (current behavior — no mount).
- The user can set the path in Settings panel.
- The path is saved to app settings (persisted across restarts).

### UI — Settings Panel
Add a new section in `Settings.tsx`:
```
┌─────────────────────────────────────────┐
│  📁 Workspace                            │
│  Path where the sandbox can access files │
│  ┌──────────────────────────────────┐    │
│  │ C:\Users\wayne\Projects         │ ... │
│  └──────────────────────────────────┘    │
│  [Browse...]  [Apply & Restart Sandbox]  │
│  Current: No workspace set               │
└─────────────────────────────────────────┘
```

The "Browse" button uses `dialog.open()` from `@tauri-apps/plugin-dialog` (if available) or a text input as fallback.

### Tauri Invoke — createSandbox config

The `createSandbox` call in auto-start currently uses hardcoded values. Change to accept `workspace_path` from settings:

```typescript
await createSandbox({
  image: 'aegis-sandbox:latest',
  workspace_path: settings.workspacePath || '',
  memory_mb: 2048,
  cpu_shares: 512,
  network_mode: 'none'
});
```

### Rust Backend — SandboxConfig default

Current default is empty workspace_path. The frontend now passes it explicitly, so the default isn't used — but make sure the Rust struct accepts it.

## Implementation Steps

1. Add `workspacePath` to settings state / persistence (AppContext or a simple `useState` + `useEffect` with `localStorage` or Tauri settings).
2. Add Workspace UI section to `Settings.tsx` with text input and optional browse button.
3. Wire the saved path into the auto-start `createSandbox` call.
4. When "Apply & Restart Sandbox" is clicked: stop container → create new one with updated volume mount → start.

## Acceptance
1. User sets workspace path in Settings → clicks Apply → sandbox restarts with `--volume C:\Path:/workspace:rw`.
2. Agent inside sandbox can `ls /workspace` and see the user's Windows files.
3. The auto-start reads the saved path on next app launch and uses it.
4. `npx tsc --noEmit` passes exit 0.
5. `pnpm build` passes exit 0.

## Note on Future Expansion
This spec uses a single workspace mount. Future versions could support multiple mounts (e.g. separate read-only mount for tools, read-write for project) — but that's out of scope for this initial implementation.
