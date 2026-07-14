# Workspace Mount Postmortem: Windows Files in Docker Sandbox

**Date:** 2026-07-12
**Author:** ShiroScoot AI Engineering Agent (orchestrator)
**Status:** Resolved ✅

## Executive Summary

The ShiroScout AI agent inside the Docker sandbox could not see files mounted from the Windows host (`C:\projects`) at `/workspace`. The Docker volume mount `-v C:\projects:/workspace:rw` was confirmed working at the infrastructure level, but the agent's exec commands ran in the default root directory `/` instead of `/workspace`, and the settings persistence pipeline silently dropped the workspace path on every app restart.

**Two root causes were identified and fixed across all four architecture layers.**

---

## Symptoms

1. User creates sandbox container with `docker run -d --network none -v C:\projects:/workspace aegis-sandbox`
2. `docker exec aegis-sandbox ls /workspace` shows real Windows files ✅
3. The Tauri app's AI agent runs `ls -la` but cannot see any files — returns root filesystem
4. Settings workspace path is left blank or reset to empty on app restart
5. `mount_workspace` checkbox in Settings appears to do nothing

---

## Root Cause Analysis

### Defect 1: Rust `AppSettings` missing `workspace_path` field (Persistence Pipeline)

**Location:** `src-tauri/src/settings.rs`

The Rust struct `AppSettings` serializes/deserializes app settings to/from `settings.json` in the app config directory. It did **not** have a `workspace_path` field:

```rust
// BUG: No workspace_path field
pub struct AppSettings {
    pub theme: String,
    pub sandbox_on_launch: bool,
    pub mount_workspace: bool,
    // ...
}
```

**Consequence:** When the frontend called `save_settings({ ..., workspacePath: 'C:\\projects', ... })`, the Rust backend deserialized only the fields it knew about. The `workspacePath` was **silently dropped**. When `load_settings` ran on next app launch, the returned struct had no workspace_path, so the frontend fell back to `DEFAULT_SETTINGS.workspacePath = ''` (empty string).

The empty `workspace_path` caused `build_host_config()` in `container.rs` to skip the bind mount entirely:

```rust
fn build_host_config(config: &SandboxConfig) -> HostConfig {
    let mut binds = Vec::new();
    if !config.workspace_path.is_empty() {  // <-- empty string → skip!
        binds.push(format!("{}:/workspace:rw", config.workspace_path));
    }
    // ...
}
```

### Defect 2: Dockerfile missing `WORKDIR /workspace` (Exec Pipeline)

**Location:** `src-tauri/docker/Dockerfile.sandbox`

The Docker image had no `WORKDIR` directive. When `ToolExecBridge` ran commands inside the sandbox via `exec_shell_in_container()`, they executed in the container's default working directory `/`, not `/workspace`. Even with the volume mount present:

- `ls` → shows root filesystem
- `pwd` → returns `/`
- file writes → land in `/`, not `/workspace`

### Defect 3: Invoke arg name mismatch (Frontend → Rust Call)

**Location:** `src/App.tsx` line 241

The auto-start `useEffect` called:
```typescript
await invoke('start_sandbox', { containerId: 'aegis-sandbox' });
```
But the Rust `start_sandbox` command signature expects:
```rust
pub async fn start_sandbox(id: String) -> Result<(), String>
```
Tauri uses the Rust parameter name (`id`) as the JSON key. The wrong key name was silently ignored and the command failed silently.

### Defect 4: `mount_workspace` checkbox not honored in auto-start

**Location:** `src/App.tsx` auto-start `useEffect` (~line 244)

The auto-start always passed the raw `workspacePath` regardless of the `mount_workspace` boolean flag. If the user unchecked "Mount /workspace read-write" in Settings, the mount was still applied.

---

## Fixes Applied

### Layer 1: Docker Image (`Dockerfile.sandbox`)

Added `WORKDIR /workspace` before `ENTRYPOINT` to ensure all exec commands start in `/workspace`:

```dockerfile
EXPOSE 8080
WORKDIR /workspace
ENTRYPOINT ["/opt/shiroscout/entrypoint.sh"]
```

