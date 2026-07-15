# ShiroScout Threat Model

**Document:** `THREAT_MODEL.md`
**Owner:** Security Engineer (blocking authority)
**Last Updated:** 2026-07-15
**Framework:** STRIDE (Spoofing, Tampering, Repudiation, Information Disclosure, Denial of Service, Elevation of Privilege)

---

## 1. Overview and Scope

### System Description
ShiroScout is a security-first Tauri 2 / React 18 desktop application that orchestrates AI coding agents inside hardened Docker sandboxes. The application targets Windows 11 and manages AI agent execution, LLM interactions via proxied API calls, MCP server tools, and workspace file access — all within a defense-in-depth architecture.

### Trust Model
| Entity | Trust Level | Rationale |
|--------|------------|-----------|
| Human user/operator | **Trusted** | Root of all trust decisions; HITL approval bypasses only by human owner |
| Tauri Rust backend (host) | **Trusted** | Privileged orchestrator; compiled native binary with LTO; manages Docker and credentials |
| WebView (React UI) | **Minimally trusted** | Executes user-entered code and chat; can be compromised via XSS or malicious input |
| Docker sandbox (AgentKit) | **Untrusted** | Runs AI agent code; capabilities hardened; zero network; no direct secrets access |
| MCP servers (inside sandbox) | **Per-connection untrusted** | Allowlist-filtered; user approval per connection; cannot persist between sessions |
| Remote LLM providers | **Semi-trusted** | Process prompts; do not store API keys; HTTPS only; user chooses local vs cloud |
| Host OS & Docker daemon | **Root of trust** | Underlying platform security assumed intact |

### Scope Inclusions
- All four security boundaries: Container, IPC (internal), Host, UI (WebView + Tauri IPC)
- External interfaces: LLM provider HTTPS, Docker image registry pulls, MCP server connections
- Assets: API keys, session data, workspace files, agent configurations, MCP tool registrations

### Scope Exclusions
- Physical attacks on host hardware
- OS-level exploits (kernel vulnerabilities, rootkits)
- Social engineering of the human user
- Third-party LLM provider data handling policies

---

## 2. Architecture Diagram Reference (Text-Based)

