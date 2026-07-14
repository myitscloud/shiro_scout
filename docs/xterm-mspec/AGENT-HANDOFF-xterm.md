# Agent Handoff: xterm.js Terminal Integration

> Paste this into your AI agent (DeepSeek/Claude Code) to execute MSPEC-T1 directly.

---

You are being asked to implement a professional terminal emulator (xterm.js) into ShiroScout. This task has three independent phases. **Execute them in order and verify each phase before moving to the next.**

## Context

Currently, command output in the chat window displays as crunched, unformatted text. The goal is to add a real terminal pane that displays raw bash output with proper line breaks, monospace font, and dark theme (matching ShiroScout's Neo-Glass design).

**Reference:** Full spec is in `MSPEC-T1-xterm-integration.md` (in outputs or project root). Read it before starting.

---

## PHASE 1: Prompt Format Rules (15 minutes)

**Owner:** You (any agent role)  
**Complexity:** Trivial — text edit only

### Task

Edit `docs/governance/KICKOFF_PROMPT.md` (or equivalent location in your ShiroScout project).

Find or create a section called "Output Format Standards" and add:

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

### Verify

```bash
grep -A 5 "Output Format Standards" docs/governance/KICKOFF_PROMPT.md
# Should print the section you just added
```

**When done:** Report back with the grep output.

---

## PHASE 2: Install xterm.js (45 minutes)

**Owner:** Frontend Engineer  
**Complexity:** Package install + create one component

### Task 2.1: Install packages

```bash
cd src-tauri  # wherever your package.json is
npm install xterm xterm-addon-fit
```

Verify:
```bash
npm list xterm
# Output should show both xterm and xterm-addon-fit with version numbers
```

### Task 2.2: Create Terminal React Component

**File:** `src/components/Terminal.tsx` (NEW FILE)

Copy this exactly:

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
        background: '#0D0D0F',  // Neo-Glass dark
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

Verify file was created:
```bash
ls -l src/components/Terminal.tsx
wc -l src/components/Terminal.tsx  # Should be ~80 lines
```

### Task 2.3: Add xterm CSS to index.css

**File:** `src/index.css` (EDIT — add at TOP of file)

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

Verify:
```bash
grep -n "xterm.css" src/index.css
# Should show the @import line
```

### Task 2.4: Build and verify no errors

```bash
cd src-tauri
npm run build
# Should complete without "xterm" errors
# If it fails, show me the full error output
```

### Verify Phase 2

When complete, run ALL of these:

```bash
# 1. Check packages installed
npm list xterm xterm-addon-fit

# 2. Check file exists and has content
ls -l src/components/Terminal.tsx
wc -l src/components/Terminal.tsx

# 3. Check CSS import
grep "xterm.css" src/index.css

# 4. Check build succeeds
npm run build 2>&1 | tail -20
```

**Report back with output from all four commands above.**

If all pass, Phase 2 is done. If any fails, show the error and we'll fix it.

---

## PHASE 3: Integrate into Chat UI (2 hours)

**Owner:** Frontend Engineer + Code Reviewer  
**Complexity:** Medium — component integration + styling

### Task 3.1: Identify your Chat component

Find which file renders chat messages. Usually:

```bash
find src -name "*Chat*" -o -name "*chat*" | grep -E "\.(tsx|jsx)$"
# Or search for where tool_result is displayed:
grep -rn "tool_result\|tool.result" src/components/ src/pages/
```

You're looking for a React component that renders the agent's responses and tool outputs.

### Task 3.2: Import Terminal component

At the top of your Chat component file, add:

```typescript
import TerminalDisplay from './Terminal';
// Adjust the path based on your folder structure
// If Terminal.tsx is in components/ and Chat is also in components/, use './Terminal'
// If they're in different folders, adjust accordingly
```

### Task 3.3: Update message rendering

Find where tool results are displayed. It probably looks like:

```typescript
// BEFORE:
<div className="tool-result">
  <pre>{toolResult.output}</pre>
</div>
```

Change it to:

```typescript
// AFTER:
<div className="tool-result">
  <TerminalDisplay 
    command={toolResult.command || message.content}
    output={toolResult.output}
  />
</div>
```

### Task 3.4: Ensure message types include command field

Check your TypeScript interfaces. The tool result should look like:

```typescript
interface ToolResult {
  type: 'tool_result';
  command?: string;      // e.g., "ls /workspace"
  output: string;        // Raw terminal output
  tool_call_id: string;
}
```

If `command` field doesn't exist, add it as optional.

### Task 3.5: Add styling (optional but recommended)

In your CSS file (probably where chat styles live), add:

```css
.tool-result {
  margin: 12px 0;
  border-radius: 8px;
  overflow: hidden;
}

.tool-result .xterm {
  padding: 12px;
  border-radius: 4px;
}
```

### Task 3.6: Test in the application

```bash
# Build the app
npm run build

# Start the Tauri app (or dev server, depending on your setup)
npm run tauri dev
# or
npm run dev
```

In the chat window, **run a command:**
```
ls /workspace
```

**Look for:**
1. A terminal-styled pane appears below the message
2. Output shows with proper line breaks (not crunched)
3. Dark background (#0D0D0F), light text
4. Monospace font
5. No errors in browser dev console (F12)

### Verify Phase 3

In browser, when you run `ls /workspace`:

```
Expected to see:
┌─ Dark pane with terminal styling ─────┐
│ $ ls /workspace                       │
│ Wayne-Tiger-ROAR                      │
│ snickers.txt                          │
│ test-mount.txt                        │
└───────────────────────────────────────┘
```

NOT:
```
❌ "The output is Wayne-Tiger-ROAR snickers.txt test-mount.txt" (old crunched style)
```

**When complete, report:**
- Screenshot or description of terminal pane
- Whether output has line breaks
- Any console errors (if yes, paste the error)

---

## Troubleshooting

### "Module not found: xterm"
```bash
npm install xterm xterm-addon-fit
npm list xterm  # Verify both installed
```

### Terminal component won't compile
```bash
# Check TypeScript errors
npm run build 2>&1 | grep -i error
```

### Terminal appears but output is jumbled
- Verify `output.split('\n')` is working
- Check that output isn't being stripped of newlines
- Look at browser DevTools → Elements → find `<div class="xterm">` and inspect the content

### Terminal doesn't show at all
- Check React DevTools: does TerminalDisplay component appear?
- Verify it's being imported and used in Chat component
- Check browser console for errors

---

## Summary

**Phase 1:** ✏️ Edit KICKOFF_PROMPT.md — add output format rules (15 min, trivial)
**Phase 2:** 📦 Install xterm, create Terminal.tsx component, add CSS (45 min, straightforward)
**Phase 3:** 🔗 Integrate Terminal into Chat component, test (2 hours, requires some debugging)

**Total time:** ~3 hours to completion

**Do phases in order. Report after each phase before moving to the next.**

Good luck! 🚀
