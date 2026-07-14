# MSPEC-T1 — xterm.js Terminal UI Integration

> **PRIORITY:** This task is time-critical. Execute in Phase order (Phase 1 → Phase 2 → Phase 3). Each phase is independent and testable.
> **Owner:** Frontend Engineer
> **Reviewers:** QA, Code Reviewer

---

## Goal

Add a professional terminal output display to ShiroScout using xterm.js. Users will see raw command output in a real terminal emulator alongside the chat interface. No more "crunched up" output formatting.

**Deliverable:** Terminal pane integrated into React UI, displaying command output with proper line breaks, monospace font, and (Phase 2 only) ANSI color support.

---

## Files in Scope

```
src-tauri/src/                    [edit: add Tauri command if needed]
src-tauri/Cargo.toml              [edit: add serde_json if not present]
src/                              [unchanged for Phase 1]
src/components/Terminal.tsx       [CREATE - new React component]
src/App.tsx                       [EDIT - add Terminal pane to layout]
src/index.css                     [EDIT - add terminal styles]
package.json                      [EDIT: add xterm dependencies]
```

Nothing else. Do not edit other files.

---

## PHASE 1: Prompt/Output Format Rules (TODAY — 15 minutes)

### 1.1 Update KICKOFF_PROMPT.md

Add this section after the non-negotiables table:

```markdown
## Output Format Standards

### Terminal/Command Output

ALL command output must use this exact format:

\`\`\`terminal
$ [command]
[output line 1]
[output line 2]
...
\`\`\`

**Examples:**

\`\`\`terminal
$ ls /workspace
Wayne-Tiger-ROAR
snickers.txt
test-mount.txt
\`\`\`

\`\`\`terminal
$ cargo build --target x86_64-pc-windows-msvc
   Compiling shiro_scout v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] [0.28s]
\`\`\`

**Rules:**
- ALWAYS use code fence \`\`\`terminal (not \`\`\`bash or bare text)
- ALWAYS preserve line breaks (one output line per line)
- Include the `$` prompt for context
- No extra prose INSIDE the code block
- Put analysis/summary AFTER the block in regular prose

**Never do:**
- ❌ "The output is Wayne-Tiger-ROAR snickers.txt test-mount.txt"
- ❌ Mix command output with explanation in the same paragraph
- ❌ Use \`\`\`bash or \`\`\`sh — always \`\`\`terminal
```

**Verification:** Grep for "Output Format Standards" in KICKOFF_PROMPT.md. Should exist and be readable.

---

## PHASE 2: Install and Configure xterm.js (THIS WEEK — 45 minutes)

### 2.1 Install xterm.js and dependencies

```bash
cd src-tauri  # or wherever package.json is
npm install xterm xterm-addon-fit
```

**Verify:**
```bash
npm list xterm
# Should show: xterm@X.X.X and xterm-addon-fit@X.X.X
# Both should be in package.json under "dependencies"
```

### 2.2 Create the Terminal React Component

**File:** `src/components/Terminal.tsx` (CREATE NEW)

```typescript
import React, { useEffect, useRef } from 'react';
import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import 'xterm/css/xterm.css';

interface TerminalProps {
  output: string;  // Raw command output
  command?: string; // The command that was run (optional)
}

export const TerminalDisplay: React.FC<TerminalProps> = ({ output, command }) => {
  const terminalRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);

  useEffect(() => {
    if (!terminalRef.current) return;

    // Create terminal instance
    const term = new Terminal({
      rows: 20,
      cols: 120,
      fontFamily: 'Menlo, Monaco, "Courier New", monospace',
      fontSize: 12,
      lineHeight: 1.4,
      theme: {
        background: '#0D0D0F',  // Neo-Glass dark (from your theme)
        foreground: '#E8E8E8',
        cursor: '#00FF00',
      },
    });

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.open(terminalRef.current);

    // Write content
    if (command) {
      term.write(`\r\n$ ${command}\r\n`);
    }
    if (output) {
      // Split output into lines and write each
      const lines = output.split('\n');
      lines.forEach((line, idx) => {
        term.write(line);
        if (idx < lines.length - 1) {
          term.write('\r\n');
        }
      });
    }

    term.write('\r\n');
    fitAddon.fit();

    termRef.current = term;

    // Cleanup
    return () => {
      if (termRef.current) {
        termRef.current.dispose();
      }
    };
  }, [output, command]);

  return (
    <div
      ref={terminalRef}
      style={{
        width: '100%',
        height: '400px',
        borderRadius: '8px',
        overflow: 'hidden',
        border: '1px solid rgba(139, 92, 246, 0.3)',
      }}
    />
  );
};

export default TerminalDisplay;
```

**Verification:**
```bash
# File should exist and have no syntax errors
ls -l src/components/Terminal.tsx
# Should be ~80 lines
wc -l src/components/Terminal.tsx
```

### 2.3 Import xterm CSS globally

**File:** `src/index.css` (EDIT)

Add at the TOP of the file:

```css
/* xterm.js terminal styling */
@import 'xterm/css/xterm.css';

/* Neo-Glass terminal theme override */
.xterm {
  background-color: #0D0D0F;
  color: #E8E8E8;
  font-family: 'Menlo', 'Monaco', 'Courier New', monospace;
  font-size: 12px;
}

.xterm-cursor {
  background-color: #00FF00;
  color: #0D0D0F;
}
```