```
┌──────────────────────────────────────────────────────────────────────────┐
│                        HOST OS (Windows 11)                             │
│                                                                          │
│  ┌─────────────────────────────┐        ┌────────────────────────────┐  │
│  │      WebView (React 18)     │        │   Rust Backend (Tauri 2)   │  │
│  │  - Chat UI / Terminal       │◄─────►│  - State machine            │  │
│  │  - Settings & HITL dialogs  │  IPC   │  - API key mgmt (keychain)  │  │
│  │  - `tauri-commands.ts`      │        │  - bollard Docker API       │  │
│  └──────────┬──────────────────┘        └────────┬───────────────────┘  │
│             │  Trust Boundary AI                  │                     │
│             │  (Tauri IPC validates on receive)   │                     │
│             │                                     │ Docker socket        │
│             │                                     ▼                     │
│             │                          ┌──────────────────────┐        │
│             │                          │   Docker Daemon       │        │
│             │                          │  (Host-level access)  │        │
│             │                          └──────────┬───────────┘        │
│             │                                     │                     │
│  ┌──────────┴─────────────────────────────────────┴──────────────────┐ │
│  │              DOCKER SANDBOX (Container)                           │ │
│  │                                                                   │ │
│  │  ┌───────────────────────────┐    ┌────────────────────────────┐  │ │
│  │  │  AgentKit (AI Worker)     │    │   MCP Servers (allowlist)  │  │ │
│  │  │  - Tool execution         │◄──►│   - Tool registration      │  │ │
│  │  │  - LLM proxied calls      │    │   - Per-connection auth    │  │ │
│  │  │  - workspace /tmp /run    │    └────────────────────────────┘  │ │
│  │  └───────────┬───────────────┘                                     │ │
│  │              │                                                     │ │
│  │              ▼                                                     │ │
│  │  ┌──────────────────────────────────────────────┐                  │ │
│  │  │  HTTP Bridge (axum, localhost:8080)           │                  │ │
│  │  │  - Proxies LLM calls to Tauri host            │                  │ │
│  │  │  - Forwards tool exec requests                │                  │ │
│  │  │  - Health check endpoint                      │                  │ │
│  │  └──────────────┬───────────────────────────────┘                  │ │
│  │                 │ Unix domain socket (inside container)            │ │
│  └─────────────────┼──────────────────────────────────────────────────┘ │
│                    │                                                    │
│                    ▼                                                    │
│         ┌─────────────────────┐                                        │
│         │  Remote LLM APIs    │                                        │
│         │  (HTTPS, proxied)   │                                        │
│         └─────────────────────┘                                        │
└──────────────────────────────────────────────────────────────────────────┘

### Security Hardening Summary (Container)

| Control | Value | Purpose |
|---------|-------|---------|
| Read-only rootfs | `true` | Prevent persistent tampering of system files |
| Network mode | `none` | No outbound; all LLM calls proxied through host |
| Init process | `true` | Zombie reaper for AgentKit child processes |
| Capabilities | `cap_drop: ALL` | Minimum privilege inside container |
| no-new-privileges | `true` | Block privilege escalation |
| User | `1000:1000` (non-root) | Reduce container escape impact |
| tmpfs /tmp | `noexec,nosuid,size=256M` | Prevent code execution from temp |
| tmpfs /run | `noexec,nosuid,size=64M` | Prevent code execution from run |
| Memory limit | 2 GB | Prevent memory exhaustion |
| CPU limit | 0.5 cores | Prevent CPU starvation |
| PIDs limit | 256 | Fork-bomb protection |
| Health check | HTTP localhost:8080 | Sandbox readiness verification |

---

## 3. STRIDE Threat Table per Security Boundary

### 3.1 Container Boundary (Docker Sandbox Internal)

| STRIDE Category | Threat Description | Affected Component | Risk | Existing Mitigations | Recommended Additional Controls |
|-----------------|-------------------|-------------------|------|---------------------|-------------------------------|
| **Spoofing** | An agent inside the sandbox impersonates another agent or MCP server to gain elevated tool access | AgentKit process pool, MCP server registry | **Medium** | Single-container design limits cross-agent interaction; process isolation via `child_process.fork()`; MCP tools allowlist-filtered | Add AgentKit process authentication tokens for tool delegation; audit log all cross-agent requests |
| **Tampering** | Malicious agent code modifies workspace files of another agent or tampers with shared `/tmp` | Workspace bind mount, shared tmpfs | **High** | Workspace is bind-mounted per-session; tmpfs `noexec,nosuid`; read-only rootfs prevents system tampering | Implement per-agent workspace subdirectories with strict permissions; add filesystem integrity monitoring for workspace files; consider session-scoped temp directories |
| **Tampering** | Agent modifies `/etc/hosts`, DNS resolver, or container networking despite `network_mode: none` | Container OS network stack | **Low** | `network_mode: none` completely removes network interfaces; read-only rootfs prevents DNS config modification | Verify in CI that container has zero network interfaces after start; periodic runtime check via `ip link` inside container |
| **Repudiation** | An agent performs a destructive action without traceable attribution | Tool execution pipeline, HITL logs | **Medium** | HITL approval required for destructive actions (no 'approve all' toggle); health check endpoint logged | Implement structured audit logging for all agent tool calls with agent ID timestamp; export logs to host via tmpfs mount; add cryptographic signing of audit entries |
| **Information Disclosure** | Agent accesses LLM API session tokens or proxy credentials passed inadvertently via environment variables | Environment variables passed to container | **Medium** | API keys never enter container (proxied via Tauri host); no secrets in env vars per design | Audit all env vars passed to container in development; add runtime env var scanner that rejects known secret patterns; use a dedicated proxy credential mechanism rather than env vars |
| **Information Disclosure** | Agent exfiltrates workspace data via encoded output, timing channels, or covert IPC | Workspace files, proxy output | **Medium** | `network_mode: none` blocks network exfiltration; all output goes through Tauri IPC which reviews (HITL) | Add output size limits per response; implement anomaly detection for high-entropy output; rate-limit response tokens |
| **Denial of Service** | Agent runs a fork bomb exhausting `pids_limit` | Process spawn in AgentKit | **Low** | `pids_limit: 256` caps processes; `init: true` reaps zombies; `mem_limit: 2g` and `cpus: 0.5` limit resources | Add timeout per agent tool execution; implement agent-level resource quotas in addition to container-level limits |
| **Denial of Service** | Agent consumes all available tmpfs space (/tmp 256 MB, /run 64 MB) | tmpfs mounts | **Low** | Size limits on tmpfs prevent unbounded growth; read-only rootfs limits writable locations | Add per-agent temp space quota via disk quotas (if feasible) or prompt injection guardrails against filling disk |
| **Elevation of Privilege** | Agent exploits kernel/container runtime bug to escape to host | Docker runtime, kernel | **High** | `cap_drop: ALL`, `no-new-privileges: true`, non-root user `1000:1000`, read-only rootfs, network isolation — defense in depth against known escape vectors | Keep container image and host kernel updated; add AppArmor/SELinux profile; run `docker run --security-opt seccomp=seccomp-profile.json` with a custom seccomp profile blocking additional syscalls; consider `--security-opt apparmor:shiroscout` for unconfined-host Linux |
| **Elevation of Privilege** | Agent escapes via Docker API socket if exposed inside container | Docker socket mount | **High** | Docker socket is NOT mounted inside the container (Rust backend uses bollard externally) | Audit that no build artifact exposes Docker socket; add CI test enforcing socket absence |

### 3.2 IPC Boundary (HTTP Bridge + Unix Domain Socket)

| STRIDE Category | Threat Description | Affected Component | Risk | Existing Mitigations | Recommended Additional Controls |
|-----------------|-------------------|-------------------|------|---------------------|-------------------------------|
| **Spoofing** | A compromised agent or another container process sends forged HTTP requests to the bridge on behalf of a legitimate agent | HTTP bridge (axum on localhost:8080) | **Medium** | Bridge listens on localhost only (no host port exposure); single container architecture limits exposure | Add authentication token between AgentKit and bridge (e.g., shared secret via env var not settable by agent); implement TLS between bridge and Tauri host |
| **Tampering** | An attacker intercepts or modifies IPC messages between bridge and Tauri host | Unix domain socket | **Low** | Unix socket permissions are managed by the container runtime; socket is inside container only | Set strict socket permissions (0600) and ownership to uid 1000; verify socket path not world-writable; implement message integrity checks |
| **Repudiation** | Tool execution requests forwarded through the bridge lack audit trail | Bridge request logs | **Medium** | ADR-003 requires logging of all proxied requests | Implement structured audit logging on the bridge with request IDs, timestamps, agent ID, and tool name; flush logs to host-readable location |
| **Information Disclosure** | Bridge logs contain sensitive tool arguments (file paths, LLM prompts) | Bridge request/response logs | **Medium** | No log redaction policy explicitly documented for bridge | Implement log redaction in the bridge: strip or hash file paths, redact prompts containing potential secrets; classify bridge logs as operational data (S6 classification) |
| **Denial of Service** | Agent floods the bridge with requests, starving other IPC | HTTP bridge server | **Low** | Single-agent per session (singleton); resource limits at container level apply | Add request rate-limiting and queue depth limits in the bridge axum handler; implement backpressure to the agent |
| **Elevation of Privilege** | Agent sends a crafted HTTP request that triggers an internal Tauri command with elevated privileges | Bridge → Tauri IPC forwarding | **Medium** | All tool execution paths must pass through a Security Engineer-reviewed allowlist (ADR-003); no generic command passthrough | Implement per-request privilege check: verify that the requested tool is allowed for the current agent; add parameter validation on the Tauri host side before execution |

### 3.3 Host Boundary (Tauri Backend + Docker Daemon)

| STRIDE Category | Threat Description | Affected Component | Risk | Existing Mitigations | Recommended Additional Controls |
|-----------------|-------------------|-------------------|------|---------------------|-------------------------------|
| **Spoofing** | Malicious WebView JS impersonates a legitimate IPC command to trigger privileged operations | Tauri IPC bridge (invoke handlers) | **High** | Tauri validates IPC message types; Rust backend validates inputs (C6: every path from frontend input validated on receive side) | Implement per-command authorization checks (capability verification before execution); audit that all IPC handlers validate caller origin |
| **Tampering** | WebView compromised via XSS modifies Tauri state or triggers unauthorized Docker operations | Tauri IPC handlers for bollard operations | **High** | No generic command passthrough; all Docker operations wrapped in typed Rust functions; C6 validation on all frontend-controlled inputs | Add parameter validation for all Docker command parameters (container ID format, image name sanitization, workspace path canonicalization); implement command allowlist per IPC endpoint |
| **Repudiation** | User denies performing a destructive action that was approved via HITL | HITL approval logs | **Low** | HITL dialog logs approval/denial events with timestamp; 60s auto-deny fallback | Add user identity attestation (Windows Hello or simple password confirmation) for destructive operations; store HITL logs as append-only audit |
| **Information Disclosure** | API key leaked via crash dump, swap file, or debugger attachment | Rust backend memory (keychain handles) | **High** | API keys stored in OS keychain (Windows Credential Manager); in-memory via `SecureString` equivalents; Tauri compiled with LTO and stripped | Enable memory encryption for sensitive data in Rust (e.g., `secrecy` crate); register for crash dump exclusion of keychain regions; verify debugger protection on release builds |
| **Information Disclosure** | Docker API socket accessible to non-privileged processes on host | Docker daemon Unix socket (Linux) / named pipe (Windows) | **High** | Application runs as user-level process; Docker socket permissions default to root-only | Document that Docker Desktop or Docker Engine must be configured to require admin privileges; add runtime check that Docker socket permissions are restrictive |
| **Denial of Service** | User starts too many containers, exhausting host resources | bollard container management | **Low** | Single-container design (one sandbox at a time); health check monitors container state | Add backend-enforced singleton: prevent starting a second container while one is active; implement container timeout after inactivity |
| **Elevation of Privilege** | Vulnerability in bollard library allows arbitrary Docker API calls | bollard dependency | **Medium** | bollard is a maintained async Rust crate; C12 dependency review performed on addition; `cargo audit` runs on every change set | Add Docker API operation whitelist in backend: only allowed operations (create, start, stop, exec, inspect, logs) are exposed; add logging of all bollard invocations |
| **Elevation of Privilege** | Workspace path traversal via symlink/junction attacks | Workspace bind mount | **Medium** | Workspace is bind-mounted; agent runs as non-root inside container | Canonicalize workspace path before bind mount (resolve symlinks); add C9-style check for symlinks/junctions inside host workspace that point outside permitted root |

### 3.4 UI / WebView Boundary (User-Facing)

| STRIDE Category | Threat Description | Affected Component | Risk | Existing Mitigations | Recommended Additional Controls |
|-----------------|-------------------|-------------------|------|---------------------|-------------------------------|
| **Spoofing** | Malicious site in embedded terminal (xterm.js) steals input focus and captures keystrokes | Terminal component, WebView interaction | **Medium** | Terminal is in a separate bottom drawer; IPC isolation limits terminal to Docker exec stream | Sanitize terminal output to prevent escape sequences that redirect input; restrict terminal to read-only mode for untrusted output |
| **Tampering** | User enters a malicious command in chat that tricks the agent into performing a destructive action | Chat input, HITL system | **High** | HITL approval required for destructive actions (default-deny); 60s timeout auto-denies; emergency kill button | Add prompt injection detection for user messages that try to override agent constraints; classify user intent before agent execution (e.g., 'informational' vs 'operational') |
| **Repudiation** | User claims they did not approve a destructive action despite HITL dialog | HITL approval dialog logs | **Low** | HITL events logged with timestamp; no fast-approve path | Screen-record HITL approval events (pseudonymous); require explicit checkbox 'I understand this action' for High risk operations before approve button activates |
| **Information Disclosure** | Agent chat history or settings exposed via browser cache or WebView storage | Local storage, session state | **Medium** | Session state stored in Rust backend (auto-saved); WebView memory is ephemeral per app lifecycle | Disable local storage in WebView configuration; use in-memory backstore only; implement session clearing on app close |
| **Denial of Service** | Malicious agent sends large streaming response to overwhelm UI rendering | MessageThread, StreamingText components | **Low** | Virtualization (CSS Grid, efficient DOM updates); streaming cursor renders incrementally | Add token cap per response; implement incremental virtual rendering for large outputs; stop streaming if agent produces more than X tokens per second |
| **Elevation of Privilege** | Compromised WebView script calls Tauri IPC commands directly without user interaction | `tauri-commands.ts`, IPC handlers | **High** | Tauri's isolation pattern prevents direct command calls from WebView without explicit IPC; HITL for destructive operations | Add Tauri capabilities file with scoped permissions per command; implement nonce-based command authorization; audit all IPC commands that can be called from WebView |

### 3.5 AgentKit Runtime Boundary (Wave 4: State Machine, Tool Exec Bridge, PTY, Persistence, MCP Discovery)

**Components:** Agent state machine (`src-tauri/src/agent/`), bollard-based Docker exec through tool exec bridge (`bridge_client.rs`, `container.rs`), PTY sessions (`pty/mod.rs`), state persistence (`agent/persistence.rs`), MCP discovery (`mcp/mod.rs`)

> ⚠️ **Wave 4 Verification Finding (2026-07-10):** All 12 security controls planned for this boundary (SC-41 through SC-52) are either ❌ Not implemented or 🟡 Partially implemented in shipped code. The Existing Mitigations column describes only deployment-layer controls (single-agent-per-container, localhost-only bridge, Rust type safety). The Recommended Additional Controls column lists controls that were planned but are NOT VERIFIED in shipped code. **Residual risk for threats in this section is higher than originally assessed.** See §5 Security Controls Inventory for per-control code evidence.

| STRIDE Category | Threat Description | Affected Component | Risk | Existing Mitigations | Recommended Additional Controls |
|-----------------|-------------------|-------------------|------|---------------------|-------------------------------|
| **Spoofing** | Agent state ID spoofing: an internal process impersonates the legitimate agent by sending forged state transition requests | Agent state machine (`agent/state.rs`) | **Medium** | Single-agent-per-container; state transitions handled by trusted Rust backend | Add session token bound to agent PID; verify token on every state transition; log state ID mismatches |
| **Spoofing** | Process inside container sends forged tool execution requests to the bridge impersonating the agent | Tool exec bridge (`bridge_client.rs`, `container.rs`) | **Medium** | Bridge listens on localhost only; single-container design | Require bearer token from agent process for tool exec requests; implement process-origin verification |
| **Spoofing** | Malicious MCP server registers fake tools via MCP discovery, impersonating legitimate tool providers | MCP discovery (`mcp/mod.rs`) | **Medium** | MCP allowlist-filtered registration; user approval per connection | Add tool registration signature verification; validate tool name and origin against allowlist; require user approval per discovery event |
| **Tampering** | Attacker forces illegal state machine transition (e.g., bypasses PendingConfirmation to jump directly to Running) | Agent state machine (`agent/state.rs`, `agent/agent.rs`) | **High** | Rust type system enforces state enum transitions; HITL flow documented in Wave 7.1 | Add transition validation in state machine: reject transitions not in allowed adjacency matrix; log all transition attempts (success/failure) |
| **Tampering** | Agent tampers with tool execution arguments between validation and bridge dispatch | Tool exec bridge | **Medium** | Arguments validated on Tauri host receive side (C6); types enforce structure | Add argument hash verification: Rust backend computes hash of validated args, bridge verifies before execution |
| **Tampering** | Attacker injects commands into an active PTY session | PTY sessions (`pty/mod.rs`) | **High** | PTY input is validated by Tauri backend; no direct container access from UI to PTY | Implement command allowlist for PTY input; strip shell escape sequences at boundary; log all PTY input and output |
| **Tampering** | State persistence file modified on disk to alter agent behavior or inject malicious configuration | State persistence (`agent/persistence.rs`) | **High** | Persistence writes to host filesystem; state is serialized Rust structs via serde | Implement integrity check (HMAC) on state files; validate checksum before deserialization; reject state files with mismatched signatures |
| **Tampering** | MCP discovery manifest modified by malicious process to register backdoored tools | MCP discovery | **High** | MCP servers inside container; sandbox isolation | Sign MCP discovery manifests; verify manifest integrity before tool registration; implement runtime manifest hash monitoring |
| **Repudiation** | Agent state transitions not logged with sufficient detail to trace unauthorized actions | Agent state machine | **Medium** | Basic state logging may exist in agent_loop | Add structured audit logging for all state transitions with timestamp, old_state, new_state, trigger event, agent_id; export logs to host (append-only) |
| **Repudiation** | Tool execution via bridge lacks call-chain attribution linking to originating agent prompt | Tool exec bridge | **Medium** | ADR-003 requires logging | Add call-chain IDs (trace_id) to every tool exec that links back to the agent message that triggered it |
| **Repudiation** | PTY command execution not logged with agent session attribution | PTY sessions | **Medium** | Container-level process logging only | Log all PTY command submissions with agent_id, timestamp, truncated command; implement session-scoped PTY history |
| **Repudiation** | State persistence operations (save/load) not attributed to specific agent sessions | State persistence | **Low** | File system timestamps provide basic chronology | Add operation log with agent_id, operation (save/load/delete), file hash, timestamp; store adjacent to state file |
| **Repudiation** | MCP server registration cleared on disconnect, leaving no discovery audit trail | MCP discovery | **Low** | Per-connection ephemeral design | Log all MCP discovery events (connect, register, disconnect); store audit log to host, not container |
| **Information Disclosure** | State machine checkpoint files contain sensitive agent prompts, tool results, or LLM conversation history | State persistence | **Medium** | State files stored on host filesystem accessible to host user only | Encrypt state files at rest (AEAD); implement session-level encryption key derived from user PIN; scope state file permissions to current user only |
| **Information Disclosure** | PTY session output leaks file paths, environment variables, or secrets from the container | PTY sessions | **Low** | Container runs non-root; secrets never enter container (proxied) | Implement PTY output redaction for known secret patterns; limit PTY scrollback buffer size; classify PTY output as sensitive |
| **Information Disclosure** | MCP discovery response reveals available tools, server capabilities, and internal configuration | MCP discovery | **Low** | MCP servers inside sandbox; no network access | Add tool metadata classification (public/internal); require user approval before exposing tool details to untrusted components |
| **Denial of Service** | Agent state machine enters infinite loop (Running to Error to Running) exhausting CPU | Agent state machine | **Low** | Self-healing loop with max retries; container resource limits | Add loop detection: if same state transition repeats >N times within a window, force state to Error; implement backoff |
| **Denial of Service** | Flooding the tool exec bridge with rapid exec requests exhausts Docker exec resources (container process slots) | Tool exec bridge, bollard | **Low** | Container-level resource limits (PIDs 256, memory 2GB) | Add rate limiter to tool exec bridge: max N concurrent execs; queue depth limits; return 429 on overload |
| **Denial of Service** | PTY session resource exhaustion: opening unlimited PTY sessions | PTY sessions | **Low** | Container PID limit (256) caps forkable processes | Enforce max 1 PTY session per agent; add session reuse/reconnect instead of create-new |
| **Denial of Service** | Malformed state file causes deserialization panic or infinite allocation | State persistence | **Medium** | serde deserialization is type-safe; Rust panics on unexpected data | Add size limit on state file before deserialization; use serde `deny_unknown_fields`; implement graceful error handling with fallback to default state |
| **Denial of Service** | MCP discovery flood: rapid connect/disconnect cycles exhaust registration resources | MCP discovery | **Low** | Per-connection ephemeral; single MCP session per agent | Add MCP connection rate limit; enforce minimum session lifetime; detect rapid connect/disconnect cycles |
| **Elevation of Privilege** | Agent bypasses state machine guard by sending raw transition events that skip security checks | Agent state machine | **High** | State transitions controlled by Rust backend; direct internal events not exposed to agent | Add authorization check per state transition type; explicitly model allowed transitions in adjacency matrix; deny any transition not in matrix |
| **Elevation of Privilege** | Agent executes a tool via bridge that escalates container privileges (e.g., cgroup modification, device access) | Tool exec bridge, Docker exec | **High** | Container hardened with `cap_drop: ALL`, `no-new-privileges: true`; tool allowlist enforced | Add tool risk classification; enforce allowlist per security level; reject tools that attempt privileged operations even within container |
| **Elevation of Privilege** | Attacker uses PTY to execute commands that leverage container misconfiguration for privilege escalation | PTY sessions | **Medium** | Non-root user (1000:1000); no-new-privileges; read-only rootfs | Add command allowlist for PTY; intercept dangerous command patterns (sudo, chroot, unshare, nsenter); terminate PTY session on escalation attempt |
| **Elevation of Privilege** | Crafted state file triggers deserialization gadget chain to achieve code execution | State persistence | **Medium** | Rust serde is not vulnerable to Java-style deserialization attacks; type-enforced | Implement `#[serde(deny_unknown_fields)]`; add strict validation of deserialized state values (path bounds, agent ID format); verify state file origin before loading |
| **Elevation of Privilege** | MCP discovery loads a tool definition with elevated capabilities that bypasses agent tool guardrails | MCP discovery | **High** | MCP tool execution goes through same HITL/allowlist path as built-in tools (Wave 7.1) | Add capability classification to MCP tool manifests; require Security Engineer approval for tools with FileWrite, CommandExec, or Network capabilities; implement capability intersection at registration: tool effective caps = user-approved caps ∩ runtime allowed caps |
| **Information Disclosure** | API key stored as plaintext String in AppSettings struct, persisted to unencrypted settings.json with no atomic write or encryption | settings.rs, settings.json | **Critical** | API key is eventually stored in OS keychain via settings::save_api_key command, but AppSettings.default().api_key starts as plaintext String in the Tauri managed state | Remove plaintext api_key field from AppSettings; always route through Keychain; add secure memory zeroization on drop; atomic write + integrity check for settings.json |
| **Information Disclosure** | MCP discovery TCP connect scan (3100-3200, 101 ports) auto-discovers ANY TCP listener on localhost, including malicious or unexpected services | mcp/mod.rs discover() → probe_port() | **High** | Probe only runs inside container (network_mode: none); results returned to host for user approval per ADR-006 | Add authenticated service discovery: require MCP servers to present identity token on /health probe; add port range configurable in settings with minimum (e.g., 1 port); validate health probe response includes MCP protocol signature |
| **Elevation of Privilege** | Agent calls exec_terminal without bridge attached, falling back to host `sh -c` command execution, bypassing sandbox isolation | tools/mod.rs exec_terminal() line 91-105 | **High** | Bridge attachment enforced via ToolExecBridge; host fallback only occurs when bridge is None (development mode) | Remove host fallback entirely; require bridge for all tool execution; enforce bridge attachment check at ToolRegistry construction time rather than execution time |
| **Tampering** | State machine transition_to() performs blind replace with no adjacency validation — any state can transition to any other state | agent/context.rs transition_to() | **High** | Rust type system prevents invalid enum construction at compile time but does not enforce transition validity at runtime | Implement allowed transition adjacency matrix; transition_to() should verify the source → target transition is valid; log and deny invalid transitions |
| **Spoofing** | HITL nonce uses std::collections::hash_map::DefaultHasher (64-bit, not cryptographic) despite comment saying "blake3_hash" | hitl.rs blake3_hash() lines 72-84 | **Medium** | Nonce prevents basic replay attacks; nonce includes UUID + timestamp input | Replace DefaultHasher with a proper cryptographic hash (blake3, SHA-256 via ring crate); the current 64-bit hash is predictable to a local attacker and breaks the nonce guarantee for replay prevention |
| **Denial of Service** | PTY background reader tasks continue running after session close, leaking tokio task handles and buffer memory | pty/mod.rs close_session() — background reader tokio::spawn not terminated | **Medium** | Close_session removes session from HashMap; background reader terminates eventually when exec stream ends (broken pipe) | Add explicit cancellation: pass a CancellationToken to each background reader; cancel on session close; set bounded buffer size for PTY output |
| **Denial of Service** | Settings.json and agent_state.json files written without size limits, integrity checks, or deny_unknown_fields on deserialization structs | settings.rs, persistence.rs | **Medium** | Atomic rename pattern prevents partial-write corruption in persistence.rs; Rust type safety limits deserialization | Add #[serde(deny_unknown_fields)] to AppSettings, AgentPersistedState, AgentContext; enforce max file size (1MB) before deserialization; add signature verification


