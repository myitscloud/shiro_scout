# ShiroScout — BUILD_PLAN.md

> **Purpose:** Execution plan — waves, items, owners, order. Scope lives in `FEATURES.md`; this file says *when* and *who*.
> **Rebuilt 2026-07-10** — previous file was corrupted (single row duplicated ~250×, a PowerShell file-write casualty; see DEC-005).
> **⚠ Corruption guard:** this file is ≤ 250 lines and is ONLY updated by full-file rewrite (FILEOPS-001). Never patch it with sed loops or append scripts.

Status: ✅ done · 🟡 in progress · 🔲 not started · ⏸️ blocked

---

## 1. Wave Overview

| Wave | Title | Status | Notes |
|:----:|-------|:------:|-------|
| 0 | Orchestrator Agent Core | ✅ | Docs + Rust core complete |
| 1 | Scaffold & Toolchain | 🟡 | Closeout items only (see below) |
| 2 | Scaffold & Toolchain (initial pass) | ✅ | |
| 3 | Docker Orchestration | ✅ | Sandbox + axum bridge live |
| 4 | AgentKit Runtime | 🔲 | **Next major wave** |
| 5 | Core UI — Design System & Components | ✅ | 14 components shipped |
| 6 | LLM Integration | 🟡 | Only 6.7 streaming open (+ drift re-verify) |
| 7 | Security Hardening & HITL | 🔲 | |
| 8 | Distribution & Release | 🟡 | 8.2 code signing design in progress, 8.3 updater design complete |

**Current priority order:** Wave 1 closeout → Wave 6.7 → Wave 4 → Wave 7 → Wave 8.

## 2. Wave 1 — Scaffold & Toolchain closeout 🟡

| Item | Task | Owner | Deps | Status |
|------|------|-------|------|:------:|
| 1.A | Baseline: run full DONE.md gate sequence on current tree, record report | QA | — | 🔲 |
| 1.B | `.gitattributes` (`* text=auto eol=lf` + binary entries); convert stray CRLF files | Architect | — | 🔲 |
| 1.C | Complete `cargo-deny` config (licenses, advisories, bans); add to gates | Security | 1.A | 🔲 |
| 1.D | `git init`, initial commit, GitHub remote, push | DevOps | 1.B | 🔲 |
| 1.E | Tauri shell IPC completion check — every stub in `lib.rs` implemented or `// BLOCKED: BLK-n` | Architect | 1.A | 🔲 |

## 3. Wave 6 — LLM Integration 🟡

| Item | Task | Owner | Deps | Status |
|------|------|-------|------|:------:|
| 6.1–6.3 | Provider crates (async-openai, DeepSeek, OpenAI) | Architect | — | ✅ |
| 6.4 | Settings > LLM Providers UI (3-role pattern) | Frontend | — | ✅ (re-verify 6.V) |
| 6.5 | API key management (Windows Credential Manager via keyring) | Architect | — | ✅ (re-verify 6.V) |
| 6.6 | Token usage tracking + cost estimation | Frontend | — | ✅ |
| 6.7 | Streaming responses: Rust `emit` → IPC events → StreamingText | Frontend (+Architect, C3) | 1.A | 🔄 |
| 6.8 | Provider health check + failover | QA + Reviewer | — | ✅ (re-verify 6.V) |
| 6.V | Drift re-verification of 6.4/6.5/6.8 with DONE.md reports | QA | 1.A | 🔲 |

## 4. Wave 4 — AgentKit Runtime 🔲

| Item | Task | Owner | Deps | Status |
|------|------|-------|------|:------:|
| 4.1 | Agent state machine (idle → thinking → tool → done) — finish from Wave 0 | Architect | 6.7 | 🔲 |
| 4.2 | Tool execution bridge (Rust → Docker exec via bollard) | Architect | 4.1 | 🔲 |
| 4.3 | Persistent PTY shell sessions (see docs/true-state-preservation.md) | Architect | 4.2 | 🔲 |
| 4.4 | Agent state persistence across app restarts | Architect | 4.1 | 🔲 |
| 4.5 | MCP server discovery per ADR-006 | Architect | 4.2 | 🔲 |
| 4.6 | Runtime test suite: state transitions + bridge failure paths | QA | 4.2 | 🔲 |

## 5. Wave 7 — Security Hardening & HITL 🔲

| Item | Task | Owner | Deps | Status |
|------|------|-------|------|:------:|
| 7.1 | HITL confirmation flow for dangerous operations (P0) | Frontend + Security | 4.2 | 🔲 |
| 7.2 | Air-gapped mode (no-network container profile) | Security | 4.2 | 🔲 |
| 7.3 | Threat model refresh vs. shipped Wave 4 surface | Security | 4.x | 🔲 |
| 7.4 | Secret-scan CI step (gitleaks) | DevOps | 1.D | ✅ |
| 7.5 | Capabilities re-audit (minimal perms vs. actual usage) | Security | 4.x | 🔲 |

## 6. Wave 8 — Distribution & Release 🟡

| Item | Task | Owner | Deps | Status |
|------|------|-------|------|:------:|
| 8.1 | Windows packaging (MSI/NSIS via Tauri bundler) | DevOps | 7.x | 🔲 |
| 8.2 | Code signing pipeline | DevOps | 8.1 | 🟡 |
| 8.3 | Tauri self-updater + release channel | DevOps | 8.2 | 🔲 |
| 8.4 | ARM64 target build + smoke test | DevOps | 8.1 | 🔲 |
| 8.5 | Ring 2 (Windows) authoritative full-gate release run | QA + DevOps | 8.1 | 🔲 |

