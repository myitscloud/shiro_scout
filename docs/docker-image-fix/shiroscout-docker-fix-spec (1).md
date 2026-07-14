# ShiroScout Docker Sandbox — Fix Specification

**Spec ID:** shiroscout-docker-fix-spec
**Date:** 2026-07-12
**Scope:** Docker sandbox subsystem — `Dockerfile.sandbox`, `entrypoint.sh`, `docker/bridge/`, `container.rs`, `bridge_client.rs`, `docker_client.rs`, `lib.rs` (bridge commands only)
**Inputs analyzed:** 8 build logs (Jul 10–12), all uploaded source files, the `agent-bridge` ELF binary
**Verification environment (Ring 1):** Linux container, rustc/cargo 1.97.0, clippy, shellcheck. No Docker daemon available — all `docker build` / runtime gates MUST be re-run on the Windows host (Ring 2, authoritative).

---

## 0. What the logs say actually happened

| Log | Timestamp | Outcome |
|---|---|---|
| `build.log` | Jul 10 | **FAILED** — NodeSource URL rendered as `setup_.x` (the `${NODE_MAJOR}` variable never expanded; literal `setup_\.x` in the Dockerfile) → HTTP 404 → `E: Unable to locate package nodejs` |
| `build2.log` | Jul 10 | Progressed past Node install (fix applied) |
| `build3.log` | Jul 10 | Shows `npm install @coinbase/agentkit@0.10.4` pulling `@x402/svm`, Solana kit, MetaMask SDK — see **F4** |
| `docker-build.log` | Jul 12 17:36 | **SUCCEEDED** |
| `docker-build-no-cache.log` / `-bg.log` | Jul 12 20:49 | **SUCCEEDED** |
| `build-final.log` / `build-detached.log.err` | Jul 12 21:14 | **SUCCEEDED** — `aegis-sandbox:latest` exported cleanly |

**Conclusion:** the image builds. The failures now live at runtime and in the architecture. `Dockerfile.sandbox` (23:02) and `entrypoint.sh` (23:47) were edited *after* the last successful build, so the current on-disk files have never been built.

**Predicted user-visible symptom:** any UI action hitting the Wave 3.3 commands (`sandbox_health`, `create_agent`, `run_agent`, `stop_agent`, `get_agent_status`) returns `Sandbox bridge unreachable: ... connection refused`, because of **F2**. The container itself likely starts and stays up; it is simply unreachable over HTTP by construction.

---

## 1. Findings

### F1 — BLOCKER: `docker/bridge/src/main.rs` is corrupted and cannot compile

11 lines contain literal `\"` sequences instead of `"` (lines 33, 121, 142, 164, 184, 212, 230, 269, 274, 290, 291).

**Proof:** `rustc --edition 2021 --emit=metadata src/main.rs` →
`error: unknown start of token: \  --> main.rs:33:22`

**Stale-binary evidence:** the `agent-bridge` ELF was built **Jul 10 04:05**; `main.rs` was last modified **Jul 10 20:46**. The binary baked into every successful image build is from an older revision of the source. The current source has never compiled.

**Corruption class:** identical to the BUILD_PLAN.md incident — an agent wrote the file through a JSON-escaped shell layer, double-escaping quotes on edited lines only (untouched lines have normal quotes). This is the second occurrence of this class. See §5 for the prevention rule.

**Status: FIXED and VERIFIED (Ring 1).** Repaired file delivered at `docker/bridge/src/main.rs`. Gates passed: `cargo check` ✅, `cargo clippy -- -D warnings` ✅ (three pre-existing warnings also cleaned: unused `std::process` imports, unused `nanos`, dead `config` field now documented + allowed as wire contract).

### F2 — BLOCKER (architectural): the HTTP bridge is unreachable by construction

Chain of facts, each verified in source:

1. `SandboxConfig::default()` → `network_mode: NetworkMode::None`; `AppSettings::default()` → `sandbox_air_gapped: true`. Containers run with `--network none`.
2. `build_host_config()` sets **no** `port_bindings` and no `publish_all_ports`. Even in bridge mode, port 8080 would not be published — and on Docker Desktop for Windows, unpublished container ports are never reachable from the host.
3. `BridgeClient::new(None)` targets `http://localhost:8080`. Nothing on the host listens there. Every call fails with connection refused.
4. Even if it were reachable: `run_agent_handler` in the bridge is an explicit stub — it flips a status flag and returns "Agent processing started" without dispatching anything.
5. The reverse path is equally dead: the bridge's `tool_exec_handler` forwards to `http://host.docker.internal:8081/api/execute-tool`, but (a) `network none` cannot reach `host.docker.internal`, and (b) the Tauri app runs no HTTP listener on 8081 — `lib.rs` registers no such server.