---

## 4. Attack Scenarios

### Scenario 1: Container Escape via Kernel Exploit
**Threat:** An AI agent crafts a payload that exploits a Linux kernel vulnerability (e.g., CVE-2022-0492 cgroup escape, CVE-2024-21626 runc) to break out of the Docker container to the host.

**Attack Path:**
1. Agent executes inside container as user 1000:1000
2. Agent uses a known kernel exploit or 0-day to escape cgroup/namespace isolation
3. Without `cap_drop: ALL` and `no-new-privileges`, the exploit could get root on host
4. Malicious agent gains persistent access to host filesystem and possibly other containers

**Likelihood:** Low (kernel exploits are rare; hardening measures heavily mitigate)
**Impact:** Critical (full host compromise)
**Existing Mitigations:** cap_drop ALL, no-new-privileges true, non-root user, read-only rootfs, AppArmor/SELinux available
**Residual Risk:** Medium — kernel 0-days bypass namespace isolation even with hardening. Need seccomp profile as additional layer.

### Scenario 2: Prompt Injection Leading to Destructive Action
**Threat:** A user or external source provides input that tricks the AI agent into executing a destructive command (rm -rf, format disk, exfiltrate data) that the user might accidentally approve.

**Attack Path:**
1. User pastes content from untrusted source (e.g., malicious code snippet from a forum)
2. Hidden prompt injection in code comment instructs agent to run `rm -rf /workspace/*`
3. Agent presents this action to user via HITL dialog with description 'Delete old workspace files'
4. User, misled, approves the destructive action