### Layer 2: Entrypoint (`entrypoint.sh`)

Added `cd /workspace` before the bridge starts as a defensive fallback for containers launched without `--workdir`:

```bash
cd /workspace 2>/dev/null || echo "[entrypoint] /workspace not mounted — running in /"; pwd
```

### Layer 3: Rust Backend (`settings.rs`)

Added `workspace_path: String` field to `AppSettings` struct and its `Default` impl:

```rust
pub struct AppSettings {
    // ...
    pub workspace_path: String,   // NEW
    pub mount_workspace: bool,
    // ...
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            // ...
            workspace_path: String::new(),  // Frontend provides the actual default
            mount_workspace: true,
            // ...
        }
    }
}
```

This ensures `workspace_path` survives serialization/deserialization through `save_settings` / `load_settings`.

### Layer 4: Frontend (`tauri-commands.ts`, `App.tsx`)

**Default workspace path** (`src/tauri-commands.ts`):
```typescript
export const DEFAULT_SETTINGS: AppSettings = {
  // ...
  workspacePath: 'C:\\projects',  // was ''
  // ...
};
```

**Fixed invoke arg** (`src/App.tsx` line 241):
```typescript
await invoke('start_sandbox', { id: 'aegis-sandbox' });  // was containerId
```

**Added `mount_workspace` gating** (`src/App.tsx` ~line 244):
```typescript
const wsPath = settingsRef.current.mount_workspace 
  ? settingsRef.current.workspacePath || '' 
  : '';
await createSandbox({
  workspace_path: wsPath,
  // ...
});
```

### Docker Image Rebuild

```powershell
docker rm -f aegis-sandbox
docker build -t aegis-sandbox:latest -f src-tauri/docker/Dockerfile.sandbox src-tauri/docker/
```

---

## Verification Results

| Check | Pre-Fix | Post-Fix |
|-------|---------|----------|
| `docker inspect aegis-sandbox --format '{{json .Mounts}}'` `Source` | `C:\projects` ✅ | `C:\projects` ✅ |
| `docker inspect aegis-sandbox --format '{{json .Mounts}}'` `RW` | `true` ✅ | `true` ✅ |
| `docker exec aegis-sandbox ls /workspace` | Shows files ✅ | Shows files ✅ |
| `docker exec aegis-sandbox pwd` | `/` ❌ | `/workspace` ✅ |
| Settings persistence across restart | Lost ❌ | Preserved ✅ |
| `mount_workspace` checkbox honored | No ❌ | Yes ✅ |

---

## Files Changed

| File | Change | Layer |
|------|--------|-------|
| `src-tauri/docker/Dockerfile.sandbox` | Added `WORKDIR /workspace` | Docker |
| `src-tauri/docker/entrypoint.sh` | Added `cd /workspace` | Docker |
| `src-tauri/src/settings.rs` | Added `workspace_path: String` field + default | Rust |
| `src/tauri-commands.ts` | Changed default `workspacePath` to `'C:\\projects'` | Frontend |
| `src/App.tsx` | Fixed `containerId` → `id`, added `mount_workspace` gating | Frontend |

---

## Prevention Recommendations

1. **Rust struct forward-compatibility:** Add `#[serde(deny_unknown_fields)]` to `AppSettings` in debug builds to catch missing fields during development
2. **Add integration test for settings round-trip:** Rust test that serializes/deserializes `AppSettings` and verifies all fields survive
3. **Add healthcheck to sandbox:** A `/workspace` directory existence and writability check in the bridge health endpoint
4. **Frontend settings validation:** TypeScript validation that workspace path exists when `mount_workspace` is enabled, with user-facing error message

---

## Related Documents

- [MSPEC-019: Workspace Mount](mini-specs/MSPEC-019-workspace-mount.md)
- [MSPEC-020: Fix Sandbox Controls and Save](mini-specs/MSPEC-020-fix-sandbox-controls-and-save.md)
- [MSPEC-021: Workspace Folder Picker and Default Mount](mini-specs/MSPEC-021-workspace-folder-picker.md)
- [Docker Sandbox Setup Guide](./SANDBOX_SETUP_GUIDE.md)