Meanwhile, the **working** path already exists: `ToolExecBridge` / `exec_in_container` / the PTY session commands go through the Docker daemon's named pipe via bollard. `docker exec` requires **no container networking at all** — it is fully compatible with `network none`. The comment atop `bridge_client.rs` ("Replaces the old HTTP proxy with direct bollard exec calls... ADR-003") shows the pivot was already decided; it was just never finished. The HTTP half was left wired into `lib.rs` as Wave 3.3.

**Resolution: Decision ADR-004 required — see §2. Option A (retire the HTTP bridge) is recommended.**

### F3 — HIGH: container user `1000:1000` does not exist in the image

`create_sandbox` sets `user: Some("1000:1000")`, but the Dockerfile declares `SANDBOX_UID/GID/USER` ARGs and never uses them — no `groupadd`, no `useradd`, no `USER` directive. Consequences under read-only rootfs:

- Process runs as an anonymous uid with no passwd entry and no writable `$HOME`.
- `xfce4-session` and `x11vnc` try to write config to `$HOME` and fail (they are backgrounded, so failures are silent).
- Playwright browsers were installed at build time as root into `/root/.cache/ms-playwright`; `/root` is mode 0700 — **uid 1000 cannot read the browsers at all**.

**Status: FIXED in delivered `Dockerfile.sandbox`** (Section 3 creates the user; Section 4 installs browsers to world-readable `/opt/ms-playwright` via `PLAYWRIGHT_BROWSERS_PATH`). Requires companion change **W-A5** in `container.rs` (tmpfs for `/home/agent`).

### F4 — HIGH: the entrypoint targets the wrong "AgentKit" entirely

`entrypoint.sh` §2 writes `/tmp/run-agent.ts` importing `@coinbase/agentkit`. That package is **Coinbase's crypto/on-chain agent framework** — build3.log proves it, pulling `@x402/svm` (Solana payments), `@solana/kit`, and MetaMask SDK as dependencies. It gives AI agents *wallets*, not *coding tools*. The name collision fooled the coding agent. Additional dead-on-arrival details:

- The package is not installed in the current image (that npm install was removed after build3).
- Nothing installs `ts-node`/`tsx`, so a `.ts` entry file couldn't run anyway.
- `AgentKit.configure({ apiKey })` is not the real Coinbase API either — the snippet is hallucinated end to end.

**Status: FIXED — deleted in delivered `entrypoint.sh`.** Agent runtime is host-side per ADR-003; nothing agent-shaped belongs in the entrypoint.

### F5 — MEDIUM: image bloat and non-reproducible Playwright install

- `xfce4` + `xfce4-goodies` installs a full desktop environment (hundreds of packages — this is the font blizzard in every log and a large share of the 3–8 minute builds) inside a headless sandbox. `tightvncserver` AND `x11vnc` are two competing VNC stacks. A desktop session serves no purpose for Playwright; observation of headed browser runs needs only Xvfb + x11vnc.
- `npx playwright install chromium` fetches whatever Playwright is latest at build time (unpinned → non-reproducible) and does not persist the `playwright` npm package for runtime use.
- `npm install -g --legacy-peer-deps npm@latest` mutates the toolchain per build for no benefit.

**Status: FIXED in delivered `Dockerfile.sandbox`:** desktop stack removed (Xvfb + x11vnc retained, VNC gated behind `ENABLE_VNC=1`), `playwright@1.61.1` pinned (current stable as of 2026-07-12 — re-pin deliberately when upgrading), installed globally with `--with-deps chromium`, browsers at `/opt/ms-playwright`. Expect a dramatically smaller image and faster builds.

### F6 — MEDIUM: hard-coded container name → 409 on every recreate

`CreateContainerOptions { name: "aegis-sandbox" }`. If a container by that name exists (running, stopped, or orphaned by an app crash), `create_sandbox` fails with `409 Conflict ... name already in use` until it is manually removed. Classic "worked once, broken forever" failure.

