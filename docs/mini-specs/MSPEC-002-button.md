# Mini-Spec 002: Button Component

**Task:** Create the Button component with 6 variants, all interactive states, and keyboard accessibility per Neo-Glass Terminus design language.

**Layers touched:** Frontend (React/TypeScript + CSS Modules)

**Owning agent:** Frontend Engineer

## Acceptance Criteria

1. All 6 variants render correctly:
   - **primary** — Purple glass with glow (Send, Confirm actions)
   - **secondary** — Subtle glass with border (Cancel, Back)
   - **ghost** — Transparent, text-only (Toolbar actions, inline links)
   - **danger** — Red glass (Destructive actions: Delete, Kill)
   - **icon** — Square, ghost-style (Toolbar icons)
   - **link** — Text with accent underline (Navigational links)

2. All interactive states working: `default | hover | active | disabled | loading`

3. Loading state shows a spinner icon and disables further clicks

4. Disabled state uses `--text-muted` color, no pointer events

5. Button text uses `--font-ui` (Geist), size 14px standard

6. CSS Module scoped styling — uses design tokens from `design-tokens.css`

7. `aria-label` on icon-only buttons

8. Focus-visible ring uses `--accent-purple` glow

9. Respects `prefers-reduced-motion: reduce` (no glow animation)

10. TypeScript strict typing with props interface: `variant`, `size`, `loading`, `disabled`, `icon`, `children`, `onClick`, `type`, `ariaLabel`

## Out of Scope

- Dropdown variants / split buttons
- Button groups

## Review Triggers Expected

- Code Reviewer
- A11y & UX Specialist
