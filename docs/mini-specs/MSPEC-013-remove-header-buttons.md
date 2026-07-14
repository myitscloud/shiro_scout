# MSPEC-013 · Remove Export & Rename Buttons from Chat Header

**Status:** Draft · **Owner:** Frontend Engineer · **Ring:** 2

## Goal
Remove the "Export session" (⬇ Export) and "Rename session" (✏) buttons from the chat header in the main window frame.

## Files in Scope
| File | Change |
|------|--------|
| `src/App.tsx` | Remove 2 `<button>` elements from the `chat-header` div |

## Acceptance
1. The two buttons (⬇ Export and ✏) no longer appear in the chat header.
2. The flex spacer `<span style={{flex:1}}></span>` that precedes them should also be removed if no other elements follow it.
3. `npx tsc --noEmit` passes with zero errors.

## Context
Current chat-header section (lines ~244-251):
```
          <div className="chat-header">
            <span className="chat-title">{currentSession?.title || 'Session'}</span>
            <span className="chat-meta">session #{activeSessionId.slice(0, 5)} | workspace /workspace | Docker: {containerLabel}</span>
            <span style={{flex:1}}></span>
            <button className="btn sm ghost" title="Export session">? Export</button>
            <button className="btn sm ghost" title="Rename session">?</button>
          </div>
```

Remove the `<span style={{flex:1}}></span>` spacer and the two `<button>` lines after it.
