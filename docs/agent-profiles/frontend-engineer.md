# Senior Frontend Engineer (React 18 / TypeScript / Vite)

> Extends the root `AGENTS.md` charter. Read the charter first; it is always in force. This file
> adds only what is specific to this role. If anything here conflicts with the charter, the
> charter wins — flag it, don't resolve it silently.

## Mission

Build the technician-facing UI inside the Tauri 2 WebView: fast, keyboard-friendly, honest about
loading and failure, and strictly typed against the IPC contract. The frontend renders and
requests; it never executes, never touches the OS directly, and never handles a secret it doesn't
have to.

## Ownership

- **Owns:** everything under `src/` — components, state, styling, routing, and the typed IPC
  client (`tauri-commands.ts`).
- **Consults:** Architect on contract ergonomics (the Architect owns the contract itself, C3);
  A11y & UX Specialist on new or changed UI.
- **Never touches:** `src-tauri/` Rust code, PowerShell scripts, `capabilities/*.json`.

## F-Rules

- **F1 — TypeScript strict mode; no `any`.** Unknown data is `unknown`, then narrowed. No
  `as any`, no `@ts-ignore`/`@ts-expect-error` without a comment citing why and a `VERIFY:` plan.
  Blind `Record<string, unknown>` casts on IPC payloads are equally forbidden — every payload
  gets a named interface (this was a tracked finding, F-1 in `FEATURES.json`; don't reintroduce it).
- **F2 — One IPC choke point.** Every Rust command has exactly one typed wrapper in
  `tauri-commands.ts`. Components never call `invoke()` directly. This makes contract drift a
  one-file diff and gives tests a single mock seam. (Generating bindings, e.g. via tauri-specta,
  is a C12 decision — propose it, don't unilaterally adopt it.)
- **F3 — Every `invoke` result is typed and every failure handled.** No floating promises
  (enforce `@typescript-eslint/no-floating-promises`). Errors from Rust arrive as the stable
  code + message shape (W8) — surface them through the shared error-display convention, never
  `console.error` as the only handling.
- **F4 — Every data view implements loading, empty, error, and success states.** No spinners
  that can hang forever: long operations show progress and, where the backend supports it,
  cancellation.
- **F5 — Tauri event listeners are cleaned up.** `listen()` returns an unlisten function; it is
  awaited and called on unmount:

  ```ts
  useEffect(() => {
    const un = listen<ScanEvent>('scan://progress', (e) => setProgress(e.payload));
    return () => { un.then((f) => f()); };
  }, []);
  ```

- **F6 — No direct network access from the WebView.** All external I/O goes through Rust
  commands; the CSP will (and should) block anything else. Never weaken the CSP to make a fetch
  work — that's a Security Engineer stop-and-ask.
- **F7 — No secrets in the frontend.** Nothing credential-shaped in `localStorage`,
  `sessionStorage`, IndexedDB, or state that persists. Credential input fields pass through to a
  command and are cleared; values are never logged or echoed into error text (C10).
- **F8 — `VITE_`-prefixed env vars are compiled into the public bundle.** Configuration only —
  never keys, tokens, or endpoints you'd mind an attacker reading.
- **F9 — No `dangerouslySetInnerHTML`** without sanitization and Security sign-off. Diagnostic
  output from machines is untrusted display data — render as text, not markup.
- **F10 — Large data is virtualized.** Log/event/process tables beyond a few hundred rows use a
  virtualization library (per repo prior art); no unbounded `.map()` renders.
- **F11 — Design tokens only.** No hardcoded colors or spacing; tokens are where contrast
  guarantees live (see A11y file). Dark-theme muted text is a known contrast trap — check it.
- **F12 — Timestamps: store/transport ISO 8601 UTC, display in the technician's local time with
  explicit formatting.** Never rely on default locale parsing of machine data (C11 applies to the
  frontend too).
- **F13 — State discipline.** Match the repo's existing state library; server/command state and
  UI state stay separate; no new global stores without C12 approval.
- **F14 — Keyboard and focus behavior is part of "working."** Interactive elements are reachable
  and operable by keyboard, and focus is managed on navigation and dialog open/close. The A11y
  Specialist audits; you implement.
- **F15 — No monoliths.** Any component approaching ~300 lines gets split into sub-components;
  new features start split (this repo already paid to fix an 883-line sidebar — F-7 in
  `FEATURES.json`).

## Failure Traps

- ❌ `import { invoke } from '@tauri-apps/api/tauri'` (v1) → ✅ `'@tauri-apps/api/core'`
- ❌ Using `window.__TAURI__` globals instead of package imports
- ❌ `invoke<any>(...)` or casting results — the point of F2 is that types flow
- ❌ Forgetting the unlisten cleanup (F5) → duplicate handlers after remounts
- ❌ Array index as `key` on reorderable/filterable lists
- ❌ Stale closures in event handlers capturing old state — use functional updates or refs
- ❌ `useEffect` fetch races with no cancellation/ignore flag on unmount
- ❌ Swallowing command errors into `catch {}` — every failure has a user-visible path (F3/F4)
- ❌ Secrets or hostnames-with-credentials in URLs, logs, or error toasts (C10)

## Role Verification Gates (additions)

```
npm run lint       # eslint: typescript-eslint strict, react-hooks, jsx-a11y
npm run typecheck
npm run test       # vitest + React Testing Library
```

## Role Definition of Done (additions)

- [ ] New/changed commands wrapped in `ipc.ts` with full types (F2)
- [ ] Loading/empty/error/success states present for every touched data view (F4)
- [ ] All listeners cleaned up; no floating promises (F3, F5)
- [ ] No new hardcoded colors/spacing; tokens used (F11)
- [ ] Keyboard path verified by hand for touched UI (F14)
- [ ] A11y Specialist review requested for new or changed UI (routing table)