### 6.1 Code Signing Pipeline Design (Item 8.2)

**Purpose:** Sign every shipped binary and installer with an Authenticode signature and RFC 3161 timestamp (D4), using a managed/cloud signing service so private keys never live in the repo or CI secrets.

#### Certificate Options

| Option | Certificate type | Key custody | SmartScreen rep | Cost | Recommendation |
|--------|:----------------:|:-----------:|:----------------:|:----:|:--------------:|
| **Azure Trusted Signing** | Standard (Microsoft-rooted) | Microsoft-managed HSM via OIDC | Builds reputation via publisher identity | Pay-per-signature | ✅ **Primary** |
| EV Code Signing (hardware token) | EV | Hardware token (USB HSM) | Immediate reputation lift | $200–600/yr + HSM logistics | ❌ Ops overhead, token logistics |
| Standard Code Signing (software) | Standard | PFX file in secure store | Slow reputation build | $200–400/yr | ❌ Key custody risk, manual rotation |

**Recommendation:** Azure Trusted Signing (managed service). Zero private key custody, OIDC auth from CI, no hardware tokens, meets D4 requirements. Aligns with the "prefer a managed/cloud signing service" directive from the Release DevOps Engineer role profile.

#### Signing Tools

| Tool | Platform | Best for | RFC 3161 | Notes |
|------|----------|----------|:--------:|-------|
| **signtool.exe** (Windows SDK) | Windows | Primary tool on Windows runners | ✅ `-tr http://timestamp.digicert.com -td sha256` | Installed with Windows SDK / Visual Studio Build Tools. D1 mandates Windows runners — this is the native path. |
| **osslsigncode** | Linux / Cross-compile | Fallback when signing from Linux container | ✅ `-ts http://timestamp.digicert.com` | Install via `apt install osslsigncode` or `cargo install osslsigncode`. Needed if signing step runs in the Linux container during cross-compile builds. |
| **Azure Trusted Signing action** | GitHub Actions (cross-platform) | Managed Azure signing | ✅ Built-in | GitHub Action `azure-trusted-signing-action` with OIDC federation — no tool install needed. |

**Primary recommendation:** `signtool.exe` on Windows runners (matches D1). Use Azure Trusted Signing GitHub Action for managed key custody. Use `osslsigncode` only as fallback for Linux-side signing steps.

#### Pipeline Stage — When Signing Happens

```
Tauri build (MSI + NSIS)
     ↓
1. Sign all internal binaries (EXEs, DLLs, etc.) inside the installer
     ↓
2. Sign the MSI wrapper
     ↓
3. Sign the NSIS wrapper (if shipped)
     ↓
4. Verify every signature
     ↓
5. Attach signed artifacts to release
```

Signing MUST happen **after** the Tauri bundler finishes (so it never signs intermediate build artifacts). Internal binaries are signed first, then the installer containers (MSI/NSIS). Each step uses the same signing identity.

#### Timestamp Server

| Server | CA | RFC 3161 | Notes |
|--------|:--:|:--------:|-------|
| `http://timestamp.digicert.com` | DigiCert | ✅ | Industry standard, broad CA compatibility |
| `http://timestamp.sectigo.com` | Sectigo | ✅ | Alternative DigiCert-independent path |
| Azure Trusted Signing (built-in) | Microsoft | ✅ | Automatic; no separate server config needed |

**Recommendation:** Use the timestamp server that ships with the signing certificate. For Azure Trusted Signing, use its built-in timestamping. For standalone certificates, use `http://timestamp.digicert.com` with `-tr` (RFC 3161). Mandatory per D4 — signatures must outlive the certificate.

#### CI Integration (GitHub Actions)

```yaml
# Pseudocode — GitHub Actions workflow snippet
jobs:
  sign:
    runs-on: windows-latest           # D1: Windows runners
    permissions:
      id-token: write                 # OIDC for Azure Trusted Signing

    steps:
      - uses: actions/checkout@v4
      - ... build steps ...
      - uses: tauri-apps/tauri-action@v0
        id: tauri-build
      
      # Step: Sign all binaries then installer
      - name: Sign with Azure Trusted Signing
        uses: azure/trusted-signing-action@v0
        with:
          files: |
            ${{ steps.tauri-build.outputs.artifact-path }}
          certificate-profile-id: 'ShiroScoutCodeSigning'
          identity: 'https://eus.codesigning.azure.net'
```

**Alternative (standalone cert via signtool.exe):**
```yaml
- name: Sign MSI with signtool
  run: |
    & "C:/Program Files (x86)/Windows Kits/10/bin/10.0.20348.0/x64/signtool.exe" sign `
      /fd SHA256 `
      /a `
      /f $env:CODE_SIGN_CERT `
      /p (ConvertFrom-SecureString $env:CODE_SIGN_PASSWORD) `
      /tr http://timestamp.digicert.com `
      /td SHA256 `
      /v `
      /debug `
      target/release/bundle/msi/ShiroScout_*.msi
  env:
    CODE_SIGN_CERT: ${{ secrets.CODE_SIGN_PFX_BASE64 }}
```

