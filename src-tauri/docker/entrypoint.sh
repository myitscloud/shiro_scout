#!/bin/bash
# ==============================================================================
# ShiroScout Sandbox Entrypoint
# ==============================================================================
#
# Launched when the Docker sandbox container starts. Performs initialization
# and then starts the HTTP bridge (agent-bridge) on port 8080.
#
# Bridge binary path passed as $1 or defaults to /opt/shiroscout/agent-bridge
# ==============================================================================

set -euo pipefail

BRIDGE_BINARY="${1:-/opt/shiroscout/agent-bridge}"
WORKSPACE_DIR="/workspace"

# ---------------------------------------------------------------------------
# 1. Ensure workspace directory exists with correct ownership
# ---------------------------------------------------------------------------
if [ ! -d "$WORKSPACE_DIR" ]; then
    mkdir -p "$WORKSPACE_DIR"
fi

# ---------------------------------------------------------------------------
# 2. Write AgentKit entry script
# ---------------------------------------------------------------------------
AGENTKIT_SCRIPT="/opt/shiroscout/run-agent.ts"
if [ ! -f "$AGENTKIT_SCRIPT" ]; then
    cat > "$AGENTKIT_SCRIPT" << 'SCRIPT'
import { AgentKit } from '@coinbase/agentkit';

// AgentKit runtime entry point for ShiroScout
// Called by the HTTP bridge via ts-node when agents need to run.

async function main() {
    const agentId = process.env.AGENT_ID || 'unknown';
    const prompt = process.argv[2] || '';

    const agentKit = await AgentKit.configure({
        apiKey: process.env.LLM_API_KEY,
        // LLM calls proxied via bridge to Tauri host
    });

    const result = await agentKit.run(prompt);
    console.log(result);
}

main().catch(console.error);
SCRIPT
    chmod 644 "$AGENTKIT_SCRIPT"
fi

# ---------------------------------------------------------------------------
# 3. Start virtual framebuffer (Xvfb) for headless browser tasks
# ---------------------------------------------------------------------------
if command -v Xvfb &> /dev/null; then
    export DISPLAY=":99"
    Xvfb "$DISPLAY" -screen 0 1920x1080x24 -ac &
    XVFB_PID=$!
    echo "[entrypoint] Xvfb started on display :99 (PID: $XVFB_PID)"
    sleep 1  # Give Xvfb time to start
fi

# ---------------------------------------------------------------------------
# 4. Start Xfce session on the virtual framebuffer (if available)
# ---------------------------------------------------------------------------
if command -v xfce4-session &> /dev/null; then
    export DISPLAY=":99"
    # Start Xfce silently - it will connect to the existing Xvfb display
    xfce4-session &
    XFCE_PID=$!
    echo "[entrypoint] Xfce session started (PID: $XFCE_PID)"
fi

# ---------------------------------------------------------------------------
# 5. Start VNC server for remote desktop access (optional)
# ---------------------------------------------------------------------------
if command -v x11vnc &> /dev/null; then
    export DISPLAY=":99"
    x11vnc -display :99 -forever -nopw -quiet &
    VNC_PID=$!
    echo "[entrypoint] x11vnc started on display :99 (PID: $VNC_PID)"
fi

# ---------------------------------------------------------------------------
# 6. Start the HTTP bridge
# ---------------------------------------------------------------------------
if [ -x "$BRIDGE_BINARY" ]; then
    echo "[entrypoint] Starting bridge: $BRIDGE_BINARY"
    export DISPLAY=":99"
    exec "$BRIDGE_BINARY"
else
    echo "[entrypoint] Bridge binary not found at $BRIDGE_BINARY"
    echo "[entrypoint] Waiting in fallback mode - bridge will be mounted at runtime"
    # Keep container alive for debugging / volume-mount scenarios
    tail -f /dev/null
fi