**Likelihood:** Medium (prompt injection is well-documented attack on AI agents)
**Impact:** High (irrevocable workspace loss, potential data destruction)
**Existing Mitigations:** HITL default-deny with 60s timeout; tool execution allowlist; Emergency kill button
**Residual Risk:** Medium — HITL dialog may show sanitized description; user may not read carefully. Agent may still describe misinformation in approval prompt.

### Scenario 3: API Key Extraction from Host Memory/Disk
**Threat:** An attacker with local access to the host (or via another compromised app) extracts LLM API keys from the Tauri backend's memory, crash dumps, or the OS keychain.

**Attack Path:**
1. Attacker gains local user-level access to host
2. Dumps process memory of Tauri backend via Task Manager / procdump
3. Scans memory for API key patterns (sk-*, bearer tokens)
4. Uses extracted keys to run unauthorized LLM queries

**Likelihood:** Low (requires local access; keychain provides some protection)
**Impact:** High (financial cost of stolen LLM API usage; potential data leak via query history)
**Existing Mitigations:** Keys stored in OS keychain (Windows Credential Manager); Tauri backend proxied, keys never reach UI
**Residual Risk:** Medium — in-memory secrets are vulnerable to memory dump attacks. Recommend `secrecy` crate for zeroization.

### Scenario 4: Malicious MCP Server Compromise
**Threat:** An MCP server (either installed by user or from a registry) contains malicious code that, when executed by the agent, accesses workspace files, tampers with other MCP tools, or disrupts agent operations.