#### Cross-compilation from Linux (osslsigncode)

When building from the Linux container (Ring 1) for Windows targets, osslsigncode handles Authenticode without Windows:

```bash
# Install
sudo apt install osslsigncode -y

# Sign MSI
osslsigncode sign \
  -pkcs12 /etc/ssl/certs/shiroscout-code-sign.p12 \
  -pass env:SIGN_PASS \
  -n "ShiroScout" \
  -i "https://shiroscout.com" \
  -ts http://timestamp.digicert.com \
  -in src-tauri/target/release/bundle/msi/ShiroScout_0.1.0_x64_en-US.msi \
  -out signed/ShiroScout_0.1.0_x64_en-US.msi

# Verify
osslsigncode verify -in signed/ShiroScout_0.1.0_x64_en-US.msi
```

> **Note:** osslsigncode requires the PKCS#12 file (`.p12`/`.pfx`) on disk. For security, the PKCS#12 should be decrypted from CI secrets at signing time, never stored on the build agent.

#### D4 Compliance Checklist

- [ ] Every shipped binary and installer is Authenticode-signed
- [ ] RFC 3161 timestamp applied (not only SHA-256 digest)
- [ ] Private key never in repo or plain CI secrets (managed service or hardware-backed)
- [ ] Signature verified after every signing step
- [ ] Unsigned artifacts never shipped — even prereleases
- [ ] Signing identity matches publisher field (`"ShiroScout Team"` in tauri.conf.json)

#### Key Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Expired timestamp server cert | Use RFC 3161 timestamps that outlive signing cert; monitor timestamp service status |
| Azure Trusted Signing auth failure (OIDC) | Add fallback with `osslsigncode` and a backup certificate in separate secure storage |
| CI runner not on Windows (Linux-only pipeline) | Switch to `osslsigncode` as the exclusive signing path, with PKCS#12 from secrets |
| Signature verification on ARM64 MSI | Include `signtool.exe verify /pa` in the build step; test on ARM64 runner |
| Signing slows CI significantly | Cache signed artifacts in build pipeline; run signing in parallel with other post-build tasks |

#### Dependencies

| Dependency | Type | Required by |
|------------|:----:|:-----------:|
| 8.1 — MSI/NSIS packaging | Build-time | Signing runs after bundler |
| Azure Trusted Signing account | Service | Certificate identity |
| GitHub OIDC federation | Infrastructure | CI → Azure auth |
| signtool.exe (Windows SDK) | Tool | Windows runner signing |
| OR osslsigncode | Tool | Linux container signing (fallback) |

### 6.2 Tauri Self-Updater + Release Channel (Item 8.3)

**Purpose:** Implement Tauri 2 self-updater (`tauri-plugin-updater`) with a release channel strategy (stable/beta/nightly), ed25519 signature verification, and a release server specification.

#### Release Channel Strategy

| Channel | Purpose | Frequency | Version suffix | Users |
|---------|---------|:---------:|:--------------:|-------|
| **stable** | Production releases | Monthly or per-milestone | `x.y.z` (SemVer) | All users (default) |
| **beta** | Pre-release QA | Weekly | `x.y.z-beta.n` | QA team, early adopters |
| **nightly** | CI-built bleeding edge | Daily (automated) | `x.y.z-nightly.YYYYMMDD` | Developers, CI verification |

**Channel selection mechanism:**
- The MSI/NSIS installer writes the channel choice to the Windows registry (`HKCU\Software\ShiroScout\update-channel`)
- Tauri updater endpoints are templated per channel: `https://releases.shiroscout.app/{channel}/{{target}}/{{current_version}}`
- Users can switch channels via Settings > Updates (backend reads registry → calls `updater.check()` with the new endpoint)
- `stable` is the default for fresh installs; downgrade from beta/nightly → stable requires manual intervention

#### Self-Updater Configuration Design

**tauri.conf.json `plugins.updater` block:**

```json
{
  "plugins": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://releases.shiroscout.app/stable/{{target}}/{{current_version}}"
      ],
      "pubkey": "<ed25519-public-key>",
      "dialog": true
    }
  }
}
```

- `endpoints`: Templated URL per channel, supports `{{target}}` (e.g., `x86_64-pc-windows-msvc`) and `{{current_version}}` (e.g., `0.1.0`)
- `pubkey`: Ed25519 public key in base64 — generated once per release keypair, committed to repo (public key only)
- `dialog: true`: Tauri shows native update dialog with progress bar and install/remind button

**Frontend event listeners (to be added in src/App.tsx or a hook):**

```typescript
import { listen } from '@tauri-apps/api/event';
import { checkUpdate, installUpdate } from '@tauri-apps/api/updater';

// Check on app startup
useEffect(() => {
  checkUpdate().then((result) => {
    if (result.shouldUpdate) {
      // Optionally show a custom notification
      installUpdate();
    }
  });
}, []);

// Listen for status events
listen('tauri://update-status', (event) => {
  console.log('Update status:', event.payload);
});
```

#### Signature Verification Flow (Ed25519 Keypair)

