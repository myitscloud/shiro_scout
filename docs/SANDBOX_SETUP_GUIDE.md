# ShiroScout Docker Sandbox — Setup & Test Guide

> **Purpose:** Step-by-step process to clean up stale Docker state, rebuild the sandbox image, launch it, and verify end-to-end connectivity between the Tauri app and the sandbox container.
> **Last updated:** 2026-07-12

---

## Quick Reference

| Image | Tag | Status | Notes |
|-------|:---:|:------:|-------|
| `aegis-sandbox` | `latest` | ✅ Built (stale) | Hardcoded in `container.rs` — this is the app's default |
| `shiroscout-sandbox` | `latest`, `stable` | ✅ Built (stale) | Identical content, different image ID |
| `agent-bridge` binary | — | ✅ Pre-compiled (6.3MB ELF) | At `src-tauri/docker/agent-bridge` |

| Container | Image | Status | Notes |
|-----------|-------|:------:|-------|
| `zealous_hypatia` | `aegis-sandbox` | ❌ Exited (137) | Crashed — likely OOM or bridge startup failure |
| `aegis-sandbox` | `aegis-sandbox` | ⏸️ Created | Never started |

---

## Phase 1 — Cleanup Stale State

### Step 1.1: Remove stale containers
```powershell
# Remove the crashed container
docker rm -f zealous_hypatia

# Remove the never-started container
docker rm -f aegis-sandbox
```

### Step 1.2: Prune unused images (optional, saves disk space)
```powershell
# List all sandbox images
docker images | Select-String "sandbox"

# Remove the duplicate/unused shiroscout-sandbox if you want
docker rmi shiroscout-sandbox:latest shiroscout-sandbox:stable

# Prune dangling images
docker image prune -f
```

### Step 1.3: Verify cleanup
```powershell
docker ps -a --filter "name=sandbox|hypatia"
docker images --format "table {{.Repository}}\t{{.Tag}}\t{{.ID}}"
```
You should see: `aegis-sandbox:latest` still present, no dead containers.

---

## Phase 2 — Rebuild the Bridge Binary

### Background: Two bridge directories were found

| Directory | Purpose | Status |
|-----------|---------|:------:|
| `src-tauri/docker/bridge/` | **Canonical** — original Rust axum bridge source | ✅ Clean |
| `src-tauri/docker/bridge-build/` | Duplicate/corrupted rebuild attempt | ❌ Should be deleted |

Both have identical source code. The existing `agent-bridge` binary (6.3MB ELF) at `src-tauri/docker/agent-bridge` was built from `bridge/`.

### Step 2.1: Clean up bridge-build directory
```powershell
Remove-Item -Recurse -Force "src-tauri\docker\bridge-build"
```

### Step 2.2: Rebuild the bridge binary

The bridge is a Rust axum HTTP server that runs INSIDE the sandbox container (target: `x86_64-unknown-linux-gnu` — native Linux, NOT Windows).

Choose ONE method:

**Option A — Build on the Agent Zero container (if Docker-in-Docker is available):**
```bash
cd /a0/usr/projects/shiro_scout/src-tauri/docker/bridge
cargo build --release --target x86_64-unknown-linux-gnu
cp target/x86_64-unknown-linux-gnu/release/agent-bridge ../agent-bridge
```

**Option B — Build on Windows host (requires WSL or MSYS2 with Rust toolchain):**
```powershell
# From the project root
Set-Location src-tauri/docker/bridge
$env:CARGO_TARGET_DIR = "$(Get-Location)/target"
cargo build --release
Copy-Item "target/release/agent-bridge" "../agent-bridge" -Force
```

### Step 2.3: Verify the binary
```powershell
# Check it's a Linux ELF (not Windows PE)
Get-Item "src-tauri/docker/agent-bridge" | Format-Table Length, LastWriteTime
# Should be ~6.3MB
```

---

## Phase 3 — Rebuild the Sandbox Image

### Step 3.1: Review Dockerfile.sandbox

The image is defined in `src-tauri/docker/Dockerfile.sandbox`:
- **Base:** `debian:bookworm-slim`
- **Content:** Python 3, Node 22, Playwright/Chromium, Xvfb, Xfce, VNC
- **Entrypoint:** `entrypoint.sh` → starts Xvfb → starts bridge
- **Bridge binary:** Copied from `agent-bridge` at build time

### Step 3.2: Build the image
```powershell
Set-Location "src-tauri\docker"
docker build -t aegis-sandbox:latest -f Dockerfile.sandbox .
```

This will:
1. Install all Debian packages (ca-certificates, curl, git, python3, nodejs, xvfb, etc.)
2. Install Node 22 from NodeSource
3. Create Python venv at `/opt/agent-venv`
4. Install Playwright + Chromium
5. Copy `agent-bridge` binary → `/opt/shiroscout/agent-bridge`
6. Copy `entrypoint.sh` → `/opt/shiroscout/entrypoint.sh`

> **Note:** First build takes 5-10 minutes due to apt and npm packages.
> Subsequent builds are faster thanks to Docker layer caching.

### Step 3.3: Verify the image
```powershell
docker images aegis-sandbox:latest
docker inspect aegis-sandbox:latest --format '{{.Size}}' | % { [math]::Round($_ / 1MB, 1) }
# Expected: ~1.5-2.0 GB
```

