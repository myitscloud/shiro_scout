# MSPEC-021: Workspace Folder Picker with Auto-Creation and Default Mount

**Status:** Approved
**Date:** 2026-07-12
**Assignee:** Full Stack

## Goal
Set `C:\\projects` as the default workspace mount path, auto-create the folder if it does not exist on Windows, and ensure the Settings → Workspace mount flow works end-to-end across all layers.

## Root Cause Analysis

The workspace mount pipeline had two defects:

### Defect 1: Rust `AppSettings` missing `workspace_path` field

The Rust struct `AppSettings` in `src-tauri/src/settings.rs` serializes/deserializes settings to disk. It did NOT have `workspace_path` — meaning when the frontend saved settings, the workspace path was silently dropped. On next launch, `load_settings` returned a deserialized struct without workspace_path, so the frontend fell back to `DEFAULT_SETTINGS.workspacePath = ''`, producing no volume mount.

**Fix:** Add `workspace_path: String` to `AppSettings` struct and its `Default` impl.

### Defect 2: No WORKDIR in Dockerfile

The `Dockerfile.sandbox` had no `WORKDIR` directive. When commands run inside the container via `ToolExecBridge.exec_shell_in_container()`, they execute in the default working directory `/`, not `/workspace`. Even with the volume mount present, `ls` or `pwd` would show the root filesystem.

**Fix:** Add `WORKDIR /workspace` to `Dockerfile.sandbox` and `cd /workspace` in `entrypoint.sh` before the bridge starts.

### Defect 3: Invoke arg name mismatch

`App.tsx` called `invoke('start_sandbox', { containerId: 'aegis-sandbox' })` but Rust expects `{ id: 'aegis-sandbox' }`. Tauri uses the Rust parameter name as the JSON key. The wrong key was silently ignored.

### Defect 4: mount_workspace checkbox not gating auto-start

The auto-start always passed `workspacePath` regardless of the `mount_workspace` boolean flag.

## Implementation

### Layer 1: Docker Image (Dockerfile.sandbox)

Add `WORKDIR /workspace` before `ENTRYPOINT`:

```dockerfile
WORKDIR /workspace
ENTRYPOINT ["/opt/shiroscout/entrypoint.sh"]
```

### Layer 2: Entrypoint (entrypoint.sh)

Add `cd /workspace` before starting the bridge:

```bash
cd /workspace 2>/dev/null || echo "[entrypoint] /workspace not mounted — running in /"; pwd
```

### Layer 3: Rust Backend (settings.rs)

Add `workspace_path: String` to `AppSettings` struct and default:

```rust
pub struct AppSettings {
    pub theme: String,
    pub reduce_motion: bool,
    pub provider: String,
    pub model: String,
    pub api_key: String,
    pub sandbox_on_launch: bool,
    pub workspace_path: String,  // NEW
    pub mount_workspace: bool,
    pub last_session_id: Option<String>,
    pub hitl_timeout_secs: u32,
    pub dangerous_operations: DangerousOperationsConfig,
    pub sandbox_air_gapped: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            reduce_motion: false,
            provider: "local".to_string(),
            model: "gpt-4o".to_string(),
            api_key: String::new(),
            sandbox_on_launch: true,
            workspace_path: String::new(),  // Frontend provides the actual default
            mount_workspace: true,
            last_session_id: None,
            hitl_timeout_secs: 30,
            dangerous_operations: DangerousOperationsConfig::default(),
            sandbox_air_gapped: true,
        }
    }
}
```

### Layer 4: Frontend Default (tauri-commands.ts)

Change default workspace path to `C:\\projects`:

```typescript
export const DEFAULT_SETTINGS: AppSettings = {
  theme: 'dark',
  workspacePath: 'C:\\projects',  // was ''
  reduce_motion: false,
  provider: 'local',
  model: 'deepseek-v4-flash',
  api_key: '',
  sandbox_on_launch: true,
  mount_workspace: true,
  last_session_id: null,
};
```

### Layer 5: Invoke Arg Fix (App.tsx)

Fix Tauri invoke arg name to match Rust signature:
```typescript
// Before:
await invoke('start_sandbox', { containerId: 'aegis-sandbox' });
// After:
await invoke('start_sandbox', { id: 'aegis-sandbox' });
```

### Layer 6: mount_workspace Gating (App.tsx)

Gate the workspace path with the mount_workspace checkbox:
```typescript
const wsPath = settingsRef.current.mount_workspace 
  ? settingsRef.current.workspacePath || '' 
  : '';
await createSandbox({
  image: 'aegis-sandbox:latest',
  workspace_path: wsPath,
  memory_mb: 2048,
  cpu_shares: 512,
  network_mode: 'none'
});
```

### Layer 7: Folder Auto-Creation (Rust Tauri Command)

Add a new Tauri command in Rust that ensures the workspace directory exists:

```rust
#[tauri::command]
pub fn ensure_workspace_dir(path: String) -> Result<String, String> {
    let p = std::path::Path::new(&path);
    if p.exists() {
        Ok(format!("Workspace '{}' already exists", path))
    } else {
        std::fs::create_dir_all(p)
            .map_err(|e| format!("Failed to create workspace directory '{}': {}", path, e))?;
        Ok(format!("Created workspace directory '{}'", path))
    }
}
```

Wire this into the auto-start and Settings Apply & Restart flow after saving settings but before creating the container.

## Testing

- `npx tsc --noEmit` → exit 0
- `pnpm build` → exit 0
- `cargo build` → exit 0
- Container mounts `C:\\projects` at `/workspace` with RW permissions
- Agent can `ls /workspace` and see host files
- Settings → change path → Apply & Restart → container recreated with new mount
- `mount_workspace` unchecked → container launched without volume mount

## Acceptance Criteria

- [ ] `C:\\projects` is the default workspace path
- [ ] `C:\\projects` is auto-created if missing
- [ ] Rust `AppSettings` includes `workspace_path` and persists across restarts
- [ ] Docker image has `WORKDIR /workspace`
- [ ] Entrypoint `cd /workspace` before bridge
- [ ] `start_sandbox` invoke uses correct `{ id: ... }` arg name
- [ ] `mount_workspace` checkbox controls whether volume mount is applied
- [ ] All TypeScript, Rust, and frontend builds pass
- [ ] User sees clear toast message when workspace folder is created or if creation fails