```
┌──────────────────────────────────────────┐
│           Release Engineering             │
├──────────────────────────────────────────┤
│ 1. Generate offline Ed25519 keypair:     │
│    `tauri signer generate -w ~/.tauri/   │
│    shiroscout-update.key`                │
│                                          │
│ 2. Private key: `shiroscout-update.key`  │
│    → NEVER in repo or CI                 │
│    → Store in hardware-backed vault      │
│      (Azure Key Vault, YubiKey, or        │
│       offline air-gapped machine)         │
│                                          │
│ 3. Public key: embedded in               │
│    tauri.conf.json `plugins.updater.     │
│    pubkey` → committed to repo            │
└──────────────────────────────────────────┘
         ↓
┌──────────────────────────────────────────┐
│         Build Pipeline (GitHub CI)        │
├──────────────────────────────────────────┤
│ 1. Tauri build creates MSI/NSIS +        │
│    `.sig` signature file (via `tauri     │
│    build` with TAURI_PRIVATE_KEY env)    │
│                                          │
│ 2. `.sig` is the Ed25519 signature of    │
│    the bundle archive                    │
│                                          │
│ 3. Release manifest (`latest.json` or    │
│    `update.json`) is also signed with    │
│    the same key                          │
│                                          │
│ 4. Artifacts + manifest pushed to        │
│    release server / GitHub Releases       │
└──────────────────────────────────────────┘
         ↓
┌──────────────────────────────────────────┐
│      Client-side (ShiroScout app)         │
├──────────────────────────────────────────┤
│ 1. Tauri updater fetches manifest from    │
│    endpoint URL                          │
│                                          │
│ 2. Verifies manifest signature with      │
│    embedded public key                   │
│                                          │
│ 3. Downloads new version archive         │
│                                          │
│ 4. Verifies archive signature (.sig)     │
│    with embedded public key              │
│                                          │
│ 5. FAIL-CLOSED: if signature invalid,    │
│    update dialog shows error,            │
│    refuses to install                    │
│                                          │
│ 6. On success: extract and apply update  │
└──────────────────────────────────────────┘
```

**Key generation command:**
```bash
# Generate keypair (run offline on secure machine)
tauri signer generate -w ~/.tauri/shiroscout-update.key

# Output:
# Private key: ~/.tauri/shiroscout-update.key
# Public key:  (printed to stdout — copy to tauri.conf.json)
```

**Build with signing (CI):**
```bash
# TAURI_PRIVATE_KEY and TAURI_KEY_PASSWORD set as CI secrets
TAURI_PRIVATE_KEY="${{ secrets.TAURI_UPDATE_PRIVATE_KEY }}" \
TAURI_KEY_PASSWORD="${{ secrets.TAURI_KEY_PASSWORD }}" \
pnpm tauri build --bundles msi,nsis
```

#### Release Server Requirements

The release server must serve the following structure per channel:

```
https://releases.shiroscout.app/
├── stable/
│   ├── x86_64-pc-windows-msvc/
│   │   ├── 0.1.0/
│   │   │   ├── ShiroScout_0.1.0_x64_en-US.msi
│   │   │   ├── ShiroScout_0.1.0_x64-setup.nsis.zip
│   │   │   ├── ShiroScout_0.1.0_x64_en-US.msi.sig
│   │   │   ├── ShiroScout_0.1.0_x64-setup.nsis.zip.sig
│   │   │   └── ShiroScout_0.1.0_x64_en-US.msi.update-metadata.json
│   │   └── latest.json          # Points to the latest version
│   └── aarch64-pc-windows-msvc/
│       └── ... (ARM64 mirror)
├── beta/
│   └── ... (same structure)
└── nightly/
    └── ... (same structure, auto-published by CI)
```

**`latest.json` manifest format (per channel+target):**

```json
{
  "version": "0.2.0",
  "pub_date": "2026-07-15T12:00:00Z",
  "url": "https://releases.shiroscout.app/stable/x86_64-pc-windows-msvc/0.2.0/ShiroScout_0.2.0_x64_en-US.msi",
  "signature": "base64-encoded-ed25519-signature",
  "notes": "Bug fixes and performance improvements"
}
```

**Server requirements:**
- Static file serving (any HTTPS-capable CDN or S3 bucket)
- CORS headers: `Access-Control-Allow-Origin: *`
- Caching: Set `Cache-Control: no-cache` on `latest.json` (or short TTL)
- Object size limit: ≥ 500 MB per artifact (MSI files can be large)
- HTTPS enforced with TLS 1.2+ (Tauri updater requires HTTPS)
- Optional: Signed manifest per channel (additional integrity layer)

**Hosting options:**

| Option | Pros | Cons | Recommended use |
|--------|------|------|:---------------:|
| GitHub Releases | Free, CI-native, built-in API | No per-channel routing, no ARM64 folder structure | Nightly builds |
| AWS S3 + CloudFront | Full folder control, cheap, global CDN | Manual bucket setup, IAM permissions | Primary (stable + beta) |
| Azure Blob + CDN | Azure-native, OIDC-friendly | Slightly higher egress cost | If Azure-first infrastructure |
| GitHub Pages | Free, simple | 1 GB limit, no server-side routing | Prototype/staging only |

**Recommendation:** GitHub Releases for nightly, S3+CloudFront for stable/beta. Use a small CI step to mirror nightly → S3 for unified endpoint routing.

#### D6 Compliance Checklist

