#!/usr/bin/env bash
# ==============================================================================
# ShiroScout Sandbox Entrypoint  (rev 2 — exec-only architecture)
# ==============================================================================
#
# The host (Tauri/bollard) drives this container entirely through the Docker
# exec API. This script only prepares the display stack and then idles as a
# long-running PID 1 child. There is no HTTP bridge and no agent runtime
# started here — agents live host-side per ADR-003.
#
# Optional env:
#   ENABLE_VNC=1   start x11vnc on the Xvfb display (loopback only)
#
# Line endings: LF. A CRLF ending on the shebang line breaks exec entirely.
# ==============================================================================

set -euo pipefail

# ---------------------------------------------------------------------------
# 1. Virtual framebuffer for headed browser runs / screenshots
#    (Playwright headless does not require this, but headed debugging does.)
# ---------------------------------------------------------------------------
if command -v Xvfb >/dev/null 2>&1; then
    export DISPLAY=":99"
    Xvfb "${DISPLAY}" -screen 0 1920x1080x24 -ac &
    sleep 1
    echo "[entrypoint] Xvfb running on ${DISPLAY}"

    # -----------------------------------------------------------------------
    # 2. Optional VNC observation of the framebuffer (off by default).
    #    -localhost is defense-in-depth; with network_mode: none there is
    #    no reachable interface anyway.
    # -----------------------------------------------------------------------
    if [ "${ENABLE_VNC:-0}" = "1" ] && command -v x11vnc >/dev/null 2>&1; then
        x11vnc -display "${DISPLAY}" -forever -nopw -quiet -localhost &
        echo "[entrypoint] x11vnc started (loopback only)"
    fi
fi

# ---------------------------------------------------------------------------
# 3. Land in the workspace if it is mounted
# ---------------------------------------------------------------------------
cd /workspace 2>/dev/null || echo "[entrypoint] /workspace unavailable; staying in /"

# ---------------------------------------------------------------------------
# 4. Idle. The container's job is to exist; the host execs into it.
#    (HostConfig init:true provides tini as PID 1 above us.)
# ---------------------------------------------------------------------------
echo "[entrypoint] Sandbox ready (exec-only mode)."
exec sleep infinity
