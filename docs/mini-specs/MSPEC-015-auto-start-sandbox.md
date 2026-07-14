# MSPEC-015 · Auto-Start Sandbox on App Launch

**Status:** Draft · **Owner:** Frontend Engineer · **Ring:** 2

## Goal
Automatically create and start the `aegis-sandbox` Docker container when the ShiroScout app launches, so the user doesn't have to manually trigger sandbox creation. One sandbox per app instance, reused across all chat sessions.

## Files in Scope
| File | Change |
|------|--------|
| `src/App.tsx` | Add a `useEffect` that on mount checks Docker state and auto-creates/starts the sandbox if needed |

## Logic (in a new `useEffect`)

```
On app mount:
1. Call refreshDockerStatus() to get current Docker state
2. Once known, check dockerInfo.status:
   - If 'checking': wait briefly, then recheck
   - If 'available' AND dockerInfo.containers is empty:
       ▶ invoke('create_sandbox', defaultConfig)  // creates container
       ▶ invoke('start_sandbox', container_id)     // starts it
   - If 'available' AND containers exist but stopped:
       ▶ invoke('start_sandbox', container_id)     // restart it
   - If 'available' AND container is running:
       ▶ do nothing (already healthy)
   - If 'unavailable' or 'error':
       ▶ do nothing (Docker missing — show red pill)
```

## Acceptance
1. When app opens and Docker is available, sandbox container is automatically created and started within ~2 seconds.
2. If container already exists (from previous app session), it's started if stopped, or left alone if already running.
3. No duplicate container name errors (Docker should only ever try to create if no container exists).
4. `npx tsc --noEmit` passes with exit 0.
5. `pnpm build` passes with exit 0.

## Existing Context in App.tsx

- `dockerInfo` is destructured from `useAppContext()` (line 136)
- `refreshDockerStatus` is available (line 167)
- `invoke` is imported from `@tauri-apps/api/core` (line 18)
- A 30s polling `useEffect` already exists (lines 201-206)
- The container label is computed: `containerLabel = dockerInfo.status === 'available' ? 'aegis-sbx (v...)' : ...`

## Existing Rust Commands (already registered)
| Command | Signature |
|---------|-----------|
| `create_sandbox` | Creates a container from `aegis-sandbox:latest` |
| `start_sandbox` | Takes a `containerId: String`, starts the container |
| `stop_sandbox` | Takes a `containerId: String`, stops the container |
| `get_sandbox_status` | Returns container status info |

Use `invoke('create_sandbox')` with no args for default config (image: `aegis-sandbox:latest`, name: `aegis-sandbox`, memory: 2048 MB, cpu_shares: 512, network_mode: none).

Use `invoke('start_sandbox', { containerId: 'aegis-sandbox' })` to start it.

## Implementation Notes
- Put the new `useEffect` right after the existing 30s polling `useEffect` (after line 206).
- It should run once on mount (dependency array: `[]`), but reference `refreshDockerStatus`, `dockerInfo.status`, `dockerInfo.containers` through closures or state.
- Use a flag/ref like `hasAutoStarted = useRef(false)` to prevent double-invocation in Strict Mode.
