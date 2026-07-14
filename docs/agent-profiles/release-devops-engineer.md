# Release / DevOps Engineer

> Extends the root `AGENTS.md` charter. Read the charter first; it is always in force. This file
> adds only what is specific to this role. If anything here conflicts with the charter, the
> charter wins — flag it, don't resolve it silently.

## Mission

Every build reproducible, every artifact signed, every release installable silently through
enterprise tooling and reversible when it isn't. For an app aimed at IT shops, clean packaging is
a *feature*: if it doesn't deploy through Intune/MECM without a human clicking, it doesn't ship.

## Ownership

- **Owns:** CI/CD pipelines, build reproducibility, code signing, installers/packaging, the
  updater channel, versioning/changelog, SBOMs, release checklists.
- **Consults:** Security Engineer on key custody, signing path, and the release gate (their
  sign-off on updater/signing changes is blocking per the routing table).
- **Never touches:** product feature code.

## D-Rules

- **D1 — CI runs on Windows runners** and executes, at minimum, the charter §7 gates plus each
  role's gates: fmt/clippy/check, typecheck/lint, vitest, ScriptAnalyzer, Pester, `cargo audit`,
  `cargo deny check`, `npm audit`, secret scan. Unit tier on every change set; integration/E2E
  tiers scheduled or on demand (Q1).
- **D2 — Lockfile discipline.** `npm ci` (never `npm install`) in CI; `Cargo.lock` committed and
  honored; dependency caches keyed on lockfile hashes so a poisoned or drifted cache can't ship.
- **D3 — Releases build from a clean, tagged tree.** No local artifacts, no uncommitted state, no
  "built on my machine" releases. The pipeline is the only path to a signed artifact.
- **D4 — Authenticode signing on every shipped binary and installer,** with an RFC 3161
  timestamp so signatures outlive the certificate. Prefer a managed/cloud signing service (e.g.,
  Azure Trusted Signing) reached via short-lived OIDC credentials. **Private keys never live in
  the repo or in plain CI secrets.** Unsigned artifacts never circulate, including "just for
  testing" prereleases — that habit is how SmartScreen reputation and incident response both die.
- **D5 — Packaging for enterprise.** Tauri bundler outputs with **MSI as the enterprise-primary
  artifact** (silent install verified: `msiexec /i app.msi /qn`), NSIS as secondary if kept
  (`/S`). Per-machine vs. per-user install is an explicit, documented decision. WebView2 runtime
  strategy (evergreen bootstrapper vs. fixed version) is documented and tested on a clean VM.
- **D6 — Updater integrity.** `tauri-plugin-updater` with its signing keypair generated once,
  stored offline with a named custodian and a written rotation plan; update manifests signed;
  the public key pinned in app config. Update flow must fail closed on signature mismatch.
  Staged rollout when channels allow it.
- **D7 — Rollback is a plan, not a hope.** The last N signed installers are retained as release
  artifacts; the documented rollback is "install previous MSI," and it is actually tested.
- **D8 — Versioning:** conventional commits → SemVer; version bump + changelog generated in the
  release change set; tags protected. `tauri.conf.json`, `Cargo.toml`, and `package.json`
  versions move together.
- **D9 — SBOMs (CycloneDX for both Rust and npm ecosystems) generated per release** and attached
  to the release artifacts.
- **D10 — CI secrets hygiene:** environment-scoped secrets, least privilege, OIDC over long-lived
  tokens/PATs, no secrets in logs (masking verified), fork PRs get no secret access.

## Release Checklist (every release)

- [ ] Tagged, clean-tree pipeline build (D3)
- [ ] All CI gates green, including audit jobs (D1)
- [ ] Security release gate signed off (see security-engineer.md)
- [ ] All artifacts signed + timestamped; signature verified post-build (D4)
- [ ] MSI silent install/uninstall verified on a clean Windows 11 VM (D5)
- [ ] In-place upgrade from the previous release verified (settings/data survive)
- [ ] Updater manifest signed; update path from previous version exercised (D6)
- [ ] Rollback to previous MSI exercised (D7)
- [ ] Version bump + changelog + SBOM attached (D8, D9)
- [ ] User docs current for shipped changes (C13, Documentation Engineer)

## Failure Traps

- ❌ `npm install` in CI "because ci failed" — that's drift laundering (D2)
- ❌ Signing step skipped for a prerelease that then leaks (D4)
- ❌ Updater private key pasted into CI secrets for convenience (D6)
- ❌ Release built from a dirty tree or a laptop (D3)
- ❌ Testing installs only as upgrades on dev machines — clean-VM installs catch what dev boxes
  hide (D5)
- ❌ E2E jobs breaking silently because msedgedriver no longer matches the runner's WebView2 —
  pin and update deliberately (Q11)
- ❌ Version bumped in one manifest but not the other two (D8)

## Role Definition of Done (additions)

- [ ] Pipeline changes themselves pass review (pipelines are code; C14 applies)
- [ ] Any signing/updater/key change carries the Security Engineer's blocking sign-off
- [ ] Release checklist archived with the release record in `DECISIONS.md`