- [ ] Offline Ed25519 keypair generated and stored in secure vault
- [ ] Public key embedded in `tauri.conf.json`
- [ ] `tauri-plugin-updater` added to `Cargo.toml` (`tauri-plugin-updater = "2"`)
- [ ] Release endpoints point to real HTTPS URLs per channel
- [ ] Update manifest signed with the private key
- [ ] Signature verification is fail-closed (invalid sig → refusal to install)
- [ ] `dialog: true` or custom dialog implemented
- [ ] CI pipeline sets `TAURI_PRIVATE_KEY` and `TAURI_KEY_PASSWORD` from secrets

#### Key Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Private key compromise | Key generation on air-gapped machine; hardware-backed storage; rotate key and re-sign manifests if compromised |
| Release server down | Endpoints array can have fallback URLs; Tauri updater tries endpoints in order |
| Update download interrupted | Tauri updater resumes downloads (built-in); user can retry from dialog |
| Channel mismatch (downgrade) | App version is SemVer-compared; Tauri rejects downgrades unless explicitly configured |
| Missing `tauri-plugin-updater` in Cargo.toml | Add `tauri-plugin-updater = "2"` to `[dependencies]` — build error if absent |

#### Dependencies

| Dependency | Type | Required by |
|------------|:----:|:-----------:|
| 8.2 — Code signing pipeline | Build-time | Binary signing before release |
| `tauri-plugin-updater = "2"` | Rust crate | Tauri updater plugin |
| `@tauri-apps/api` (updater module) | npm package | Frontend update check/install |
| Ed25519 keypair | Secret | Signature generation & verification |
| Release server (S3/CDN) | Infrastructure | Hosting release artifacts + manifests |
| CI secrets: `TAURI_PRIVATE_KEY`, `TAURI_KEY_PASSWORD` | CI config | Signing in build pipeline |

---

## 6.3 ARM64 Target Build + Smoke Test (Item 8.4)

**Purpose:** Enable cross-compilation of ShiroScout for Windows 11 ARM64 (`aarch64-pc-windows-msvc`), produce signed ARM64 MSI installers, and verify basic functionality via smoke test.

### Target Specification

| Field | Value |
|-------|-------|
| **Rust target triple** | `aarch64-pc-windows-msvc` |
| **Tauri bundler target** | `--target aarch64-pc-windows-msvc` |
| **MSI architecture** | ARM64 (WiX `-arch arm64`) |
| **NSIS architecture** | ARM64 |

### Toolchain Requirements

#### Rust target installation
```bash
rustup target add aarch64-pc-windows-msvc
```

#### LLVM/Clang for cross-compilation

When building from an x64 Windows host (Ring 2), the standard MSVC toolchain includes ARM64 cross-compilers via Visual Studio 2022 Build Tools with the "MSVC v143 - VS 2022 C++ ARM64 build tools" workload.

When building from Linux (Ring 1, cross-compilation), the following additional tooling is required:
- **LLVM/Clang** (v16+): `apt install llvm-dev libclang-dev` — provides ARM64 code generation
- **Windows SDK** (headers/libs): Available via `xwin` crate or mounted from a Windows SDK installation
- **LLD linker**: LLVM's ARM64-capable linker for cross-target

#### WiX Toolset for MSI

- **Required for MSI bundling** on both x64 and ARM64
- WiX v3.x or v4.x must be installed on the build machine
- For ARM64 MSI, WiX needs the ARM64 architecture module: `wix extension add WixToolset.UI.wixext` (v4) or the `/arch arm64` flag (v3)
- Install on Windows: `dotnet tool install --global wix` (WiX v4+)
- On Linux cross-compilation: WiX runs only on Windows — MSI building must happen on Windows runners (Ring 2)

### Tauri Bundler Configuration

Current `tauri.conf.json` targets `["msi", "nsis"]` which are installer formats, not CPU architectures. The architecture is determined by the Rust target triple passed to `pnpm tauri build`.

To build for ARM64:

```bash
# On Windows x64 (cross-compile)
pnpm tauri build --target aarch64-pc-windows-msvc --bundles msi

# On Windows ARM64 (native)
pnpm tauri build --bundles msi
```

**Future `tauri.conf.json` update** (when Tauri 2 supports multi-arch builds in one invocation):
```json
{
  "bundle": {
    "targets": ["msi"],
    "windows": {
      "wix": {
        "language": "en-US",
        "arch": ["x64", "arm64"]
      }
    }
  }
}
```
> **Note:** As of Tauri 2, multi-arch builds require separate invocations per target. CI must run two jobs (x64 + ARM64) in parallel.

### CI Pipeline Design

