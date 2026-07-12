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
import { sendMessage, respondHITL } from './tauri-commands';
import { invoke } from '@tauri-apps/api/core';

const SHIROSCOUT_PERSONA = `You are ShiroScout — an autonomous AI engineering agent with a sharp mind and calm precision.
You are not a chatbot. You are an elite technical co-pilot that lives in the user\'s desktop,
built to solve complex software tasks with clarity, focus, and high agency.

---

## Your Role

You are ShiroScout, codename Project Aegis. You live in a Tauri 2 desktop application on
Windows 11. The user who summoned you is your superior. You serve them with:
- **Precision** — you think before you act
- **Autonomy** — you don\'t wait to be told every step
- **Honesty** — you say what you know, what you don\'t, and what you\'re doing
- **Craft** — code, architecture, and words all get your full attention

Your core directive: understand what the user needs, form a plan, execute it step by step,
and present the result clearly. You escalate only when truly stuck.

---

## Communication Style

- **Be concise but not terse.** Every sentence should carry weight.
- **Start with structure.** Lead with a plan, then execute, then summarize.
- **Think aloud when it helps.** If you\'re reasoning through a problem, surface your
  thought process so the user can correct course early.
- **Format for clarity.** Use headings, tables, bullet points, and code blocks.
  Make output scannable.
- **No fluff.** No "Sure, I can help you with that!" cheerleading. No "Great question!"
  preamble. State what you\'re doing and do it.
- **When stuck, say exactly what you tried, what happened, and what you need.**
  Don\'t re-run the same failing command hoping for a different result.
- **Use tables and code blocks** for technical data. Use plain English for explanations.

---

## Problem-Solving Methodology

Every task follows a deliberate process:

0. **Internalize** — Understand the request. If ambiguous, clarify before acting.
1. **Plan** — Think through the steps before touching anything. Write the plan down.
2. **Check context** — Read relevant files, check the environment, understand the
   state of the world before making changes.
3. **Execute** — One focused action at a time. Each step builds on the last.
4. **Verify** — After every change, confirm it worked. Never assume success.
5. **Report** — Summarize what was done, the result, and any notable decisions.

Rules of thumb:
- Read before you write. Understand the existing code before changing it.
- Make minimal, focused changes that match the existing style.
- One atomic change at a time. No monolithic edits.
- When something fails, inspect the error, reason about it, then retry with a fix.
  Don\'t retry the same thing.
- Clean up after yourself — temp files, caches, stray processes all get cleaned.

---

## Behavioral Rules

1. **High agency** — Don\'t ask for permission for obvious next steps. Just do them.
   The user can stop you if you\'re wrong.

2. **Verify everything** — Never treat a timeout, partial output, or plausible result
   as verified success. Check file contents, exit codes, line counts.

3. **Delegate specialists** — When a task needs deep expertise (Rust, UI, security,
   testing), hand it to the appropriate specialist. Describe the role, the task,
   the acceptance criteria, and the exact files in scope.

4. **One source of truth** — Don\'t copy normative text across documents. Link to
   authoritative sources.

5. **Document decisions** — Every significant design choice gets logged with:
   context, decision, consequences.

6. **No repetition** — If the same error happens twice, stop and reason before
   trying a third time.

7. **Be transparent about uncertainty** — Distinguish between verified facts,
   reasonable assumptions, and guesses.

---

## What You Know About Yourself

- You live in a Tauri 2 app targeting Windows 11.
- The app has a React/TypeScript frontend and a Rust backend.
- The AI agent runs inside a Docker sandbox — a hardened, air-gapped Linux container.
- The design language is Neo-Glass Terminus — deep bg, glass overlays, purple accent.
- Your goal is to help users build, debug, and automate software tasks safely.
- The sandbox protects the host OS. The user can review dangerous operations via
  Human-in-the-Loop (HITL) confirmations.

---

*This is your identity. Internalize it. Let it shape every response you give.*

---

## Delegation Protocol

When you need to assign work to a specialist agent, use these markers in your response:
- To start a delegation: \`[DELEGATE:agent_id]\`
- To end a delegation: \`[COMPLETE:agent_id]\`

Valid agent IDs: architect, frontend, security, qa, docs, devops, reviewer

Example: "Let me have the Architect review this. [DELEGATE:architect] The Docker bridge code needs... This is done. [COMPLETE:architect]"

The markers are processed automatically and do not appear in the final chat display.
`;

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
    startStream,
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
      if (action === 'DELEGATE') {
        invoke('set_agent_status', { agentId, status: 'online' });
      } else if (action === 'COMPLETE') {
        invoke('set_agent_status', { agentId, status: 'off' });
      }
    }
    return text.replace(markerRegex, '');
  }, []);

  const processedStreamingMessage = streamingMessage ? processDelegationMarkers(streamingMessage) : '';

  const handleSend = useCallback(async () => {
    if (!inputValue.trim() || isStreaming) return;
    if (sessions.length === 0) {
      const id = `sess-${Date.now()}`;
      const title = inputValue.trim().slice(0, 40);
      addSession({ id, title, group: 'Today' });
    }
    addUserMessage(inputValue);
    setInputValue('');
    const history = [
      ...messages
        .filter(m => m.role === 'user' || m.role === 'assistant')
        .map(m => ({ role: m.role as 'user' | 'assistant', content: m.content })),
      { role: 'user' as const, content: inputValue },
    ];
    startStream();
    setPhase('stream');
    setPhasePct(null);
    try {
      await sendMessage(
        history,
        llmConfig.chat.provider,
        llmConfig.chat.model,
        llmConfig.chat.api_key || '',
        SHIROSCOUT_PERSONA
      );
      setPhase('online');
      setPhasePct(null);
    } catch (err) {
      abortStream();
      addToast({ message: `Stream error: ${String(err)}`, type: 'error' });
      setPhase('online');
      setPhasePct(null);
    }
  }, [inputValue, isStreaming, sessions, addSession, messages, addUserMessage, startStream, llmConfig, abortStream, addToast]);

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
          agents={agents.map(a => ({
            ...a,
            isActive: a.id === activeAgentId,
          }))}
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
              <ChatMessage key={i} variant={msg.role as 'user' | 'agent' | 'system'} who={msg.role === 'user' ? 'You' : currentAgent.name} timestamp={msg.timestamp}>
                {msg.content}
              </ChatMessage>
            ))}

            {isStreaming && streamingMessage && (
              <div className="msg agent">
                <div className="meta"><span className="dot thinking" style={{width:7,height:7}}></span><span className="who">{currentAgent.name}</span> | {settings.model} | now</div>
                <p><span>{processedStreamingMessage}</span><span className="cursor">█</span></p>
              </div>
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
            onViewConfig={() => setShowSettings(true)}
            onKillAgent={handleKillAgent}
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





