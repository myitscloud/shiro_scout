import React, { useState, useRef, useEffect } from 'react';
import styles from './Navbar.module.css';

export interface NavbarProps {
  agentName?: string;
  agentStatus?: 'online' | 'thinking' | 'warn' | 'err' | 'off' | 'wait';
  agentPhase?: string;
  providerLabel?: string;
  sandboxLabel?: string;
  sandboxStatus?: 'checking' | 'available' | 'unavailable' | 'error';
  sandboxVersion?: string | null;
  /** Transient boot-phase LED sequence. When set, overrides sandboxStatus dot display. */
  sandboxBootPhase?: 'booting-red' | 'booting-orange' | 'booting-yellow' | 'booting-blink' | null;
  onSettingsClick?: () => void;
  onToggleRightPanel?: () => void;
  onTogglePalette?: () => void;
  onStartSandbox?: () => void;
  onRestartSandbox?: () => void;
  onStopSandbox?: () => void;
}

const Navbar: React.FC<NavbarProps> = ({
  agentName = 'Orchestrator',
  agentStatus = 'online',
  providerLabel = 'gpt-4o',
  sandboxLabel = 'Sandbox',
  sandboxStatus = 'checking',
  sandboxVersion = null,
  sandboxBootPhase = null,
  onSettingsClick,
  onToggleRightPanel,
  onTogglePalette,
  onStartSandbox,
  onRestartSandbox,
  onStopSandbox,
}) => {
  const [showDropdown, setShowDropdown] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setShowDropdown(false);
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, []);

  const sandboxColors: Record<string, string> = {
    available: '#22c55e',
    checking: '#8b5cf6',
    unavailable: '#ef4444',
    error: '#ef4444',
  };
  const bootColors: Record<string, string> = {
    'booting-red': '#ef4444',
    'booting-orange': '#f97316',
    'booting-yellow': '#eab308',
    'booting-blink': '#22c55e',
  };
  const versionStr = sandboxVersion ? ` (v${sandboxVersion})` : '';
  const sandboxTitles: Record<string, string> = {
    available: `Sandbox: ${sandboxLabel}${versionStr} · running`,
    checking: 'Sandbox · checking...',
    unavailable: 'Sandbox · unavailable',
    error: 'Sandbox · error',
  };
  const bootTitles: Record<string, string> = {
    'booting-red': 'Sandbox · initializing...',
    'booting-orange': 'Sandbox · preparing...',
    'booting-yellow': 'Sandbox · almost ready...',
    'booting-blink': 'Sandbox · starting...',
  };
  const sandboxLabels: Record<string, string> = {
    available: 'Sandbox',
    checking: 'Checking…',
    unavailable: 'No sandbox',
    error: 'Error',
  };
  const bootLabels: Record<string, string> = {
    'booting-red': 'Init...',
    'booting-orange': 'Prep...',
    'booting-yellow': 'Ready...',
    'booting-blink': 'Start...',
  };
  // Boot phase overrides normal status when active
  const color = bootColors[sandboxBootPhase || ''] || sandboxColors[sandboxStatus] || '#ef4444';
  const title = bootTitles[sandboxBootPhase || ''] || sandboxTitles[sandboxStatus] || 'Sandbox';
  const label = bootLabels[sandboxBootPhase || ''] || sandboxLabels[sandboxStatus] || 'Sandbox';
  const isChecking = sandboxStatus === 'checking' && !sandboxBootPhase;
  // Dot class: use boot phase class if active, else fall through to normal dot styling via inline style
  const dotClass = sandboxBootPhase ? styles[sandboxBootPhase as keyof typeof styles] || '' : '';
  const showGlow = sandboxStatus === 'available' && !sandboxBootPhase;

  return (
  <header className={styles.titlebar} data-tauri-drag-region>
    <div className={styles.brand} title="Aegis — home"><span className={styles.logo}>▲</span> Aegis</div>
    <div className={styles['tb-sep']}></div>
    <div className={styles['tb-group']}>
      <div className={styles.pill}>
        <span className={`${styles.dot} ${styles[agentStatus]}`} id="navDot"></span> <strong id="navAgent">{agentName}</strong>
      </div>
      <button className={styles.pill} title={`Provider: Ollama · ${providerLabel} (local)`}><span className={styles.mono}>🤖 {providerLabel}</span></button>
      <button className={styles.pill} title="Local provider — no data leaves this machine" style={{color:'var(--status-online)'}}>🔒 <span>Local</span></button>
      <div className={styles.sandboxControl}>
        <button className={styles.pill} title={title}>
          <svg width="13" height="13" viewBox="0 0 24 24" fill="currentColor" style={{color}}>
            <path d="M4 11h2v2H4zm3 0h2v2H7zm3 0h2v2h-2zm-3-3h2v2H7zm3 0h2v2h-2zm0-3h2v2h-2zM2 14s.7 4.5 5.5 4.5c6.6 0 9.8-3.2 11-5 0 0 3 .4 3.5-1.5-1-.8-2.6-.6-2.6-.6s.2-1.5-1.4-2.4c-.9 1-.8 2.4-.8 2.4H2z"/>
          </svg>
          <span className={`${styles.dot} ${dotClass} ${isChecking ? styles.thinking : ''}`} style={{width:6,height:6,background:color,boxShadow:showGlow ? '0 0 8px rgba(34,197,94,.8)' : 'none'}}></span> <span>{label}</span>
        </button>
        <button className="btn icon ghost" onClick={() => setShowDropdown(!showDropdown)} title="Sandbox controls">▼</button>
        {showDropdown && (
          <div className={styles.dropdown} ref={dropdownRef}>
            <div className={styles.dropdownInfo}>
              <div>Container: aegis-sandbox</div>
              <div>Status: {sandboxStatus}</div>
              <div>Image: aegis-sandbox:latest</div>
              {sandboxVersion && <div>Version: v{sandboxVersion}</div>}
            </div>
            <div className={styles.dropdownDivider} />
            <button className={styles.dropdownAction} onClick={() => { onStartSandbox?.(); setShowDropdown(false); }}>
              ▶ Start
            </button>
            <button className={styles.dropdownAction} onClick={() => { onRestartSandbox?.(); setShowDropdown(false); }}>
              🔄 Restart
            </button>
            <button className={styles.dropdownAction} onClick={() => { onStopSandbox?.(); setShowDropdown(false); }}>
              ⏹ Stop
            </button>
          </div>
        )}
      </div>
    </div>
    <button className="btn icon ghost" id="rpToggle" title="Toggle agent details panel" aria-label="Toggle right panel" onClick={onToggleRightPanel}>▥</button>
    <button className="btn icon ghost" id="paletteBtn" title="Command palette (Ctrl+K)" aria-label="Open command palette" onClick={onTogglePalette}>?</button>
    <button className="btn icon ghost" id="settingsBtn" title="Settings (Ctrl+,)" aria-label="Open settings" onClick={onSettingsClick}>⚙</button>
    <div className={styles['win-controls']} aria-hidden="true">
      <button className={styles['win-btn']} title="Minimize">—</button>
      <button className={styles['win-btn']} title="Maximize">□</button>
      <button className={`${styles['win-btn']} ${styles.close}`} title="Close">✕</button>
    </div>
  </header>
  );
};

Navbar.displayName = 'Navbar';
export default Navbar;
