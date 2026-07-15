import { useState, useEffect, useCallback, useRef } from 'react';
import styles from './App.module.css';
import {
  Navbar,
  Sidebar,
  BottomDrawer,
  RightPanel,
  ChatMessage,
  Modal,
  useToast,
  SettingsView,
  FirstRunWizard,
} from './components';
import ConfirmationDialog from './components/ConfirmationDialog/ConfirmationDialog';
import { useAppContext } from './context/AppContext';
import { respondHITL, processAgentMessage } from './tauri-commands';
import { invoke } from '@tauri-apps/api/core';

// ============================================================
// ShiroScout Persona — baked-in system prompt for the orchestrator agent.
// This constant is passed to sendMessage() as the systemPrompt parameter.
// Inspired by Agent Zero's style: concise, structured, think-aloud, high-agency.
// ============================================================
export const SHIROSCOUT_PERSONA = `You are ShiroScout — Captain Wayne's autonomous AI engineering orchestrator with sharp precision and calm confidence.

## Identity & Orchestration
- **Role**: Master orchestrator and AI engineering agent. You own the full task lifecycle. Never delegate the whole task away.
- **Agency**: High agency — retry, adapt, verify. Don't accept failure.
- **Team**: You orchestrate specialist agents (Architect, Frontend, Security, QA, Docs, DevOps, Reviewer) defined in docs/agent-profiles/. Set their status working/idle in the sidebar using '[DELEGATE:agent_id]' marker when you start delegating work, and '[COMPLETE:agent_id]' when they finish. The markers are stripped from display text. Verify their output with Ring 1/Ring 2 gates before accepting.
- **Dual-environment**: Windows host (code_execution_remote) for builds, file ops, Tauri dev. Docker sandbox (code_execution_tool) for isolated code runs, security scans. Pick the right one.

## Communication
- **Format**: Strict JSON: { thoughts, headline, tool_name, tool_args }. No JSON in markdown fences. No text before/after the JSON object.
- **Headline**: Short summary declaring intent upfront.
- **Thoughts**: Natural language reasoning before actions.
- **Tone**: Concise, precise, no fluff. "Informative but tight, not terse and not verbose."
- **Emojis**: Use naturally: ✅ success ❌ errors 📁 files 🚀 builds 🔧 fixes 🔍 search 📊 stats 🎯 goals 🧪 testing 🔒 security 📝 docs 🔄 restart
- **Formatting**: Tables for technicals, lists for summaries, \`\`\`fences with language ID for code, \`\`\`terminal with \$ prompt for shell output. Full file paths so they're clickable.
- **Analysis after code blocks** — never mix explanation with output.

## Problem-Solving Loop
0. INTERNALIZE the task fully
1. PLAN your approach (best tool? delegate? self-execute?)
2. CHECK governance files (AGENTS, FILEOPS, MEMORY, TODO), memories, project context
3. EXECUTE with verification at every step
4. REPORT: what was done, what was verified, what's next

### Coding Rules
- Read specs, tests, existing code FIRST. Inspect environment concisely.
- Make minimal focused changes matching existing style.
- Do not edit tests, docs, lockfiles, or generated files unless the task requires it.
- Verify exact: path, filename, permissions, line count, bytes, content, exit codes.
- Split long work: probe → build → run → verify. For long jobs: write logs, poll output, stop stale work.
- If a tool patch fails: inspect current file and retry with smaller context.
- Never treat timeout, partial output, or plausible result as verified success.
- Clean temp files, caches, logs, and background processes you created.
- In reports: separate verified facts from assumptions.

## Behavioral Rules
- **Self-healing**: If output fails to parse, you see correction text and retry. If you repeat yourself, try something different. If a tool errors, the error text comes back — diagnose and fix. Use Repairable (retry) vs Critical (stop) distinction.
- **Ring verification**: Ring 1 = unit tests after changes. Ring 2 = integration/contract tests. Ring 3 = Captain review for destructive actions.
- **HITL**: Before destructive actions (removing files, killing containers), ask Captain for confirmation. Present what, why, and expected outcome. Wait for explicit approval.
- **File sovereignty**: All project code under project root. Never reference outside. This app is fully self-contained — Agent Zero platform is off-limits.
- **Memory discipline**: Memorize stable facts only — not one-off commands, temp state, or implementation minutiae.
- **Favor Linux commands** in sandbox for simple tasks over Python. Use PowerShell on host.
- **Build awareness**: pnpm build (frontend), cargo tauri build (production), cargo check (Rust lint), cargo clippy -- -D warnings (gate).

## Verification Culture
- **Never assume success.** File written? Read it back. Command ran? Check exit code. Build complete? Test the binary.
- High-agency means resourcefulness, not guesswork. Verify everything.
- Don't leave the project messier than you found it.
`;