```yaml
jobs:
  build-x64:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          target: x86_64-pc-windows-msvc
      - run: pnpm install
      - run: pnpm tauri build --bundles msi,nsis
      - uses: actions/upload-artifact@v4
        with:
          name: shiroscout-x64
          path: src-tauri/target/release/bundle/

  build-arm64:
    runs-on: windows-latest  # x64 host cross-compiling to ARM64
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          target: aarch64-pc-windows-msvc
      - run: pnpm install
      - run: pnpm tauri build --target aarch64-pc-windows-msvc --bundles msi
      - uses: actions/upload-artifact@v4
        with:
          name: shiroscout-arm64
          path: src-tauri/target/aarch64-pc-windows-msvc/release/bundle/

  smoke-test-arm64:
    if: github.event_name == 'pull_request' || github.ref == 'refs/heads/main'
    runs-on: [self-hosted, Windows, ARM64]  # Requires ARM64 runner
    needs: build-arm64
    steps:
      - uses: actions/download-artifact@v4
        with:
          name: shiroscout-arm64
      - name: Smoke test — silent install
        run: msiexec /i *.msi /qn /norestart
      - name: Verify install
        run: |
          $installed = Get-WmiObject Win32_Product | Where-Object { $_.Name -like "*Shiro Scout*" }
          if (-not $installed) { throw "ARM64 MSI install failed" }
          Write-Output "ARM64 smoke test PASSED: $($installed.Name) v$($installed.Version)"
      - name: Uninstall
        run: msiexec /x *.msi /qn /norestart
```

### Known Caveats & Mitigations

| Caveat | Impact | Mitigation |
|--------|--------|------------|
| **WebView2 on ARM64** | Windows 11 ARM64 ships with native ARM64 WebView2 pre-installed (22H2+). On older builds, x86 emulated WebView2 is available. | Target Windows 11 22H2+ minimum. The Tauri 2 app will use the system WebView2 regardless. |
| **x64 emulation vs native ARM64** | x64 emulated on ARM64 runs at ~50-80% native perf. ARM64 native is ~2x faster. | Ship ARM64 native binaries. Do not ship x64 binaries as ARM64 (use correct target). |
| **WiX ARM64 support** | WiX v3 has limited ARM64 support; WiX v4 has full ARM64 MSI building. | Pin to WiX v4+ for ARM64 builds. Ensure `wix extension add` includes ARM64 modules. |
| **Code signing** | Authenticode signing applies to ARM64 MSI identically to x64. | Same signing pipeline (D4) — no architecture-specific changes needed. |
| **LLVM/Clang on Linux** | Cross-compiling `aarch64-pc-windows-msvc` from Linux requires Windows SDK headers. | Use `xwin` crate to download/setup Windows SDK, or build on Windows runners exclusively. |
| **Updater manifests** | ARM64 and x64 need separate updater channels (`{{target}}` in endpoint URL). | Already supported by the updater design (8.3) via `{{target}}` templating. |
| **CI runner availability** | GitHub-hosted ARM64 runners are not yet GA; self-hosted ARM64 runners required. | Use x64 Windows runners with cross-compilation. Self-hosted ARM64 runners for smoke tests. |

### Cross-compilation from Linux (Ring 1) — Experimental

Building ARM64 from the Linux container is possible but not recommended for production until all toolchain issues are resolved:

```bash
# PREREQUISITE: rustup target add aarch64-pc-windows-msvc
# PREREQUISITE: apt install llvm-dev libclang-dev
# PREREQUISITE: cargo install xwin
# PREREQUISITE: xwin --accept-license splat --output /opt/windows-sdk

# Build with custom SDK
CARGO_TARGET_AARCH64_PC_WINDOWS_MSVC_LINKER=/usr/bin/llvm-lld \
CC_aarch64_pc_windows_msvc=/usr/bin/clang \
CXX_aarch64_pc_windows_msvc=/usr/bin/clang++ \
WINAPI_NO_BUNDLED_LIBRARIES=1 \
XWIN_ARCH=aarch64 \
pnpm tauri build --target aarch64-pc-windows-msvc --bundles msi
```

**Blockers for Ring 1 cross-compile:**
1. WiX Toolset is Windows-only — MSI bundling requires Ring 2 or a Windows runner
2. Tauri bundler calls native Windows APIs during MSI construction
3. `xwin` SDK is not yet fully compatible with all Tauri dependencies

**Recommendation:** Build ARM64 on Windows x64 CI runners (Ring 2) with cross-compilation. Use Ring 1 (Linux container) for Rust static analysis only (`cargo check --target aarch64-pc-windows-msvc`).

### Dependencies

| Dependency | Type | Required by |
|------------|:----:|:-----------:|
| 8.1 — MSI/NSIS packaging | Build-time | ARM64 MSI bundling |
| Rust target `aarch64-pc-windows-msvc` | Toolchain | Cross-compilation |
| WiX Toolset v4+ | Tool | ARM64 MSI creation |
| Visual Studio Build Tools (ARM64 workload) | Tool | MSVC ARM64 CRT/link |
| Self-hosted ARM64 runner (optional) | Infrastructure | Native builds + smoke tests |
| `xwin` (optional, Linux cross-compile) | Tool | Windows SDK on non-Windows |

### D4/D5 Compliance Checklist

- [ ] `rustup target add aarch64-pc-windows-msvc` — verified in CI
- [ ] ARM64 MSI builds and installs silently (`msiexec /i app.msi /qn`)
- [ ] ARM64 binary is Authenticode-signed (same pipeline as x64)
- [ ] Updater manifest includes `aarch64-pc-windows-msvc` entries
- [ ] CI has at least cross-compilation pipeline; native ARM64 runner tracked as future work
- [ ] Release checklist includes ARM64 verification (clean Windows 11 ARM64 VM)

### 6.4 Ring 2 (Windows) Authoritative Full-Gate Release Run (Item 8.5)

**Purpose:** Authoritative Ring 2 release run procedure — executed on a clean Windows 11 x64 VM, running every gate G0–G5, then Tauri build, MSI installation, smoke test, and code signing. Final quality gate before any release ships.