**Fix (W-A4):** before create, inspect for an existing container named `aegis-sandbox` (or better, filter by label `com.shiroscout.sandbox=true`, which the new Dockerfile sets); if found, force-remove it, then create. Idempotent create is the correct semantic for a singleton sandbox.

### F7 — MEDIUM → **CONFIRMED IN PRODUCTION (2026-07-13 diagnostics)**: empty `workspace_path` = nowhere to work

> Field evidence: `Diagnose-SandboxMount.ps1` on the dev host showed `settings.json → workspace_path: ''` while a *manually created* container with a `C:\projects:/workspace:rw` bind displayed host files perfectly (9p/drvfs mount healthy). Conclusion: Docker/WSL2 plumbing is fine; app-created containers simply never receive a bind. This is the root cause of "agents see an empty /workspace." Note the diagnostic container also showed `User: ''` and `ReadonlyRootfs: False` — it was not created by current `container.rs` code, and it squats the `aegis-sandbox` name (F6): it must be `docker rm -f`'d before the app can recreate.

`binds` is only populated when `workspace_path` is non-empty; `AppSettings::default().workspace_path` is `""`. Result: a fresh install gets a sandbox whose `/workspace` is read-only and whose only writable paths are `/tmp` (noexec) and `/run`. Every agent file-write fails.

**Fix (W-A5):** in `build_host_config`, when `workspace_path` is empty, add `/workspace` to the tmpfs map (`rw,nosuid,size=512M`) as a fallback scratch space; additionally always add `/home/agent` tmpfs (`rw,nosuid,size=64M`) to support F3. Optionally surface a UI warning that no persistent workspace is mounted.

### F8 — LOW: `set_sandbox_network_mode` is a no-op that reports success

It returns the input value; enforcement is delegated (per its comment) to a frontend rebuild that may or may not happen. HITL approves a "dangerous operation" that changes nothing by itself. **Fix (W-A6):** derive `network_mode` in `create_sandbox` from persisted settings (`sandbox_air_gapped`) rather than trusting the passed-in `SandboxConfig`, so the setting is enforced at the only place it matters. Keep the command as a settings mutation, not a fake action.

### F9 — LOW: `iso_timestamp()` produced garbage dates

It hardcoded `"2026-07-{:02}"` and set the day to `10 + days_since_unix_epoch` → dates like `2026-07-20657`. **Status: FIXED and VERIFIED** in the delivered `main.rs` (dependency-free civil-from-days algorithm; assertions checked for 1970-01-01, 2026-07-12, and leap day 2000-02-29).

### F10 — LOW: build-system footguns

- **reqwest native-tls:** `docker/bridge/Cargo.toml` uses reqwest 0.12 default features → requires OpenSSL headers on the build machine (this bit during Ring 1 verification). If Option B is ever chosen, switch to `reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }`. Under Option A this is moot.
- **CRLF:** the current `Dockerfile.sandbox` has CRLF line endings (FILEOPS violation class; caused the `NoEmptyContinuation` warning in build.log). A CRLF shebang in `entrypoint.sh` would have been fatal (`exec: no such file or directory`) — it was LF, narrowly. All delivered files are LF; enforce with `.gitattributes` (`docker/** text eol=lf`).

### F11 — HIGH (field-discovered 2026-07-13): settings schema migration failure — old `settings.json` cannot deserialize

The on-disk `settings.json` predates the Wave 7 fields (`sandbox_air_gapped`, `hitl_timeout_secs`, `dangerous_operations` are absent). `AppSettings` derives `Deserialize` with **no `#[serde(default)]`** on any field, so `serde_json::from_str` fails with `missing field` on every launch against an old file. Likely frontend behavior: treat load error/None as first-run → defaults → the user's saved `workspace_path` never loads, and may appear to "not stick." This also means **every future field added to `AppSettings` silently bricks existing installs' settings.**

**Fix (W-A10):**
1. Add `#[serde(default)]` at the struct level on `AppSettings` (and on `DangerousOperationsConfig`), so missing fields take `Default::default()` values. Struct-level default requires the existing `impl Default` — already present. Do the same for `LlmSettings` if it exists.
2. In `load_settings`, on parse failure do NOT discard the file: log the error, back up the unreadable file to `settings.json.bak`, and return defaults — never silently overwrite user data.
3. Regression test: deserialize a JSON fixture containing only v1 keys (`theme`, `provider`, `model`, `api_key`, `workspace_path`, `mount_workspace`) and assert it loads with defaults for the rest.
4. Verify on the dev host: set workspace in the UI → save → `Get-Content $env:APPDATA\com.shiroscout.app\settings.json` must show the non-empty `workspace_path` AND the new-schema keys. If `workspace_path` is still `''` after saving, the save path has an additional bug — capture verbatim output per blocker protocol.