// Simple LLM log helper
function addLog(status: string, category: string, message: string) {
  console.log(`[${status}][${category}] ${message}`);
}


function App() {
  const { addToast } = useToast();
  const {
    dockerInfo,
    agents,
    activeAgentId,
    setActiveAgentId,
    updateAgentStatus,
    sessions,
    activeSessionId,
    setActiveSessionId,
    addSession,
    removeSession,
    settings,
    showRightPanel,
    setShowRightPanel,
    drawerCollapsed,
    setDrawerCollapsed,
    messages,
    streamingMessage,
    isStreaming,

    abortStream,
    addUserMessage,
    clearMessages,
    llmConfig,
    currentPendingHITL,
    setCurrentPendingHITL,
  } = useAppContext();

  // Chat state
  const [inputValue, setInputValue] = useState('');
  const threadRef = useRef<HTMLDivElement>(null);
  const [isUserNearBottom, setIsUserNearBottom] = useState(true);

  // Agent/session phase state
  const [phase, setPhase] = useState<'online' | 'thinking' | 'gather' | 'tool' | 'stream'>('online');
  const [phasePct, setPhasePct] = useState<number | null>(null);

  // Modal state
  const [showSettings, setShowSettings] = useState(false);
  const [showPalette, setShowPalette] = useState(false);
  const [showWizard, setShowWizard] = useState(false);

  // Keyboard shortcuts
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setCurrentPendingHITL(null); setShowSettings(false); setShowPalette(false); setShowWizard(false);
      }
      if (e.ctrlKey && e.key === 'Enter') handleSend();
      if (e.ctrlKey && e.key === '`') { e.preventDefault(); setDrawerCollapsed(p => !p); }
      if (e.ctrlKey && (e.key === 'k' || e.key === 'K')) { e.preventDefault(); setShowPalette(true); }
      if (e.ctrlKey && e.key === ',') { e.preventDefault(); setShowSettings(true); }
    };
    document.addEventListener('keydown', handler);
    return () => document.removeEventListener('keydown', handler);
  }, [inputValue, isStreaming]);
  // Auto-scroll when new content arrives and user is near bottom
  useEffect(() => {
    const el = threadRef.current;
    if (!el) return;
    if (!isUserNearBottom) return;
    el.scrollTo({ top: el.scrollHeight, behavior: 'smooth' });
  }, [messages.length, streamingMessage, isStreaming, isUserNearBottom]);

  const handleThreadScroll = useCallback(() => {
    const el = threadRef.current;
    if (!el) return;
    const near = el.scrollHeight - el.scrollTop - el.clientHeight < 100;
    setIsUserNearBottom(near);
  }, []);

  // Process delegation markers [DELEGATE:agent_id] and [COMPLETE:agent_id]
  // Invokes set_agent_status and strips markers from display text
  const processDelegationMarkers = useCallback((text: string): string => {
    const markerRegex = /\[(DELEGATE|COMPLETE):(\w+)\]/g;
    let match;
    while ((match = markerRegex.exec(text)) !== null) {
      const [, action, agentId] = match;
      const status = action === 'DELEGATE' ? 'online' : 'off';
      invoke('set_agent_status', { agent_id: agentId, status });
    }
    return text.replace(markerRegex, '');
  }, []);

  const processedStreamingMessage = streamingMessage ? processDelegationMarkers(streamingMessage) : '';

  // Mark orchestrator as thinking during streaming; update sidebar agent list
  const agentsWithState = agents.map(a => ({
    ...a,
    isActive: a.id === activeAgentId,
    isThinking: a.id === 'orchestrator' ? isStreaming : (a as any).isThinking,
  }));

  const handleSend = useCallback(async () => {
    if (!inputValue.trim() || isStreaming) return;
    if (sessions.length === 0) {
      const id = `sess-${Date.now()}`;
      const title = inputValue.trim().slice(0, 40);
      addSession({ id, title, group: 'Today' });
    }
    addUserMessage(inputValue);
    setInputValue('');
    setPhasePct(null);
    try {
      const response = processDelegationMarkers(await processAgentMessage(inputValue)); addLog('ok', 'llm', `Agent: ${response.slice(0,80)}`);
      setPhase('online');
      setPhasePct(null);
    } catch (err) {
      abortStream();
      addToast({ message: `Stream error: ${String(err)}`, type: 'error' });
      setPhase('online');
      setPhasePct(null);
    }
  }, [inputValue, isStreaming, sessions, addSession, addUserMessage, llmConfig, abortStream, addToast]);

  const handleNewSession = useCallback(() => {
    const id = `sess-${Date.now()}`;
    addSession({ id, title: 'New Session', group: 'Today' });
    clearMessages();
    setPhase('online');
    addToast({ message: 'New session started', type: 'info' });
  }, [addSession, clearMessages, addToast]);

  const handleKillAgent = useCallback(async () => {
    setPhase('online');
    updateAgentStatus(activeAgentId, 'err');
    addToast({ message: `Agent ${agents.find(a => a.id === activeAgentId)?.name} terminated`, type: 'error' });
  }, [activeAgentId, updateAgentStatus, agents, addToast]);

  const currentAgent = agents.find(a => a.id === activeAgentId) || agents[0];
  const navStatus = phase === 'online' ? 'online' as const : 'thinking' as const;
  const currentSession = sessions.find(s => s.id === activeSessionId);
  const dockerVersionLabel = dockerInfo.version || 'unknown';
  const containerLabel = dockerInfo.status === 'available'
    ? `aegis-sbx (v${dockerVersionLabel})`
    : dockerInfo.status === 'checking'
    ? 'checking...'
    : 'no sandbox';

  return (
    <div className={styles.app}>
      <a className="skip" href="#chatInput">Skip to content</a>
      <pre className="backdrop" aria-hidden="true" id="backdrop"></pre>

      <Navbar
        agentName={currentAgent.name}
        agentStatus={navStatus}
        agentPhase={phase}
        providerLabel={settings.model}
        sandboxLabel={containerLabel}
        onSettingsClick={() => setShowSettings(true)}
        onToggleRightPanel={() => setShowRightPanel(p => !p)}
      />

      <div className={styles.workspace}>
        <Sidebar
          agents={agentsWithState}
          sessions={sessions.map(s => ({ ...s, isActive: s.id === activeSessionId }))}
          onToggleRail={() => setDrawerCollapsed(p => !p)}
          onNewSession={handleNewSession}
          onAgentClick={(id) => { setActiveAgentId(id); addToast({message: `Switched to agent ${agents.find(a=>a.id===id)?.name}`, type: 'info'}); }}
          onSessionClick={(id) => setActiveSessionId(id)}
          onSessionDelete={(id) => removeSession(id)}
        />

        <main className={styles.main}>
          <div className="chat-header">
            <span className="chat-title">{currentSession?.title || 'Session'}</span>
            <span className="chat-meta">session #{activeSessionId.slice(0, 5)} | workspace /workspace | Docker: {containerLabel}</span>
            <span style={{flex:1}}></span>
            <button className="btn sm ghost" title="Export session">⬇ Export</button>
            <button className="btn sm ghost" title="Rename session">✏</button>
          </div>

          <div className="thread" id="thread" role="log" aria-live="polite" aria-label={`Conversation with agent ${currentAgent.name}`} ref={threadRef} onScroll={handleThreadScroll}>
            <div className="msg system">- session started | sandbox <b>{containerLabel}</b> attached | /workspace mounted read-write -</div>

            {messages.map((msg, i) => (
              <ChatMessage key={i} variant={msg.role as 'user' | 'agent' | 'system'} who={msg.role === 'user' ? 'You' : currentAgent.name} timestamp={msg.timestamp} content={msg.content} />
            ))}

            {isStreaming && streamingMessage && (
              <ChatMessage
                variant="agent"
                who={currentAgent.name}
                timestamp="now"
                content={processedStreamingMessage}
                isStreaming={true}
              />
            )}
          </div>

          <div className="phase-strip" id="phaseStrip" role="status" aria-live="assertive">
            <span className="picon" id="phaseIcon">
              {phase === 'thinking' && '⚙'}
              {phase === 'gather' && '⚡'}
              {phase === 'tool' && '⚒'}
              {phase === 'stream' && '→'}
              {phase === 'online' && '●'}
            </span>
            <span id="phaseText" style={{color:'var(--text-primary)',fontWeight:600}}>
              {phase === 'online' && 'Online - waiting for input'}
              {phase === 'thinking' && 'Thinking...'}
              {phase === 'gather' && 'Gathering context - reading workspace files'}
              {phase === 'tool' && 'Executing tool - write_file'}
              {phase === 'stream' && 'Streaming response'}
            </span>
            {phasePct != null && (
              <>
                <span className="bar"><i id="phaseBar" style={{width:phasePct+'%'}}></i></span>
                <span className="pct" id="phasePct">{phasePct}%</span>
              </>
            )}
          </div>

          <div className="chat-input">
            <div className="ci-box">
              <textarea
                id="chatInput"
                rows={1}
                placeholder={`Message ${currentAgent.name}...`}
                aria-label="Message the agent"
                value={inputValue}
                onChange={e => setInputValue(e.target.value)}
                onKeyDown={e => { if (e.key === 'Enter' && e.ctrlKey) { e.preventDefault(); handleSend(); }}}
              />
              <div className="ci-row">
                <button className="btn icon ghost sm" title="Attach file" aria-label="Attach file" onClick={() => addToast({message: 'File attachment coming soon', type: 'info'})}>📎</button>
                <button className="btn icon ghost sm" title="Slash commands" aria-label="Slash commands" onClick={() => setInputValue(prev => prev + '/')}>/</button>
                <button className="btn icon ghost sm" title="Insert code block" aria-label="Insert code block" onClick={() => setInputValue(prev => prev + '\n```\n\n```\n')}>{ }</button>
                <span className="ci-hint">
                  <span className="ci-count" id="charCount">{inputValue.length}</span>
                  <kbd>Ctrl</kbd>+<kbd>Enter</kbd>
                </span>
                <button className="btn primary" id="sendBtn" onClick={handleSend} disabled={!inputValue.trim() || isStreaming}>
                  Send →
                </button>
              </div>
            </div>
          </div>
        </main>

        {showRightPanel && (
          <RightPanel
            agentName={currentAgent.name}
            statusLabel={dockerInfo.status === 'available' ? '● Active' : dockerInfo.status === 'error' ? '⚠ Error' : '○ Checking'}
            model={settings.model}
            provider={settings.provider === 'local' ? 'Local | Ollama' : 'Cloud | ' + settings.model}
            toolsEnabled={5}
            sessionTime={new Date().toLocaleTimeString()}
            hitlMode="Ask every write"
            tokensUsed={12401}
            tokenLimit={128000}
            recentTools={[
              { name: 'check_docker_daemon', status: dockerInfo.status === 'available' ? 'ok' as const : 'bad' as const, duration: '0.3s' },
              { name: 'Docker version', status: dockerInfo.version ? 'ok' as const : 'bad' as const, duration: '-' },
              { name: 'containers', status: (dockerInfo.containers.length > 0 ? 'ok' : 'run') as 'ok' | 'run' | 'bad', duration: `${dockerInfo.containers.length} active` },
            ]}
            costSession="$0.00 (local)"
            costDetail="disabled"

          />
        )}
      </div>

      <BottomDrawer
        collapsed={drawerCollapsed}
        onToggleCollapse={() => setDrawerCollapsed(p => !p)}
      />

      {currentPendingHITL && (
        <ConfirmationDialog
          isOpen={true}
          onApprove={(reason) => {
            respondHITL(currentPendingHITL.session_id, true, reason, currentPendingHITL.nonce);
            setCurrentPendingHITL(null);
          }}
          onReject={(reason) => {
            respondHITL(currentPendingHITL.session_id, false, reason, currentPendingHITL.nonce);
            setCurrentPendingHITL(null);
          }}
          operationName={currentPendingHITL.operation_name}
          operationDescription={currentPendingHITL.operation_description}
          riskLevel={currentPendingHITL.risk_level as 'critical' | 'high' | 'medium' | 'low'}
          onClose={() => setCurrentPendingHITL(null)}
        />
      )}

      <SettingsView isOpen={showSettings} onClose={() => setShowSettings(false)} />

      {/* Command palette */}
      <Modal
        isOpen={showPalette}
        onClose={() => setShowPalette(false)}
        title=""
        palette={true}
      >
        <input className="input" placeholder="Type a command..." id="palInput" aria-label="Search commands" autoFocus />
        <div className="pal-list">
          <button className="pal-item" onClick={() => { setShowPalette(false); handleNewSession(); }}>➕ New session <kbd>Ctrl+Shift+N</kbd></button>
          <button className="pal-item" onClick={() => { setShowPalette(false); setDrawerCollapsed(p=>!p); }}>⬅ Toggle bottom drawer <kbd>Ctrl+`</kbd></button>
          {agents.filter(a => a.id !== activeAgentId).map(a => (
            <button key={a.id} className="pal-item" onClick={() => { setShowPalette(false); setActiveAgentId(a.id); addToast({message:`Switched to agent ${a.name}`, type:'info'}); }}>
              {a.avatar} Switch to agent {a.name} <kbd>Ctrl+Shift+{agents.indexOf(a)+1}</kbd>
            </button>
          ))}
          <button className="pal-item" onClick={() => { setShowPalette(false); setShowSettings(true); }}>⚙ Open settings <kbd>Ctrl+,</kbd></button>
          <button className="pal-item" onClick={() => { setShowPalette(false); addToast({message:'Copied last code block', type:'success'}); }}>✂ Copy last code block <kbd>Ctrl+Shift+C</kbd></button>
          <button className="pal-item" onClick={() => { setShowPalette(false); handleKillAgent(); }}>✖ Kill {currentAgent.name}</button>
        </div>
      </Modal>

      <FirstRunWizard isOpen={showWizard} onComplete={() => setShowWizard(false)} onSkip={() => setShowWizard(false)} />

      {/* Demo launcher — development only */}
      {import.meta.env.DEV && (
        <div className="demo-pill" id="demoPill">
          <button className="btn secondary sm" id="demoBtn" onClick={() => document.getElementById('demoPill')?.classList.toggle('open')}>⚙ UI demos</button>
          <div className="demo-menu glass-overlay">
            <button className="btn" onClick={() => setCurrentPendingHITL({ session_id: 'demo', operation_name: 'demo_operation', operation_description: 'Demo operation for testing', risk_level: 'high', payload: null, nonce: 'demo-nonce-123' })}>⚠ HITL approval dialog</button>
            <button className="btn" onClick={() => setShowWizard(true)}>🎉 First-run wizard</button>
            <button className="btn" onClick={() => setShowPalette(true)}>⌘ Command palette</button>
            <button className="btn" onClick={() => { addToast({message:'Tool completed - write_file 0.9s', type:'success'}); addToast({message:'Session autosaved', type:'info'}); addToast({message:'Context window at 78%', type:'warning'}); addToast({message:'LLM connection lost - retrying in 5s', type:'error'}); }}>🔔 Toast notifications</button>
            <button className="btn" onClick={() => { setPhase('online'); addToast({message:'Agent Alpha hit an error | open Logs for details', type:'error'}); }}>⚠ Simulate agent error</button>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;





