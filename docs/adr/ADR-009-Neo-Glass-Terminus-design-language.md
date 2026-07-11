# ADR-009: Neo-Glass Terminus Design Language

**Date:** 2026-07-08
**Status:** Accepted

## Context

Project Aegis needs a distinctive visual identity that differentiates it from existing AI chat interfaces (ChatGPT, Claude, Cursor) while being professional and accessible. It must work as a dark-first desktop application for IT professionals, software engineers, and researchers.

Key constraints:
- Dark mode is the primary identity (light mode as accessibility toggle)
- Must feel like a high-end developer tool, not generic UX
- Cross-platform rendering (WebView2, WKWebView, WebKitGTK)
- Binary size budget under 15MB
- WCAG 2.2 AA compliance

## Decision

**Adopt "Neo-Glass Terminus"** — A fusion of modern glassmorphism, terminal-inspired typography, and cybernetic accent lighting.

Core principles:
- **Dark-first:** `#0D0D0F` deep background, `rgba(26,26,36,0.85)` glass, `backdrop-filter: blur(8px)`
- **Monospace first:** JetBrains Mono for all chat text (not just code blocks)
- **Depth through frost:** Closer panels more opaque, deeper layers more blurred
- **Status as light:** Glowing dots and rings instead of colored badges
- **Electric accents:** Purple-violet (#8B5CF6) for AI activity, selections, key UI
- **Light mode:** Accessibility toggle that strips glass effects

## Consequences

**Positive:**
- Distinctive visual identity vs. competitors
- Developer-native feel with monospace default
- Complete design spec in 715-line Design Guide and 1169-line HTML mockup
- All design tokens, components, and states documented

**Negative:**
- `backdrop-filter` may have inconsistent Linux WebKit support (mitigated via `@supports` fallback)
- Glass effects add rendering cost on low-end hardware
- Light mode requires separate token set
- Custom titlebar adds cross-platform complexity

## Alternatives Considered

- **Standard dark UI (Linear-style)** — Safe but generic. Would not differentiate.
- **Terminal-only aesthetic (Warp-style)** — Too niche; alienates non-terminal users.
- **Material Design 3** — Cross-platform proven but lacks distinctive identity.
