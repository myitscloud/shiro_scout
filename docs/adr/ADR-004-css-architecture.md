# ADR-004: CSS Architecture

**Status:** Accepted
**Date:** 2026-07-07
**Deciders:** Frontend Engineer, Tech Lead

## Context
Project Aegis needs a CSS architecture that supports theming, component isolation, and a clean, minimal UI inspired by Agent Zero but simpler. The architecture must be easy to maintain by a small team and work seamlessly with Vite and React 18.

## Decision Drivers
- Must support theming (light/dark mode) with minimal overhead
- Must avoid global style conflicts in a component-based React architecture
- Must keep build complexity low — no additional toolchain beyond Vite
- Must be easy for new contributors to understand
- CSS file size should remain small for fast page loads

## Considered Options
- **Option A:** Tailwind CSS — popular utility-first approach, but adds build step, JIT compilation overhead, large stylesheet despite purging, and makes design tokens implicit in class strings
- **Option B:** CSS-in-JS (styled-components, Emotion) — dynamic theming, but adds runtime overhead (~10KB gzip), complicates SSR, and obscures styles from browser DevTools
- **Option C:** Global design-tokens.css + CSS Modules per component — zero runtime, native CSS, explicit tokens, Vite-native support with .module.css, clear separation of concerns

## Decision
Chosen: **Option C — Global design-tokens.css + CSS Modules per component**

Define all design tokens (colors, spacing, typography, shadows, breakpoints) in a single `src/styles/design-tokens.css` file using CSS custom properties. Each component styles itself using CSS Modules (`ComponentName.module.css`). Theming is achieved by switching a `data-theme` attribute on `<html>`, which overrides the custom property values. No Tailwind, no CSS-in-JS runtime.

### Example structure
```
src/
  styles/
    design-tokens.css      # --color-bg, --spacing-md, --font-base, etc.
    reset.css              # Minimal CSS reset
  components/
    Button/
      Button.tsx
      Button.module.css
    Terminal/
      Terminal.tsx
      Terminal.module.css
```

## Consequences
- Positive: Zero runtime — all styles are static CSS, parsed once by browser
- Positive: Vite-native — no plugins needed beyond built-in CSS Modules support
- Positive: Explicit design tokens — theming is just redeclaring custom properties
- Positive: Component scoping via CSS Modules prevents selector collisions
- Positive: Familiar to any developer who knows CSS — no framework lock-in
- Negative: Dynamic style composition requires classNames library (already a dependency)
- Negative: CSS Modules file per component increases file count slightly
- Negative: No style composition primitives (like Tailwind's @apply or styled-components' inheritance)

## Compliance
- All colors, spacing, and typography values must be defined as CSS custom properties in design-tokens.css — no hardcoded values
- Components must use CSS Modules (.module.css) for all scoped styles
- Global styles (reset, typography base) belong in src/styles/*.css
- Theming changes must only modify custom property values; no class toggling on individual components