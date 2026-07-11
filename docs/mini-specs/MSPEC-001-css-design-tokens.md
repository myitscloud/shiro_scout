# Mini-Spec 001: CSS Design Token System (Phase 0)

**Task:** Create the CSS Custom Properties token system for Project Aegis's Neo-Glass Terminus design language.

**Layers touched:** Frontend (CSS Modules)

**Owning agent:** Frontend Engineer

## Acceptance Criteria

1. All color tokens from Design Guide §2.1 are defined as CSS custom properties on `:root`
   - Deep bg: `--bg-deep: #0D0D0F`
   - Glass surfaces: `--bg-glass: rgba(26,26,36,.85)`, `--bg-glass-elevated`, `--bg-glass-hover`
   - Glass borders: `--border-glass`, `--border-glass-light`
   - Text colors: `--text-primary: #E4E4E7`, `--text-secondary`, `--text-muted`, `--text-code`
   - Accent: `--accent-purple: #8B5CF6`, `--accent-purple-soft`, `--accent-purple-glow`
   - Status colors: `--status-online`, `--status-thinking`, `--status-warning`, `--status-error`, `--status-neutral`, `--status-human-wait`

2. Typography tokens from Design Guide §2.2
   - `--font-ui: 'Geist', -apple-system, sans-serif`
   - `--font-head: 'Instrument Sans', 'Geist', sans-serif`
   - `--font-mono: 'JetBrains Mono', monospace`

3. Glass effect tokens from Design Guide §2.5
   - `--blur-base: blur(8px)`, `--blur-raised`, `--blur-overlay`, `--blur-tooltip`

4. Elevation tokens
   - `--elevation-base`: 85% opacity, 8px blur
   - `--elevation-raised`: 92% opacity, 12px blur
   - `--elevation-overlay`: 95% opacity, 16px blur
   - `--elevation-tooltip`: 98% opacity, 20px blur

5. Light palette override via `body.light` selector (strips glass effects, white surfaces)

6. Spacing scale from Design Guide §2.4: 4px base unit, tokens for 4-48px

7. Reduced motion: `@media (prefers-reduced-motion: reduce)` disables all animations

8. Fonts loaded via `@font-face` or Google Fonts with `font-display: swap`

## Out of Scope

- Component-specific styles (Button, ChatMessage, etc.) — those are separate mini-specs
- JavaScript/CSS-in-JS — use CSS Modules only

## Review Triggers

- Code Reviewer
- A11y & UX Specialist (verify contrast ratios per Design Guide §7.1)