**Verification:**
```bash
grep -n "xterm.css" src/index.css
# Should show the import statement
```

### 2.4 Verify xterm.js builds without errors

```bash
cd src-tauri
npm run build  # or your build command
# Should complete without "xterm" errors
```

**Acceptance Criteria:**
- [ ] `npm list xterm` shows both packages installed
- [ ] `src/components/Terminal.tsx` exists and is valid TypeScript
- [ ] `src/index.css` imports xterm CSS
- [ ] `npm run build` completes without xterm-related errors

---

## PHASE 3: Integrate Terminal into Chat UI (AFTER Phase 2 — 2 hours)

### 3.1 Update Chat Component to Send Terminal Output

Wherever the chat handles tool responses, when displaying command output, use the Terminal component.

**File:** Identify your main chat component (likely `src/components/Chat.tsx` or `src/pages/ChatPage.tsx`)

Find where messages are rendered. Around the tool response section, change:

**BEFORE:**
```typescript
// Old way: just display as text
<div>{toolResult.output}</div>
```

**AFTER:**
```typescript
// New way: use Terminal component
import { TerminalDisplay } from './Terminal';

// In your render:
<TerminalDisplay 
  command={toolResult.command}
  output={toolResult.output}
/>
```

### 3.2 Update message type (if needed)

Ensure your message/tool-result type includes `command` field:

```typescript
interface ToolResult {
  type: 'tool_result';
  command?: string;     // e.g., "ls /workspace"
  output: string;       // Raw output
  tool_call_id: string;
}
```

### 3.3 Style the terminal in your layout

Add styling so terminal doesn't overflow:

```css
.terminal-container {
  margin: 16px 0;
  max-height: 500px;
  overflow-y: auto;
  border-radius: 8px;
  padding: 8px;
  background: rgba(13, 13, 15, 0.8);
}

.terminal-container .xterm {
  padding: 16px;
  border-radius: 4px;
}
```

**Verification:**
```bash
# Test in browser: run a command in chat
# Look for a terminal pane below the chat message
# Output should have proper line breaks and dark background
```

---

## DONE Verification (All Phases)

### Phase 1 Verification
```bash
grep -A 5 "Output Format Standards" docs/governance/KICKOFF_PROMPT.md
# Must show the section
```

### Phase 2 Verification
```bash
# Check npm packages
npm list xterm xterm-addon-fit | grep -E "xterm@|xterm-addon-fit@"

# Check file exists
ls -l src/components/Terminal.tsx

# Check build
npm run build 2>&1 | grep -i "error.*xterm" || echo "No xterm errors"
```

### Phase 3 Verification
```bash
# In browser dev tools:
# 1. Run: ls /workspace
# 2. Look for <div> with class containing "xterm"
# 3. Check output has line breaks (not crunched)
# 4. Verify dark background + green text
```

---

## Failure Paths & Handling

**If `npm install xterm` fails:**
- Check Node version: `node -v` (should be 18+)
- Try: `npm install --save xterm@latest xterm-addon-fit@latest`
- If still fails, show the full error output

**If Terminal component won't compile:**
- Check TypeScript version: `npm list typescript`
- Verify xterm types are installed: `npm list @types/xterm`
- If missing: `npm install --save-dev @types/xterm` (may not exist; ignore if not found)

**If terminal appears but output is jumbled:**
- Check that `\r\n` line endings are preserved
- Verify output isn't being stripped of whitespace
- Check browser console for xterm errors

**If terminal doesn't show at all:**
- Verify Terminal component is imported and used in Chat component
- Check React DevTools: component should appear in tree
- Verify CSS is loading: inspect element → check for xterm.css styles

---

## Success Criteria (Agent must verify BEFORE marking ✅)

- [ ] Phase 1: KICKOFF_PROMPT.md has Output Format Standards section
- [ ] Phase 2: xterm packages installed, Terminal.tsx created, CSS imported, build succeeds
- [ ] Phase 3: Terminal displays in chat when running commands, output has proper line breaks
- [ ] Terminal appearance: dark background (#0D0D0F), light text (#E8E8E8), monospace font
- [ ] No console errors when running commands in chat
- [ ] Markdown code fences with \`\`\`terminal format are no longer necessary (terminal UI handles it)

---

## Notes for Agent

1. **Do phases in order.** Phase 1 is instant. Phase 2 is standalone. Phase 3 depends on Phase 2.
2. **Each phase is reviewable separately.** You can commit/test after each phase.
3. **Exact file names matter.** Use `Terminal.tsx`, not `terminal.tsx` or `TerminalComponent.tsx`.
4. **Import paths matter.** Adjust based on your folder structure (e.g., if components are in `src-tauri/ui/components`, adjust the import).
5. **xterm.js is stable and widely used.** It's in VSCode, GitHub, many terminals. Don't overthink it.
6. **If Chat component isn't obvious, search:** `grep -rn "export.*Chat\|export default.*Chat" src/components/ src/pages/`

---

## Next Steps After Completion

Once Phase 3 is done:
1. Update BUILD_PLAN.md: mark this item ✅
2. Add to DECISIONS.md: "DEC-009 — xterm.js adopted for terminal output display"
3. Update MEMORY.md §2: note xterm dependency added
4. In future, whenever an agent runs commands, output will automatically display in terminal pane

---

*This spec is actionable. No {{PLACEHOLDERS}}. All code is ready to copy-paste.*
