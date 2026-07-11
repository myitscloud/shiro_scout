import { useState, useEffect, useCallback } from 'react';
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
import { useAppContext } from './context/AppContext';

const REPLIES = [
  'Patched src/api/handlers.rs and src/main.rs. cargo check now passes cleanly - 0 errors, 1 pre-existing warning. Migration to router_v2 is complete across all 3 files.',
  'Understood. I will read the three call sites, apply the router_v2 signature changes, and re-run the checks before handing back.',
  'Done. I logged the decision in DECISIONS.md and left the diff in the drawer for review.',
];

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
    settings,
    showRightPanel,
    setShowRightPanel,
    drawerCollapsed,
    setDrawerCollapsed,
    createContainer,
  } = useAppContext();

  // Chat state
  const [messages, setMessages] = useState<{ role: string; content: string; ts: string }[]>([]);
  const [inputValue, setInputValue] = useState('');
  const [isStreaming, setIsStreaming] = useState(false);
  const [streamContent, setStreamContent] = useState('');
  let replyIx = 0;

  // Agent/session phase state
  const [phase, setPhase] = useState<'online' | 'thinking' | 'gather' | 'tool' | 'stream'>('online');
  const [phasePct, setPhasePct] = useState<number | null>(null);

  // Modal state
  const [showHitl, setShowHitl] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [showPalette, setShowPalette] = useState(false);
  const [showWizard, setShowWizard] = useState(false);
  const [hitlCountdown, setHitlCountdown] = useState(60);

  // Keyboard shortcuts
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setShowHitl(false); setShowSettings(false); setShowPalette(false); setShowWizard(false);
      }
      if (e.ctrlKey && e.key === 'Enter') handleSend();
      if (e.ctrlKey && e.key === '`') { e.preventDefault(); setDrawerCollapsed(p => !p); }
      if (e.ctrlKey && (e.key === 'k' || e.key === 'K')) { e.preventDefault(); setShowPalette(true); }
      if (e.ctrlKey && e.key === ',') { e.preventDefault(); setShowSettings(true); }
    };
    document.addEventListener('keydown', handler);
    return () => document.removeEventListener('keydown', handler);
  }, [inputValue, isStreaming]);

  // HITL countdown
  useEffect(() => {
    if (!showHitl) { setHitlCountdown(60); return; }
    const iv = setInterval(() => {
      setHitlCountdown(c => {
        if (c <= 1) {
          setShowHitl(false);
          addToast({ message: 'Action auto-denied - no response within 60s', type: 'warning' });
          return 0;
        }
        return c - 1;
      });
    }, 1000);
    return () => clearInterval(iv);
  }, [showHitl]);

  const handleSend = useCallback(() => {
    if (!inputValue.trim() || isStreaming) return;
    const now = new Date();
    const ts = now.getHours().toString().padStart(2,'0') + ':' + now.getMinutes().toString().padStart(2,'0');
    setMessages(prev => [...prev, { role: 'user', content: inputValue, ts }]);
    setInputValue('');
    setIsStreaming(true);
    setPhase('thinking');
    setPhasePct(null);

    setTimeout(() => { setPhase('gather'); }, 900);
    setTimeout(() => {
      setPhase('stream');
      setStreamContent('');
      const target = REPLIES[replyIx++ % REPLIES.length];
      let idx = 0;
      const iv = setInterval(() => {
        idx += 3;
        if (idx >= target.length) {
          clearInterval(iv);
          setMessages(prev => [...prev, { role: 'assistant', content: target, ts }]);
          setStreamContent('');
          setIsStreaming(false);
          setPhase('online');
          setPhasePct(null);
        } else {
          setStreamContent(target.slice(0, idx));
        }
      }, 28);
    }, 2000);
  }, [inputValue, isStreaming]);

  const handleNewSession = useCallback(async () => {
    setMessages([]);
    setPhase('online');
    addToast({ message: 'New session started', type: 'info' });

    // Create a new Docker sandbox container if Docker is available
    if (dockerInfo.status === 'available') {
      try {
        await createContainer({
          image: 'aegis-sandbox:latest',
          workspace_path: '',
          memory_mb: 2048,
          cpu_shares: 512,
          network_enabled: false,
        });
      } catch (err) {
        addToast({ message: `Failed to create sandbox: ${String(err)}`, type: 'error' });
      }
    }
  }, [dockerInfo.status, createContainer, addToast]);

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
          onToggleRail={() => {}}
          onNewSession={handleNewSession}
          onAgentClick={(id) => { setActiveAgentId(id); addToast({message: `Switched to agent ${agents.find(a=>a.id===id)?.name}`, type: 'info'}); }}
          onSessionClick={(id) => setActiveSessionId(id)}
        />

        <main className={styles.main}>
          <div className="chat-header">
            <span className="chat-title">{currentSession?.title || 'Session'}</span>
            <span className="chat-meta">session #{activeSessionId.slice(0, 5)} | workspace /workspace | Docker: {containerLabel}</span>
            <span style={{flex:1}}></span>
            <button className="btn sm ghost" title="Export session">⬇ Export</button>
            <button className="btn sm ghost" title="Rename session">✏</button>
          </div>

          <div className="thread" id="thread" role="log" aria-live="polite" aria-label={`Conversation with agent ${currentAgent.name}`}>
            <div className="msg system">- session started | sandbox <b>{containerLabel}</b> attached | /workspace mounted read-write -</div>

            {messages.map((msg, i) => (
              <ChatMessage key={i} variant={msg.role as 'user' | 'agent' | 'system'} who={msg.role === 'user' ? 'You' : currentAgent.name} timestamp={msg.ts}>
                {msg.content}
              </ChatMessage>
            ))}

            {isStreaming && streamContent && (
              <div className="msg agent">
                <div className="meta"><span className="dot thinking" style={{width:7,height:7}}></span><span className="who">{currentAgent.name}</span> | {settings.model} | now</div>
                <p><span>{streamContent}</span><span className="cursor">█</span></p>
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
                rows={2}
                placeholder={`Message ${currentAgent.name}.  (Ctrl+Enter to send | / for commands)`}
                aria-label="Message the agent"
                value={inputValue}
                onChange={e => setInputValue(e.target.value)}
                onKeyDown={e => { if (e.key === 'Enter' && e.ctrlKey) { e.preventDefault(); handleSend(); }}}
              />
              <div className="ci-row">
                <button className="btn icon ghost sm" title="Attach file" aria-label="Attach file">📎</button>
                <button className="btn icon ghost sm" title="Slash commands" aria-label="Slash commands">/</button>
                <button className="btn icon ghost sm" title="Insert code block" aria-label="Insert code block">{ }</button>
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
            onViewConfig={() => addToast({message: 'Viewing agent config - coming soon', type: 'info'})}
            onKillAgent={handleKillAgent}
          />
        )}
      </div>

      <BottomDrawer
        collapsed={drawerCollapsed}
        onToggleCollapse={() => setDrawerCollapsed(p => !p)}
      />

      {/* HITL approval dialog */}
      <Modal
        isOpen={showHitl}
        onClose={() => setShowHitl(false)}
        title="⚠ Human-in-the-loop required"
        accentColor="var(--status-human-wait)"
        actions={
          <>
            <span className="countdown">⏳ Auto-denying in {hitlCountdown}s</span>
            <button className="btn secondary" onClick={() => addToast({message: 'Opening diff in the drawer', type: 'info'})}>Review files</button>
            <button className="btn ghost" onClick={() => { setShowHitl(false); addToast({message:'Denied - agent will re-plan', type:'warning'}); }}>Deny</button>
            <button className="btn primary" onClick={() => { setShowHitl(false); addToast({message:'Approved - action executing', type:'success'}); }}>Approve</button>
          </>
        }
      >
        <div style={{fontSize:'13px',color:'var(--text-secondary)'}}>Agent <b style={{color:'var(--text-primary)'}}>{currentAgent.name}</b> wants to run a destructive action:</div>
        <div className="quote">delete_file | src/old_routes.rs</div>
        <div style={{fontSize:'11.5px',color:'var(--text-muted)'}}>Default-deny is on. No approval is remembered - every write asks again.</div>
      </Modal>

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

      {/* Demo launcher */}
      <div className="demo-pill" id="demoPill">
        <button className="btn secondary sm" id="demoBtn" onClick={() => document.getElementById('demoPill')?.classList.toggle('open')}>⚙ UI demos</button>
        <div className="demo-menu glass-overlay">
          <button className="btn" onClick={() => setShowHitl(true)}>⚠ HITL approval dialog</button>
          <button className="btn" onClick={() => setShowWizard(true)}>🎉 First-run wizard</button>
          <button className="btn" onClick={() => setShowPalette(true)}>⌘ Command palette</button>
          <button className="btn" onClick={() => { addToast({message:'Tool completed - write_file 0.9s', type:'success'}); addToast({message:'Session autosaved', type:'info'}); addToast({message:'Context window at 78%', type:'warning'}); addToast({message:'LLM connection lost - retrying in 5s', type:'error'}); }}>🔔 Toast notifications</button>
          <button className="btn" onClick={() => { setPhase('online'); addToast({message:'Agent Alpha hit an error | open Logs for details', type:'error'}); }}>⚠ Simulate agent error</button>
        </div>
      </div>
    </div>
  );
}

export default App;