---

## Phase 4 — Launch the Sandbox Container

### Step 4.1: Create and start the container

**Method A — Via Tauri app UI (if app is running):**
1. The app's `create_sandbox()` command (registered in `container.rs`) uses:
   - Image: `aegis-sandbox:latest`
   - Container name: `aegis-sandbox`
   - Network mode: `none` (air-gapped)
   - Memory: 2048 MB
   - CPU shares: 512
2. Click "New Session" or use the sandbox start action in the UI

**Method B — Via Docker CLI (for testing):**
```powershell
docker run -d --name aegis-sandbox `
  --network none `
  --memory 2g `
  --cpus 0.5 `
  --restart no `
  --label "shiroscout.managed=true" `
  aegis-sandbox:latest
```

### Step 4.2: Check container status
```powershell
docker ps -a --filter "name=aegis-sandbox" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
```

Expected status: `Up X seconds` or `Up X minutes`

### Step 4.3: Check bridge health (if container is running)
```powershell
docker exec aegis-sandbox sh -c "wget -qO- http://localhost:8080/health 2>/dev/null || curl -s http://localhost:8080/health"
```

Expected response:
```json
{"status":"ok","version":"0.1.0","agents_running":0}
```

### Step 4.4: View container logs (for debugging)
```powershell
docker logs aegis-sandbox
```

---

## Phase 5 — Connect Tauri App to Sandbox

### Step 5.1: Verify Rust backend is ready

`container.rs` registers these Tauri commands:
| Command | Function |
|---------|----------|
| `build_sandbox_image` | Build the Docker image from Dockerfile.sandbox |
| `create_sandbox` | Create a container from the image |
| `start_sandbox` | Start the container by ID |
| `stop_sandbox` | Stop the container by ID |
| `get_sandbox_status` | Check container status |
| `pull_image` | Pull an image from registry |
| `check_docker_daemon` | Check Docker daemon availability |
| `get_docker_info` | Get Docker version + status |

### Step 5.2: Frontend status check

The sandbox status pill in the Navbar now:
- Shows 🟢 green when Docker is available
- Shows 🟣 purple pulse when checking
- Shows 🔴 red when Docker is unavailable
- Polls every 30 seconds via `refreshDockerStatus()`
- Shows real container status, not hardcoded "healthy"

### Step 5.3: Full end-to-end test

1. Start the Tauri app (development mode):
   ```powershell
   Set-Location "C:\Users\wayne\agent-zero\Shiro-Scout\usr\projects\shiro_scout"
   pnpm tauri dev
   ```

2. Verify the Navbar shows 🟢 green sandbox pill
3. Click the sandbox pill — should show tooltip with container name and version
4. Create a new session → should trigger `create_sandbox()` → container starts
5. The pill should update in real time

---

## Troubleshooting

| Problem | Likely Cause | Fix |
|---------|-------------|-----|
| Container exits immediately | Bridge binary not found or wrong arch | Check `docker logs aegis-sandbox`. Rebuild bridge for correct target. |
| Container exits with code 137 | Out of memory (OOM) | Increase memory limit: `--memory 4g` |
| Bridge not responding on :8080 | Entrypoint script failed | `docker exec aegis-sandbox sh -c "ls -la /opt/shiroscout/"` |
| Image build slow | apt packages + npm+playwright install | Expected on first build. Use `--no-cache` only when needed. |
| Docker not available in app | Docker daemon not running | Start Docker Desktop on Windows. Verify with `docker ps`. |
| Two bridges found | `bridge-build/` was a corrupted rebuild | Delete `src-tauri/docker/bridge-build/` — it's a duplicate. |

---

## Architecture Summary

```
┌─────────────────────┐
│  Tauri App (React)   │
│  ┌───────────────┐  │
│  │ Navbar Pill    │──┼── polls every 30s
│  │ (status LED)   │  │
│  └───────────────┘  │
│         │           │
│    invoke()         │
│    (Tauri IPC)      │
└─────────┬───────────┘
          │
┌─────────▼───────────┐
│  Rust Backend        │
│  ┌───────────────┐  │
│  │ container.rs   │──┼── bollard Docker API
│  │ docker_client  │  │
│  └───────────────┘  │
└─────────┬───────────┘
          │
┌─────────▼───────────┐
│  Docker Sandbox      │
│  ┌───────────────┐  │
│  │ agent-bridge   │──┼── HTTP :8080
│  │ (axum server)  │  │
│  │ Xvfb + Xfce    │  │
│  │ Playwright     │  │
│  │ Python + Node  │  │
│  └───────────────┘  │
│  Network: none       │
│  Memory: 2048 MB     │
└─────────────────────┘
```

---

## Checklist

- [ ] Phase 1: Cleanup — remove stale containers, prune images
- [ ] Phase 2: Bridge — rebuild `agent-bridge` binary, delete `bridge-build/`
- [ ] Phase 3: Image — rebuild `aegis-sandbox:latest` with fresh binary
- [ ] Phase 4: Launch — start container, verify health endpoint
- [ ] Phase 5: Test — start Tauri app, verify sandbox pill status
- [ ] Final: End-to-end test — create session, agent runs in sandbox