Related nit: `AppSettings.mount_workspace` exists but `SandboxConfig` has no such field — confirm the frontend actually gates the bind on it when mapping settings → `SandboxConfig`, or delete the setting.

---

## 2. ADR-004 (decision required): retire or revive the HTTP bridge

### Option A — RECOMMENDED: exec-only architecture (finish the ADR-003 pivot)

The container is a passive tool-execution environment. The host owns agent lifecycle, LLM calls, and orchestration; every command enters via `docker exec` (bollard). No in-container server, no ports, no network. Air-gap claim becomes *true by construction* instead of contradicted by the design.

**Why:** it's the architecture 80% built already (ToolExecBridge, exec_sandbox_command, PTY sessions); it is the only design compatible with `network none`; it deletes an entire attack/maintenance surface; it kills F2 outright.

**Work items:**

- **W-A1 (`lib.rs`):** remove the five Wave 3.3 commands (`sandbox_health`, `create_agent`, `run_agent`, `stop_agent`, `get_agent_status`) and their `invoke_handler` registrations. Replace health with an exec-based probe: new command `sandbox_health(container_id)` → `exec_shell_in_container(id, "echo ok")`, healthy iff exit 0. Agent lifecycle (create/run/stop/status) routes to the host-side `agent` module (state already lives in `AppAgentState`), not to the container.
- **W-A2 (`bridge_client.rs`):** delete the `BridgeClient` struct and all HTTP methods plus the now-unused request/response mirror types (`AgentInfo`, `CreateAgentRequest`, …, `HealthResponse`). Keep `ToolExecBridge`, `ToolExecResult`, `BridgeError` (drop the `BridgeError::BridgeError`/`ParseError` variants if nothing references them after deletion). Remove `reqwest` from the **app's** Cargo.toml if no other module uses it.
- **W-A3 (frontend):** update any `invoke("sandbox_health" | "create_agent" | ...)` call sites to the new command names/signatures. Grep target: `invoke(` across `src/`.
- **W-A4 (`container.rs`):** idempotent create per **F6** (label-based find + force-remove before create).
- **W-A5 (`container.rs`):** tmpfs fallbacks per **F7** (`/home/agent` always; `/workspace` when no bind).
- **W-A6 (`container.rs` / `settings.rs`):** enforce `sandbox_air_gapped` at create time per **F8**.
- **W-A7 (docker/):** adopt delivered `Dockerfile.sandbox` + `entrypoint.sh`. Delete `docker/bridge/` and the stale `agent-bridge` binary from the build context, remove `EXPOSE`, rebuild image. (Keep the repaired `main.rs` in the repo history if you want the Option B escape hatch.)
- **W-A8 (repo hygiene):** `.gitattributes` LF rule; delete the eight build logs from the working tree (archive if desired).

### Option B — NOT recommended: revive the HTTP bridge

Required to make it real: switch containers to bridge networking **(breaks the air-gap ADR — this alone should end the discussion)**, add `port_bindings` for 8080 (host-port collision management on user machines becomes your problem), implement actual agent dispatch in `run_agent_handler` (currently a stub), either add a Tauri-side HTTP listener on 8081 or delete `tool_exec_handler`, swap reqwest to rustls (F10), and set up a cross-compilation pipeline so the Linux `agent-bridge` binary is rebuilt from source on your Windows host every time it changes — the stale-binary trap (F1) exists *because* this pipeline doesn't. High cost, negative security value, duplicates a working mechanism.

The repaired, gate-passing `main.rs` is delivered regardless, so Option B stays possible without archaeology.

---

## 3. Verification gates (Ring 2 — Windows host, authoritative)

Run in order; stop at first failure and report verbatim output per blocker protocol.

