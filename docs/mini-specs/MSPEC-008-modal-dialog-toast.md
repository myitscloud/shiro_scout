# Mini-Spec 008: Modal, Dialog & Toast Components

**Task:** Create the Modal & Dialog components (glass elevated overlay with focus trap) and Toast & Notification Banner (stacked notification system) for the overlay stack.

**Layers touched:** Frontend (React/TypeScript + CSS Modules)

**Owning agent:** Frontend Engineer

## Acceptance Criteria

### Modal & Dialog
1. Glass elevated surface (`--elevation-overlay`: 95% opacity, blur(16px)) with backdrop blur

2. Backdrop: semi-transparent dark overlay (`rgba(0,0,0,0.6)`) that closes modal on click

3. Title bar, content area, and action button footer

4. **Escape to close** — pressing Escape dismisses the modal

5. **Click-outside to close** — clicking the backdrop closes the modal

6. **Focus trap** — Tab/Shift+Tab cycles within modal elements; focus does not escape to background

7. Initial focus on first interactive element inside modal

8. Restore focus to the triggering element on close

9. `role="dialog"` + `aria-modal="true"` + `aria-labelledby` pointing to title

10. Animated open: fade in backdrop + scale up glass surface (`--ease-glass` 400ms)

11. Animated close: fade out + scale down

12. Respects `prefers-reduced-motion: reduce` (instant open/close)

13. TypeScript props: `isOpen`, `onClose`, `title`, `children`, `actions`, `closeOnEsc`, `closeOnOutsideClick`

### Toast & Notification Banner
14. 4 variants with colors and icons:
    | Type | Color | Icon | Timeout |
    |------|-------|------|---------|
    | success | Green `--status-online` | ✓ | 4s auto-dismiss |
    | error | Red `--status-error` | ✗ | Persistent (manual dismiss) |
    | warning | Yellow `--status-warning` | ⚠ | 8s auto-dismiss |
    | info | Blue `--status-human-wait` | ℹ | 6s auto-dismiss |

15. Stacked at bottom-right corner, above bottom drawer

16. Each toast has: icon, message text, close button

17. Slide-in animation from right, slide-out to right (`--ease-normal` 200ms)

18. Auto-dismiss with pause-on-hover (hovering a toast pauses its timer)

19. Maximum 5 visible toasts; older toasts pushed off when limit exceeded

20. `role="alert"` + `aria-live="assertive"` for error toasts

21. TypeScript interface for toast: `{ id, type, message, duration?, onUndo? }`

## Out of Scope

- ApprovalDialog (HITL checkpoint — separate component in Wave 6)
- Toast action buttons (undo, retry) — v1.1

## Review Triggers Expected

- Code Reviewer
- A11y & UX Specialist
