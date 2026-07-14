# MSPEC-012 · Command Palette Button in Navbar

**Status:** Draft · **Owner:** Frontend Engineer · **Ring:** 2

## Goal
Add a command palette trigger button in the Navbar (next to the right-panel toggle) and wire it to open the existing palette modal (Ctrl+K). The palette itself already exists inline in `App.tsx` as a `<Modal>` with command items — no structural changes to the palette content needed.

## Files in Scope
| File | Change |
|------|--------|
| `src/components/Layout/Navbar.tsx` | Add command palette button after rpToggle and before settingsBtn |
| `src/components/Layout/Navbar.module.css` | No CSS changes needed (existing `.pill` styles cover the button) |
| `src/App.tsx` | Pass `onTogglePalette={() => setShowPalette(true)}` to `<Navbar>` |

## Acceptance
1. A command palette button (icon `?` with tooltip "Command palette (Ctrl+K)") appears in the Navbar between the right-panel toggle and the settings button.
2. Clicking it opens the existing command palette modal.
3. `npx tsc --noEmit` passes with zero errors.

## References
- Navbar already has `onTogglePalette?: () => void` in its interface (prop exists, unused)
- Palette Modal already live in App.tsx with `showPalette` state and Ctrl+K shortcut (`Ctrl+,` is settings, `Ctrl+K` is palette)
- Palette items already exist: New session, Toggle drawer, Switch agent, Open settings, Copy code, Kill agent