**G1 — Stub scan:** confirm no code path still claims agent dispatch happens in-container. `grep -rn "run-agent\|8080\|8081\|BridgeClient" src-tauri/src/` → expected: no live references after W-A1/W-A2 (comments/history acceptable).
**G2 — Corruption scan:** `grep -rn '\\"' src-tauri/ --include=*.rs` → expected: zero matches.
**G3 — `cargo check`** (workspace) → clean.
**G4 — `cargo clippy -- -D warnings`** → clean.
**G5 — `cargo test`** → all pass. NOTE: `test_network_mode_default` currently asserts the Default is Bridge (`!is_air_gapped()`) while `SandboxConfig::default()` uses None — after W-A6, make the enum Default `None` and update this test; air-gapped must be the default posture, and the test should encode that.
**G6 — Frontend:** `pnpm typecheck && pnpm build` → clean.
**G7 — Image build:** `docker build -f src-tauri/docker/Dockerfile.sandbox -t aegis-sandbox:latest src-tauri/docker/` → succeeds. Expect a much smaller image than before (desktop stack removed); record the size in DECISIONS.
**G8 — Runtime smoke (mirrors production HostConfig):**
```
docker run -d --name aegis-smoke --network none --read-only --init ^
  --cap-drop ALL --security-opt no-new-privileges:true --pids-limit 256 ^
  --tmpfs /tmp:rw,noexec,nosuid,size=256m --tmpfs /run:rw,noexec,nosuid,size=64m ^
  --tmpfs /home/agent:rw,nosuid,size=64m --tmpfs /workspace:rw,nosuid,size=512m ^
  aegis-sandbox:latest
docker exec aegis-smoke id            → uid=1000(agent) gid=1000(agent)
docker exec aegis-smoke node --version    → v22.x
docker exec aegis-smoke python3 --version → Python 3.11.x
docker exec aegis-smoke playwright --version → Version 1.61.1
docker exec aegis-smoke ls /opt/ms-playwright → chromium-* directory present
docker exec aegis-smoke sh -c "echo hi > /workspace/t && cat /workspace/t" → hi
docker logs aegis-smoke               → "[entrypoint] Sandbox ready (exec-only mode)."
docker rm -f aegis-smoke
```
**G9 — App-level round trip:** launch app → `check_docker_daemon` OK → `create_sandbox` → `start_sandbox` → `exec_sandbox_command("echo hello")` returns `exit_code: 0, stdout: "hello\n"` → new exec-based `sandbox_health` returns healthy → `stop_sandbox` → **immediately `create_sandbox` again without manual cleanup** (F6 regression test — must succeed).

---

## 4. Known constraints (document, don't fight)

- **K1 — Chromium sandbox:** with `cap_drop ALL` + `no-new-privileges`, Chromium's own sandbox cannot initialize. Playwright launches must set `chromiumSandbox: false` (args `--no-sandbox`). This is standard for hardened containers — the *Docker* boundary is the sandbox.
- **K2 — No runtime package installs:** read-only rootfs + `network none` means pip/npm at runtime is impossible *by design*. Anything agents need must be baked at image build. Treat "agent needs package X" as a Dockerfile change with a rebuild, not a runtime action.
- **K3 — `/tmp` is noexec:** interpreted scripts run fine (`python3 /tmp/x.py`, `bash /tmp/x.sh`); dropped *binaries* in `/tmp` will not execute. Intentional.

---

## 5. Prevention — proposed FILEOPS-031 (quote-corruption class, 2nd occurrence)

> Agents writing source files MUST use a mechanism that cannot re-escape content: heredoc with a **quoted** delimiter (`cat > file << 'EOF'`), base64-decode, or a native file-write tool. Writing code through JSON-escaped `echo`/`printf`/`Set-Content` string interpolation is prohibited. **Post-write gate:** `grep -n '\\"' <file>` on every `.rs/.ts/.tsx/.ps1` write; any match fails the step.

Two incidents (BUILD_PLAN.md, bridge main.rs) make this a pattern, not bad luck.

---

## 6. Deliverables manifest

| File | Status |
|---|---|
| `shiroscout-docker-fix-spec.md` | this document |
| `docker/Dockerfile.sandbox` | rewritten (Option A); desk-checked; **must pass G7 on Ring 2** |
| `docker/entrypoint.sh` | rewritten (Option A); shellcheck clean, `bash -n` clean, LF |
| `docker/bridge/src/main.rs` | repaired + cleaned; **Ring 1 verified:** `cargo check` ✅, `clippy -D warnings` ✅; keep only if Option B is ever chosen |

Ring 1 could not run `docker build` (no daemon in the analysis sandbox); G7–G9 are Ring 2 obligations.