**Owner:** QA + DevOps  ·  **Ring:** 2 (Windows 11 host)  ·  **Deps:** 8.1, 8.2, 8.3, 8.4

| Item | Task | Owner | Deps | Status |
|------|------|-------|------|:------:|
| 8.5 | Ring 2 full-gate release run procedure | QA + DevOps | 8.1 | 🟡 |

#### Prerequisites — Windows 11 Host

| Category | Tool | Verify | Notes |
|----------|------|--------|-------|
| Rust | `rustup` + MSVC toolchain | `rustc --version && cargo --version` | Select x86_64-pc-windows-msvc default |
| ARM64 target | Rust cross-compile | `rustup target list --installed` | `rustup target add aarch64-pc-windows-msvc` |
| Node.js | v20 LTS+ | `node --version` | nodejs.org |
| pnpm | v9+ | `pnpm --version` | `corepack enable && corepack prepare pnpm@latest --activate` |
| WiX Toolset | v4+ | `wix --version` | `dotnet tool install --global wix` |
| Windows SDK | Latest (signtool) | `signtool sign /?` | Via VS Build Tools 2022 |
| cargo-deny | Latest | `cargo deny --version` | `cargo install cargo-deny` |
| Docker Desktop | Latest, WSL2 | `docker --version` | For smoke test |
| Clean VM | Win 11 x64 22H2+ | `winver` | No pre-existing dev tools |

#### Prerequisites Checklist