**Attack Path:**
1. User approves an MCP server connection (Tool A)
2. MCP server registers a tool 'format-code.js' which contains a backdoor
3. Agent calls this tool with workspace file contents as arguments
4. MCP server exfiltrates data via side-channel (timing, encoded output) or modifies workspace files

**Likelihood:** Medium (depends on user's MCP server vetting)
**Impact:** Medium-High (workspace data exposure, output manipulation)
**Existing Mitigations:** MCP servers inside sandbox (container isolation); tool registry allowlist-filtered; user approval per connection; no persistent installs
**Residual Risk:** Medium — single container means all MCP servers share the same workspace and process space. MCP server cannot call network (network_mode: none) but can read all workspace files.

### Scenario 5: IPC Hijacking Inside Container
**Threat:** A compromised process inside the container (AgentKit child or an installed tool) sends forged HTTP requests to the local bridge on port 8080, impersonating the legitimate AI agent to execute unauthorized tools or access proxied LLM endpoints.

**Attack Path:**
1. AgentKit child process compromised via malicious code execution
2. Compromised process sends HTTP POST to http://localhost:8080 with fake tool request
3. Bridge forwards request to Tauri host for execution
4. Tool execution bypasses normal agent guardrails

**Likelihood:** Medium (AgentKit process isolation via fork(2) is weak; shared memory possible)
**Impact:** High (unauthorized tool execution, potential data access)
**Existing Mitigations:** Bridge listens on localhost only; all tool exec paths require Security Engineer allowlist review
**Residual Risk:** Medium-High — no authentication between agent and bridge; any process on localhost can call the bridge.

### Scenario 7: Air-Gapped Mode Switch Bypass (Wave 7.2)
**Threat:** An attacker (or malicious agent) switches the sandbox network mode from `none` (air-gapped) to `bridge` without HITL approval, allowing data exfiltration or inbound network attacks.

**Attack Path:**
1. Agent operates in air-gapped mode (network_mode: none), unable to exfiltrate
2. Attacker triggers set_sandbox_network_mode command with `new_mode: Bridge`
3. Without HITL enforcement, the container gains network access
4. Agent exfiltrates workspace data via outbound connections

**Likelihood:** Low (requires bypassing HITL or compromised state machine)
**Impact:** High (loss of air-gapped guarantee; data exfiltration)
**Existing Mitigations:** HITL confirmation required for `toggle_network_mode` (classified High risk); frontend must orchestrate request_hitl_confirmation → respond_hitl → set_sandbox_network_mode; mode configuration change requires container restart; default config is air-gapped (NetworkMode::None)
**Residual Risk:** Low — dual-call HITL pattern prevents single-command bypass; container restart requirement means mode change has immediate cost
### Scenario 8: State File Tampering Leading to Code Execution
**Threat:** An attacker with host file access modifies the agent state persistence file to inject malicious configuration or trigger a deserialization vulnerability, altering agent behavior or gaining code execution.

**Attack Path:**
1. Attacker gains local user-level access to host
2. Locates state file at `<workspace>/agent_state_*.json`
3. Modifies serialized state to inject malicious tool configurations or transition vectors
4. On next agent start, state is loaded with modified configuration
5. Attacker-controlled tool executes with agent privileges inside the sandbox

**Likelihood:** Low (requires local file access)
**Impact:** Medium-High (agent behavior modified; potential container reconfiguration)
**Existing Mitigations:** State files stored in user profile directory; no world-writable permissions; serde type-safe
**Residual Risk:** Medium — state file integrity not verified before load; no encryption at rest

### Scenario 9: PTY Session Hijacking and Command Injection
**Threat:** An attacker inside the container hijacks an active PTY session or injects commands into the PTY stream, executing arbitrary shell commands with the agent's identity.

**Attack Path:**
1. Agent has an active PTY session for shell operations
2. Malicious process (or compromised child) discovers the PTY master file descriptor or named pipe
3. Attacker writes malicious command bytes to the PTY input stream
4. Command executes in the PTY shell with same permissions as the agent

**Likelihood:** Medium (child process can inherit or probe PTY descriptors)
**Impact:** High (arbitrary shell execution with agent context)
**Existing Mitigations:** Agent runs as non-root (1000:1000); container capabilities dropped; no-new-privileges
**Residual Risk:** Medium — PTY descriptors may be inherited by child processes; no per-command input validation

### Scenario 10: MCP Discovery Tool Spoofing
**Threat:** A malicious MCP server registers a tool with a name identical to a legitimate tool (e.g., `read_file`) during MCP discovery, intercepting tool calls intended for the legitimate tool.

**Attack Path:**
1. Agent has an MCP server with a legitimate `read_file` tool
2. User approves a second MCP server connection
3. Second MCP server registers a tool also named `read_file` with malicious behavior
4. Agent calls `read_file`, routed to malicious server instead of legitimate one
5. Malicious server returns manipulated file content or exfiltrates data

**Likelihood:** Low-Medium (requires user to approve malicious MCP server; same-named tools must overlap)
**Impact:** Medium (tool output manipulation; potential data exfiltration via side-channel)
**Existing Mitigations:** MCP allowlist-filtered registration; user approval per connection; no persistent trust
**Residual Risk:** Medium — no tool name uniqueness enforcement across MCP servers; routing to first-registered or last-registered is ambiguous


### Scenario 6: Supply Chain Attack (Docker Image or Dependency)
**Threat:** An attacker compromises the ShiroScout Docker image or a critical dependency (npm package, Rust crate) to inject malicious code that runs inside the sandbox or on the host.

**Attack Path:**
1.a Attacker publishes a malicious npm package that gets pulled via lockfile update without C12 review
1.b Attacker compromises the Docker base image or registry to replace shiro-scout-sandbox:latest with a backdoored version
2. On next sandbox start, malicious code executes inside container or on host
3. Backdoor exfiltrates workspace data or provides persistent access

**Likelihood:** Low-Medium (requires compromised upstream or CI)
**Impact:** Critical (full workspace access; potential host compromise via container escape)
**Existing Mitigations:** C12 dependency review (blocking); `cargo audit`/`npm audit`/`cargo deny` per change set; lockfiles enforced; Docker image pinning to digest; read-only rootfs
**Residual Risk:** Medium — Docker image is pinning to digest not implemented yet; no image signature verification (cosign/notary).

### Scenario 11: API Key Persisted as Plaintext in settings.json
**Threat:** The LLM API key is stored as a plaintext String field in the `AppSettings` struct and persisted to `settings.json` at the app config directory, accessible to any local user or process with file system access.

**Attack Path:**
1. Attacker gains local user-level access to host
2. Attacker reads `<app_config_dir>/settings.json` which contains `"api_key":"sk-..."` as a plaintext field
3. API key is used for unauthorized LLM API calls at the user's expense
4. Additionally, the AppSettings struct is mutable Tauri State; a compromised WebView could read it via IPC

**Likelihood:** High (api_key field is defined as plaintext String in settings.rs AppSettings struct; file is unencrypted JSON)
**Impact:** Critical (API key compromise leads to financial loss, potential data leak via query history)
**Existing Mitigations:** OS keychain integration exists via save_api_key/get_api_key commands, but AppSettings.api_key field remains as a duplicate plaintext path; settings.json has no encryption or ACL enforcement
**Residual Risk:** Critical — the plaintext api_key field in AppSettings creates a second, unprotected storage path that bypasses the secure keychain. A compromised WebView or file read access is sufficient to extract the key.

### Scenario 12: MCP Port Scan Auto-Discovers Malicious Listener
**Threat:** The MCP discovery system's TCP port scan (ports 3100-3200) auto-discovers ANY TCP listener on localhost, including a malicious service placed by a compromised container process, and registers it as a trusted MCP server.

**Attack Path:**
1. Process inside container compromises or creates a child
2. Malicious process binds to a port in range 3100-3200 and serves a fake /health endpoint
3. MCP discovery scans the port range, detects the open TCP port
4. Malicious service is registered as an MCP server with `Online` status
5. Agent calls the fake server's tools, exposing tool calls and data to attacker

**Likelihood:** Medium (container has network_mode: none, so inside-container process needed; but PID namespace is shared per SC-52, allowing cross-process probing)
**Impact:** High (attacker can intercept MCP tool calls, steal tool arguments, or impersonate legitimate MCP tool responses)
**Existing Mitigations:** Probe only runs inside container (network isolation); user approval required per ADR-006; health probe requires HTTP 200 response
**Residual Risk:** High — no authentication on probe; any TCP listener responding with HTTP 200 is accepted as an MCP server; no protocol-level handshake validates server identity

---

## 5. Existing Security Controls Inventory

| Control ID | Category | Description | Verifiable? | Source |
|-----------|----------|-------------|-------------|--------|
| SC-01 | Container | Read-only rootfs (`read_only: true`) | ✅ Docker inspect | ADR-002 |
| SC-02 | Container | Network isolation (`network_mode: none`) | ✅ Docker inspect | ADR-002 |
| SC-03 | Container | No capabilities (`cap_drop: ALL`) | ✅ Docker inspect | ADR-002 |
| SC-04 | Container | No new privileges (`security_opt: no-new-privileges:true`) | ✅ Docker inspect | ADR-002 |
| SC-05 | Container | Non-root user (1000:1000) | ✅ Docker inspect | ADR-002 |
| SC-06 | Container | tmpfs hardening (`noexec,nosuid` on /tmp, /run) | ✅ Docker inspect | ADR-002 |
| SC-07 | Container | Resource limits (2GB RAM, 0.5 CPU, 256 PIDs) | ✅ Docker inspect | ADR-002 |
| SC-08 | Container | Init process (`init: true`) | ✅ Docker inspect | ADR-002 |
| SC-09 | Container | Health check (HTTP localhost:8080) | ✅ Docker inspect | ADR-002 |
| SC-10 | IPC | HTTP bridge on localhost only (no host port exposure) | ✅ Container config | ADR-003 |
| SC-11 | IPC | Tool execution allowlist (Security Engineer reviewed) | ✅ Code review | ADR-003 |
| SC-12 | IPC | LLM API keys never enter container (proxied via host) | ✅ Code review | ADR-003 |
| SC-13 | IPC | Unix domain socket (no direct network IPC) | ✅ Code review | ADR-003 |
| SC-14 | Host | API keys stored in OS keychain (Windows Credential Manager) | ✅ Code review | Security template |
| SC-15 | Host | bollard eliminates shell injection from Docker commands | ✅ Rust types | ADR-005 |
| SC-16 | Host | C6 validation: all frontend-controlled input validated on receive | ✅ Code review | AGENTS.md C6 |
| SC-17 | Host | C10 sanitization: no stack traces/paths/raw OS errors in IPC responses | ✅ Code review | AGENTS.md C10 |
| SC-18 | Host | C7 typed errors sanitized before crossing IPC | ✅ Code review | AGENTS.md C7 |
| SC-19 | Host | C9 graceful degradation on missing privileges | ✅ Code review | AGENTS.md C9 |
| SC-20 | Host | Tauri compiled with LTO + opt-level z (3-5 MB binary) | ✅ Build config | AEGIS-DESIGN |
| SC-21 | UI | HITL default-deny with 60s timeout | ✅ UX design | AEGIS-DESIGN |
| SC-22 | UI | Emergency kill button in HITL dialog | ✅ UX design | AEGIS-DESIGN |
| SC-23 | UI | No 'approve all' toggle | ✅ UX design | AEGIS-DESIGN |
| SC-24 | UI | Privacy badge (🔒 local / ☁ cloud) | ✅ UX design | AEGIS-DESIGN |
| SC-25 | UI | Sandbox status indicator (green/yellow/red) | ✅ UX design | AEGIS-DESIGN |
| SC-26 | UI | IPC connection lost → 'Sandbox disconnected' overlay with reconnect | ✅ UX design | AEGIS-DESIGN |
| SC-27 | MCP | Servers inside container only | ✅ Container config | ADR-006 |
| SC-28 | MCP | Tool registry allowlist-filtered | ✅ Code review | ADR-006 |
| SC-29 | MCP | User approval per connection (no persistent trust) | ✅ UX design | ADR-006 |
| SC-30 | CI | `cargo audit`, `cargo deny check`, `npm audit` per change set | ✅ CI config | AGENTS.md C5 |
| SC-31 | CI | Secrect scan (gitleaks or equivalent) per change set | ✅ CI config | AGENTS.md |
| SC-32 | Process | C12 dependency approval (Security Engineer blocking) | ✅ Process | AGENTS.md C12 |
| SC-33 | Process | S2 threat model per feature (this document) | ✅ Process | security-engineer.md S2 |
| | SC-53 | Settings | AppSettings.api_key field removed — API keys stored exclusively through OS keychain (Keychain::new) with no plaintext fallback | ❌ Not implemented — settings.rs AppSettings has plaintext api_key: String field persisted to unencrypted settings.json | Requires removal of api_key field from AppSettings, migration to Keychain-only access, and secure zeroization on drop |
| | SC-54 | MCP | MCP discovery port scan requires authenticated handshake (protocol-level token or challenge) before trust assignment | ❌ Not implemented — mcp/mod.rs probe_port() accepts any TCP listener responding with HTTP 200; no server identity verification | Requires protocol-level MCP discovery handshake; configurable port range with minimum 1 port; health response validation requiring MCP protocol signature |

---

## 6. Recommended Remediation Priority List

| Priority | ID | Recommendation | Rule Reference | Effort | Addresses Scenario |
|----------|----|---------------|----------------|--------|--------------------|
| **P1** | R1 | Add mutual authentication between AgentKit and HTTP bridge (shared secret or token) | S1, C6 | Medium | S5 (IPC Hijacking) |
| **P1** | R2 | Add custom seccomp profile blocking dangerous syscalls (mount, unshare, ptrace, perf_event_open) | S1 | Medium | S1 (Container Escape) |
| **P1** | R3 | Implement prompt injection detection for user messages that attempt to override agent guardrails | S1, C6 | High | S2 (Prompt Injection) |
| **P2** | R4 | Add per-agent workspace subdirectory isolation with strict permissions (UID-based) | S2 | Medium | S1 (Cross-Agent Tampering) |
| **P2** | R5 | Implement `secrecy` crate for in-memory API key zeroization in Rust backend | S5, C2 | Low | S3 (API Key Extraction) |
| **P2** | R6 | Pin Docker image to digest and implement cosign signature verification | S1, C12 | Medium | S6 (Supply Chain) |
| **P3** | R7 | Add structured audit logging with cryptographic chaining for all tool executions | S6, C2 | High | S2 (Repudiation) |
| **P3** | R8 | Implement log redaction in HTTP bridge and Tauri backend for sensitive fields (S6 list) | S6 | Medium | S4, S5 (Info Disclosure) |
| **P3** | R9 | Add AppArmor/SELinux profile for the sandbox container | S1 | Medium | S1 (Container Escape) |
| **P3** | R10 | Lock npm/pip packages inside container via pinned lockfiles and vendor verification | C12 | Low | S6 (Supply Chain) |
| **P4** | R11 | Add Tauri capabilities file with scoped permissions per command (least privilege) | S3 | Medium | UI Boundary Elevation |
| **P4** | R12 | Implement nonce-based command authorization for IPC calls from WebView | S1, C6 | High | Host Boundary Spoofing |
| **P4** | R13 | Add anomaly detection for high-entropy/encoded output (potential exfiltration attempt) | S2 | Medium | Container Info Disclosure |
| **P4** | R14 | Implement token-per-second rate limiting on agent LLM calls and bridge requests | S2 | Low | IPC Boundary DoS |

| **P1** | R15 | Implement authentication between agent process and tool exec bridge (bearer token) | S1, C6 | Medium | S5, S9 |
| **P1** | R25 | Implement allowed transition adjacency matrix for agent state machine (SC-41) | S1, S2 | Medium | State machine Tampering/EoP |
| **P1** | R26 | Bind session token to agent PID for state transitions (SC-42) | S1, S2 | Medium | State machine Spoofing |
| **P1** | R27 | Add state file HMAC integrity check before deserialization (SC-46) | S1, C6 | Low | State file Tampering |
| **P2** | R28 | Enforce max 1 PTY session per agent (SC-44) | S2 | Low | PTY DoS |
| **P2** | R29 | Add PTY input/output audit logging with agent attribution (SC-45) | S6 | Medium | PTY Repudiation |
| **P2** | R30 | Implement state file size limit and deny_unknown_fields serde guard (SC-47) | S2 | Low | Persistence DoS/EoP |
| **P2** | R31 | Add MCP tool registration signature verification (SC-48) | S1, S3 | Medium | MCP Spoofing |
| **P2** | R32 | Enforce MCP tool name uniqueness across servers (SC-49) | S3 | Low | MCP Spoofing |
| **P2** | R33 | Add MCP discovery audit logging (SC-50) | S6 | Low | MCP Repudiation |
| **P2** | R34 | Implement structured audit logging with trace_id for all tool exec (SC-51) | S6 | High | All Repudiation |
| **P3** | R35 | Configure PID namespace isolation for container (SC-52) | S1 | Low | PTY EoP |
| **P1** | R16 | Add state file integrity verification (HMAC or AEAD encryption) before deserialization | S1, C6 | Low | S8 |
| **P1** | R17 | Implement allowed transition adjacency matrix for agent state machine | S1, C6 | Medium | S11 |
| **P2** | R18 | Add PTY command input validation and allowlist enforcement | S1, C6 | Medium | S9 |
| **P2** | R19 | Enforce MCP tool name uniqueness across registered servers; validate tool origin | S3 | Low | S10 |
| **P2** | R20 | Implement structured audit logging with trace_id for all tool exec and state transitions | S6 | High | S5, S8, S11 |
| **P3** | R21 | Encrypt state files at rest with session-derived key | S5 | Medium | S8 |
| **P3** | R22 | Add MCP tool capability classification with Security Engineer approval for high-capability tools | S3, S1 | Medium | S10 |
| **P3** | R23 | Implement rate limiting on tool exec bridge and MCP discovery | S2 | Low | S5, S10 |
| **P4** | R24 | PTY output redaction for secret patterns | S6 | Low | S9 |
| **P0** | R36 | Remove plaintext api_key field from AppSettings struct — route all key access through Keychain exclusively | S1, S5, C2 | Medium | S11 (API Key Plaintext) |
| **P0** | R37 | Add MCP discovery protocol-level authentication — require health probe to present MCP protocol signature before trust assignment | S1, S3 | Medium | S12 (MCP Port Scan) |
| **P1** | R38 | Remove host `sh -c` fallback from tools/mod.rs exec_terminal — require bridge for all tool execution | S1, C6 | Low | Tool exec EoP |
| **P1** | R39 | Implement allowed transition adjacency matrix in AgentContext::transition_to() — validate and log all transitions | S1, S2 | Medium | State machine Tampering/EoP |
| **P2** | R40 | Replace DefaultHasher in HITL nonce with cryptographic hash (blake3 or SHA-256 via ring crate) | S1, S2 | Low | HITL Spoofing |
| **P2** | R41 | Add CancellationToken to PTY background reader tasks — cancel on session close to prevent resource leak | S2 | Low | PTY DoS |
| **P3** | R42 | Add #[serde(deny_unknown_fields)] to AppSettings, AgentPersistedState, AgentContext; add file size limit before deserialization | S2 | Low | State file Tampering/DoS |
| **P3** | R43 | Scope capabilities/default.json with per-command IPC permissions matching actual Tauri command surface | S3 | Medium | UI Boundary Elevation |
---

## 7. HITL (Human-In-The-Loop) Confirmation Flow — STRIDE Pass (Wave 7.1)

**Feature:** When the agent requests a dangerous operation (file deletion, command execution, network operations), the Rust backend emits a confirmation event to the frontend, the user approves/rejects via ConfirmationDialog, and the agent pauses at `PendingConfirmation` state until response. 30s timeout defaults to reject.

### Trust Boundary: WebView ↔ Rust IPC (HITL dialog channel)

| STRIDE Category | Threat Description | Affected Component | Risk | Existing Mitigations | Recommended Additional Controls |
|-----------------|-------------------|-------------------|------|---------------------|-------------------------------|
| **Spoofing** | Malicious WebView JS crafts a fake HITL approval event without user consent | Tauri IPC command `respond_hitl` | **High** | Tauri IPC validates message types; Rust side verifies dialog is actually open before accepting response | Add nonce/token to each HITL session: frontend must return the nonce with the response; Rust backend generates nonce per dialog and rejects mismatches (C6) |
| **Spoofing** | An attacker replays a previously captured HITL approval | IPC event replay | **Medium** | Nonce per dialog prevents replay (if implemented) | Add timestamp validation: reject responses older than 30s; clear nonce after use |
| **Tampering** | Frontend modifies the displayed operation details to trick the user | ConfirmationDialog display | **Medium** | Operation details serialized in Rust, sent as immutable payload; display-only in dialog | Hash the operation payload in Rust and display the hash in the dialog footer; reject any frontend-signed response that doesn't match original hash |
| **Tampering** | User's approval reason is modified in transit or storage | HITL log storage | **Low** | C10 sanitization: no raw data in IPC; struct is typed | Sign HITL audit log entries with an in-memory session key |
| **Repudiation** | User claims they never approved a destructive action | HITL approval logs | **Low** | HITL events logged with timestamp; 30s auto-deny fallback; no fast-approve path | Store structured audit log for every HITL event (dialog_created, user_approved, user_rejected, timeout); include operation_id, timestamp, user_reason |
| **Information Disclosure** | Operation details in HITL dialog leak sensitive information (file paths, IPs) | IPC event payload | **Low** | Dialog description is sanitized by Rust before emission | Add configurable redaction patterns for operation descriptions (S6), e.g., replace user home paths with `~` |
| **Denial of Service** | Agent sends thousands of HITL requests flooding the UI | ConfirmationDialog queue | **Low** | Single HITL at a time enforced by `PendingConfirmation` state; state machine rejects second request | Add per-session HITL rate limit (max 10 dialogs per minute); enforce dialog timeout |
| **Elevation of Privilege** | User approves an operation that the agent misdescribed | HITL approval on sanitized description | **Medium** | Dangerous operations are classified by configurable list; default-deny for unknown operations | Add risk level classification to each dangerous operation (Low/Medium/High/Critical); require warning color per level; require explicit checkbox for High+ before approve button activates |

### Trust Boundary: Agent State Machine (PendingConfirmation state)

| STRIDE Category | Threat Description | Affected Component | Risk | Existing Mitigations | Recommended Additional Controls |
|-----------------|-------------------|-------------------|------|---------------------|-------------------------------|
| **Tampering** | Agent loop continues processing while waiting for HITL confirmation | Agent `process_message` loop | **Medium** | `PendingConfirmation` state is terminal for the current message cycle; loop returns control to UI | Verify in review that `process_message` exits the loop when hitting PendingConfirmation; add explicit assertion that next iteration starts only after state != PendingConfirmation |
| **Denial of Service** | HITL timeout (30s) is too short for user to read complex operations | ConfirmationDialog timeout | **Low** | 30s timeout defaults to safe (reject), not approve | Make timeout configurable in settings (5s–120s range); warn user 5s before timeout with a countdown |
| **Elevation of Privilege** | Agent bypasses HITL by calling dangerous operation directly (tool not routed through guard) | Tool execution pipeline | **High** | All tool exec requests checked against dangerous ops list before execution | Enforce HITL check at the tool execution layer: any tool marked dangerous must pause at PendingConfirmation before executing; verify the check is not bypassable by changing tool names |

### Security Controls Added by Wave 7.1

| Control ID | Category | Description | Verifiable? |
|-----------|----------|-------------|-------------|
| SC-34 | HITL | Dangerous operations configurable list in settings (file_delete, command_exec, network_op) | ✅ Code review |
| SC-35 | HITL | 30s timeout defaults to reject (safe default) | ✅ Code review |
| SC-36 | HITL | PendingConfirmation state pauses agent until user responds | ✅ State machine review |
| SC-37 | HITL | Nonce-based HITL session (prevents replay/spoofing) | ✅ Code review |
| SC-38 | HITL | User reason optional but stored in audit log | ✅ Code review |
| SC-39 | HITL | Single HITL at a time enforced by state machine | ✅ State machine review |
| | SC-41 | State Machine | Allowed transition adjacency matrix enforced in Rust | ❌ Not implemented - state.rs has is_busy()/can_accept_input() helpers but no explicit transition adjacency matrix | Wave 4 - needs full implementation |
| | SC-42 | State Machine | Session token bound to agent PID for state transitions | ❌ Not implemented - no session token, PID binding, or transition authentication found in any source file | Wave 4 - needs implementation |
| | SC-43 | Tool Exec | Request bearer token for tool exec bridge calls | ❌ Not implemented - BridgeClient and ToolExecBridge have no auth token or bearer header | Wave 4 - needs implementation |
| | SC-44 | PTY | Max 1 PTY session per agent; session reuse policy | 🟡 Partial - PtyManager has session_count() but no max-1 enforcement; unlimited sessions can be created | Wave 4 - needs cap enforcement |
| | SC-45 | PTY | PTY input/output audit logging with agent attribution | ❌ Not implemented - pty/mod.rs has no audit logging; only eprintln! for stream errors | Wave 4 - needs implementation |
| | SC-46 | Persistence | State file integrity check (HMAC) before deserialization | ❌ Not implemented - persistence.rs reads and deserializes directly with no integrity check | Wave 4 - needs implementation |
| | SC-47 | Persistence | State file size limit and `deny_unknown_fields` serde guard | 🟡 Partial - atomic rename prevents partial-state corruption on crash, but no size limit or deny_unknown_fields on structs | Wave 4 - needs size limit + deny_unknown_fields |
| | SC-48 | MCP | Tool registration signature verification | ❌ Not implemented - ToolRegistry uses simple HashMap lookup; no registration signature check | Wave 4 - needs implementation |
| | SC-49 | MCP | Tool name uniqueness enforcement across MCP servers | ❌ Not implemented - McpRegistry keys by server ID, not tool name; no cross-server uniqueness check | Wave 4 - needs implementation |
| | SC-50 | MCP | MCP discovery audit logging (connect, register, disconnect) | ❌ Not implemented - mcp/mod.rs has no audit logging at all; no events emitted | Wave 4 - needs implementation |
| | SC-51 | All | Structured audit logging with trace_id for all tool exec calls | ❌ Not implemented - no trace_id or structured audit logging in bridge_client, tools/mod.rs, or hitl.rs | Wave 4 - needs implementation |
| | SC-52 | Container | PID namespace isolation prevents PTY descriptor probing by child processes | ❌ Not implemented - container.rs uses default PID namespace sharing; no PidMode configured | Wave 4 - needs implementation |
| SC-40 | HITL | No 'approve all' toggle or fast-approve path | ✅ UX design |

---

## 8. Review Triggers for Future Audits

The Security Engineer must perform a full review of `THREAT_MODEL.md` under the following conditions. Each trigger requires a new STRIDE pass on the affected component(s).

| Trigger | Component | Add/Update Threat Scenarios |
|---------|-----------|------------------------------|
| New feature adds cross-agent communication | Agent pool, IPC | S1-S6 for new boundary |
| Adding network access to sandbox (e.g., `network_mode: bridge`) | Container boundary | S5 (Exfiltration), S6 (External compromise) |
| New dependency added to Rust/TS stack | Supply chain, Docker image | S6 (Supply chain attack) |
| MCP server registry or marketplace feature | MCP boundary | S4 (MCP compromise), S1 (Tool spoofing) |
| Changes to credential storage mechanism (keychain → file) | Host boundary, API secrets | S3 (API key extraction) |
| Updater/Auto-update pipeline changes | Host boundary, signing | S6 (Updater MITM), S1 (Update spoofing) |
| Adding host filesystem access beyond workspace | Host boundary | S1 (Path traversal), S2 (Data tampering) |
| New IPC channel (WebSocket, named pipe) | IPC boundary | S1-S6 for new IPC |
| Adding multi-tenancy (multiple containers per user) | All boundaries | S1 (Container escape), S4 (Cross-tenant) |
| Pinning Docker image to digest changes or base image update | Container boundary, supply chain | S6 (Supply chain attack) |
| State machine transition model changes | Agent state machine | S1 (State spoofing), S2 (Unauthorized transition), EoP (Transition bypass) |
| Adding PTY multiplexing or multi-agent PTY access | PTY sessions | S1 (Session hijacking), S5 (PTY command injection) |
| MCP discovery mechanism change (polling vs push, auto-discovery) | MCP discovery | S1 (Tool spoofing), S4 (Fake registration) |
| Changes to state persistence format or storage location | State persistence | S2 (State tampering), S4 (Info disclosure), EoP (Deserialization) |
| Tool exec bridge protocol change or adding new transport (WebSocket, named pipe) | Tool exec bridge | S1-S6 for new IPC |
| Changes to API key storage mechanism (removing plaintext from AppSettings) | Settings, keychain | S3 (API key extraction), S5 (Plaintext exposure) |
| MCP discovery port range, host, or timeout configuration change | MCP discovery | S4 (Unauthorized registration), S1 (Malicious MCP server) |
| Changes to ToolRegistry fallback behavior (host execution vs bridge-only) | Tool execution bridge | EoP (Host escape via command execution) |
| Adding or modifying state machine transition rules in context.rs transition_to() | Agent state machine | S2 (Illegal transitions), EoP (Privilege escalation) |
| HITL nonce generation algorithm change | HITL confirmation | S1 (Replay attack), S4 (Nonce prediction) |
| PTY background reader task lifecycle change (new task spawning pattern) | PTY sessions | S5 (Resource leak), S2 (Data tampering via leaked buffer) |
| Serialization struct changes affecting serde deserialization boundaries | Persistence, settings | S2 (Data tampering), EoP (Deserialization) |
| Capabilities/default.json permission scope changes | Tauri IPC, frontend | EoP (Unauthorized IPC command execution) |

---

## Document History

| Date | Change | Author |
|------|--------|--------|
| 2026-07-08 | Initial creation — comprehensive STRIDE threat model across 4 boundaries with 6 attack scenarios, 33 security controls, and 14 prioritized recommendations | Security Engineer |
| 2026-07-10 | Wave 4 code verification refresh: SC-41/SC-52 verified against shipped code (10 ❌ Missing, 2 🟡 Partial); risk reassessed; §5 controls updated with evidence; §6 expanded with R25-R35 | Security Engineer |
| 2026-07-10 | **V2 Wave 4 deep code review** — 8 new critical/medium findings from source code audit; Settings plaintext API key exposure; MCP port scanning amplification; tool exec host fallback; PTY background task leak; HITL nonce weakness; state machine adjacency missing; capabilities scoping gap; update §3.5 STRIDE threats, §4 new scenarios 11-12, §5 new SC-53/SC-54, §6 new R36-R43, §8 new triggers | Security Engineer |
| 2026-07-15 | **Wave 7.3 refresh** — Code re-verification of all ❌ and 🟡 controls confirms no changes since 2026-07-10. SC-41/SC-52 remain 10 ❌, 2 🟡. Fixes applied this session (stray `O` char fix, unused var fix, `.a0proj` cleanup, `env/mod.rs` removal) are housekeeping only and do not impact any security control. `lib.rs` line endings normalized. Wave 6 streaming verified (not a security control change). All existing R36-R43 remediation priorities remain valid. See `MEMORY.md` for full session log. | Security Engineer |
