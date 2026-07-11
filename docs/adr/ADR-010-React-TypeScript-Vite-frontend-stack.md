# ADR-010: React 18 + TypeScript + Vite Frontend Stack

**Date:** 2026-07-08
**Status:** Accepted

## Context

Project Aegis needs a frontend framework for the Tauri 2 WebView supporting:
- Rich component-based UI architecture
- Real-time streaming token rendering
- Complex state management (agent lifecycle, multi-session, multi-agent)
- Type-safe IPC communication with Rust backend
- Small bundle size (target: 600KB JS gzipped)
- Fast development iteration (HMR)

The WebView is OS-native (WebView2 on Windows, WKWebView on macOS, WebKitGTK on Linux) — not a bundled Chromium.

## Decision

**Adopt React 18.3 + TypeScript strict mode + Vite** as the frontend stack. UI libraries: Radix UI primitives (accessible), framer-motion (limited subset), lucide-react (tree-shaken icons).

Key rules:
- TypeScript strict mode; no `any`, no `@ts-ignore`, no blind `Record<string, unknown>` casts
- One IPC choke point: every Rust command has exactly one typed wrapper in `ipc.ts`
- Design tokens only: no hardcoded colors or spacing (F11)
- Component splitting at ~300 lines (F15)
- Every data view implements loading, empty, error, and success states (F4)

## Consequences

**Positive:**
- Mature ecosystem with excellent Tauri 2 integration
- Radix UI provides accessible, composable primitives
- Vite provides fast HMR for development
- TypeScript strict catches contract drift at compile time
- React Testing Library + vitest for testing
- Small bundle size via tree-shaking and code splitting

**Negative:**
- React 18 concurrent mode has learning curve for streaming patterns
- Stale closure risk with event handlers (mitigated via functional updates + refs)
- React runtime overhead vs. lighter alternatives (Svelte, Solid)

## Alternatives Considered

- **Svelte 5** — Lighter bundle, simpler reactivity, but smaller ecosystem and fewer Tauri 2 examples. Rejected for ecosystem maturity.
- **SolidJS** — Excellent performance but smaller community. May revisit for v2.
- **Vue 3** — Good ecosystem but different mental model; React preferred based on AgentKit examples.