- [ ] Clean Windows 11 x64 VM provisioned
- [ ] Internet connectivity (GitHub, crates.io, npm)
- [ ] All tools installed per table
- [ ] Docker Desktop running with WSL2
- [ ] Tagged repo clone: `git clone --branch v<version> <repo>`
- [ ] Release commit verified: `git log -1 --oneline`
- [ ] Code signing certificate available (Azure Trusted Signing or PKCS#12)
- [ ] Updater signing key ready (`TAURI_PRIVATE_KEY`, `TAURI_KEY_PASSWORD`)

#### Step-by-Step Procedure

**Step a — Git checkout**
```powershell
git clone --branch v<version> <repo-url>
cd shiroscout
git log -1 --oneline
git tag -l --points-at HEAD
```

**Step b — G0 stub scan**
```powershell
$stubs = Select-String -Path (git diff --name-only HEAD~1) `
  -Pattern "todo!\(\)|unimplemented!\(\)|not.?implemented|\bTODO\b|\bFIXME\b|\bHACK\b" `
  -CaseSensitive:$false
$stubs.Count  # Expect 0 outside tests/ + docs/
```
Gate: zero new stub matches.

**Step c — G1 TypeScript type check**
```powershell
npx tsc --noEmit
if ($LASTEXITCODE -ne 0) { throw "G1 failed" }
```

**Step d — G2 frontend build**
```powershell
pnpm build
if ($LASTEXITCODE -ne 0) { throw "G2 failed" }
```

**Step e — G3 Rust check (x64 + ARM64)**
```powershell
cargo check --manifest-path src-tauri/Cargo.toml
if ($LASTEXITCODE -ne 0) { throw "G3 x64 failed" }
cargo check --manifest-path src-tauri/Cargo.toml --target aarch64-pc-windows-msvc
if ($LASTEXITCODE -ne 0) { throw "G3 ARM64 failed" }
```

**Step f — G4 clippy (x64 + ARM64)**
```powershell
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
if ($LASTEXITCODE -ne 0) { throw "G4 x64 failed" }
cargo clippy --manifest-path src-tauri/Cargo.toml --target aarch64-pc-windows-msvc -- -D warnings
if ($LASTEXITCODE -ne 0) { throw "G4 ARM64 failed" }
```

**Step g — G4.5 supply chain audit**
```powershell
cargo deny --manifest-path src-tauri/Cargo.toml check
if ($LASTEXITCODE -ne 0) { throw "G4.5 deny failed" }
pnpm audit --audit-level=high
if ($LASTEXITCODE -ne 0) { throw "G4.5 audit failed" }
```

**Step h — G5 tests**
```powershell
cargo test --manifest-path src-tauri/Cargo.toml
if ($LASTEXITCODE -ne 0) { throw "G5 failed" }
```
Record pass/fail/ignored counts.

**Step i — Tauri build (MSI + NSIS x64)**
```powershell
$env:TAURI_PRIVATE_KEY = "<secret>"
$env:TAURI_KEY_PASSWORD = "<secret>"
pnpm tauri build --bundles msi,nsis
if ($LASTEXITCODE -ne 0) { throw "Tauri build failed" }
```
Verify `src-tauri/target/release/bundle/msi/` artifacts exist.

**Step j — Install MSI**
```powershell
msiexec /i src-tauri/target/release/bundle/msi/ShiroScout_*.msi /qn /norestart
if ($LASTEXITCODE -ne 0) { throw "MSI install failed" }
$app = Get-WmiObject Win32_Product | Where-Object { $_.Name -like "*Shiro Scout*" }
if (-not $app) { throw "MSI install not detected" }
```

**Step k — Smoke test**
```powershell
$proc = Start-Process -FilePath "$env:LOCALAPPDATA\Programs\ShiroScout\ShiroScout.exe" -PassThru
Start-Sleep -Seconds 5
if (-not (Get-Process -Id $proc.Id -ErrorAction SilentlyContinue)) { throw "Launch failed" }
docker ps  # Verify Docker connection
Stop-Process -Id $proc.Id -Force
```
Criteria: launch → no crash → window visible (process check) → Docker responds → clean close.

**Step l — Sign artifacts**
```powershell
# Internal binaries
& "C:/Program Files (x86)/Windows Kits/10/bin/10.0.20348.0/x64/signtool.exe" sign `
  /fd SHA256 /f $certPath /p $certPass `
  /tr http://timestamp.digicert.com /td SHA256 /v (Get-ChildItem src-tauri/target/release -Include *.exe,*.dll -Recurse)

# MSI wrapper
& "C:/Program Files (x86)/Windows Kits/10/bin/10.0.20348.0/x64/signtool.exe" sign `
  /fd SHA256 /f $certPath /p $certPass `
  /tr http://timestamp.digicert.com /td SHA256 /v src-tauri/target/release/bundle/msi/ShiroScout_*.msi

# Verify all
signtool verify /pa /v <each artifact>
```

#### Ring 2 Report Template

```
## RING 2 RELEASE RUN REPORT
Release version: <vX.Y.Z>
Build commit: <SHA>
Run date: <YYYY-MM-DD>
Run by: <name>
Windows build: <winver>
Environment: clean VM | dev machine

### Prerequisites
- Clean VM: YES/NO
- All tools installed: YES/NO
- Tag checkout v<ver>: YES/NO
- Docker running: YES/NO
- Signing cert ready: YES/NO
- Updater key ready: YES/NO

### Gates
| Gate | Command | Exit | Status |
|:----:|---------|:----:|:------:|
| G0 | Stub scan | <n> | PASS/FAIL |
| G1 | tsc --noEmit | <code> | PASS/FAIL |
| G2 | pnpm build | <code> | PASS/FAIL |
| G3-x64 | cargo check x64 | <code> | PASS/FAIL |
| G3-arm64 | cargo check ARM64 | <code> | PASS/FAIL |
| G4-x64 | clippy x64 | <code> | PASS/FAIL |
| G4-arm64 | clippy ARM64 | <code> | PASS/FAIL |
| G4.5-deny | cargo deny | <code> | PASS/FAIL |
| G4.5-audit | pnpm audit | <code> | PASS/FAIL |
| G5 | cargo test | <code> | PASS/FAIL |

### Build
| Step | Status |
|------|:------:|
| Tauri MSI | PASS/FAIL |
| Tauri NSIS | PASS/FAIL |
| MSI install | PASS/FAIL |
| App launch | PASS/FAIL |
| Docker conn | PASS/FAIL |
| Code sign | PASS/FAIL |
| Sig verify | PASS/FAIL |

### Test Results
- Passed: <n> / Failed: <n> / Ignored: <n>

### Warnings Delta
- 0 new | <list>

### Artifact Hashes
<file> SHA256: <hash>

### Blockers
- none | BLK-8.5.1: <description>

### Sign-off
- QA Engineer: PASS/FAIL
- DevOps Engineer: PASS/FAIL
```

#### Verification Checklist (all or nothing — DONE-001)

- [ ] G0–G5 all pass
- [ ] x64 + ARM64 cargo check + clippy both 0
- [ ] MSI installs and uninstalls cleanly
- [ ] Smoke: launch, UI visible, Docker connection, no crash
- [ ] All artifacts signed (binaries, MSI, NSIS)
- [ ] Signatures verified via signtool verify /pa
- [ ] SHA256 hashes recorded per artifact
- [ ] Report template fully filled

#### Key Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Clean VM diverges from CI env | Maintain standardized VM image with setup doc |
| Docker not running on VM | Include `docker ps` in prerequisites; document fix |
| Code signing cert unavailable | Pre-stage to cert store; document manual Azure Trusted Signing fallback |
| ARM64 target check fails | Cross-compile via x64 host only; ARM64 native test requires separate runner |
| MSI install fails silently | Log `msiexec` exit code; verify via WMI query |
| Smoke test on headless CI | Use WinSession or manual fallback |

#### Dependencies

| Dependency | Type | Required by |
|------------|:----:|:-----------:|
| 8.1 — MSI/NSIS packaging | Build-time | Tauri bundler config |
| 8.2 — Code signing pipeline | Build-time | Signing step |
| 8.3 — Tauri updater | Build-time | TAURI_PRIVATE_KEY for updater sig |
| 8.4 — ARM64 target | Build-time | ARM64 gate steps |
| Clean Windows 11 x64 VM | Infrastructure | Full-gate environment |
| Docker Desktop | Runtime | Smoke test |

---

## 7. Housekeeping backlog (schedule into any batch with spare slots)

| Item | Task | Owner |
|------|------|-------|
| H.1 | De-duplicate ADR numbering (two ADR-001s, ADR-002s, etc. in docs/adr/) — renumber files, keep content, add index | Doc Engineer | ✅ |
| H.2 | Convert all CRLF markdown to LF after 1.B lands | Architect |
| H.3 | GLOSSARY.md: add Batch Loop, STOP/ASK, ROUTE line, WIP limit terms | Doc Engineer |

---

*Maintained by the Orchestrator. Full-file rewrites only.*
