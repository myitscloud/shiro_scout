import React from 'react';
import styles from './Navbar.module.css';

export interface NavbarProps {
  agentName?: string;
  agentStatus?: 'online' | 'thinking' | 'warn' | 'err' | 'off' | 'wait';
  agentPhase?: string;
  providerLabel?: string;
  sandboxLabel?: string;
  onSettingsClick?: () => void;
  onToggleRightPanel?: () => void;
}

const Navbar: React.FC<NavbarProps> = ({
  agentName = 'Orchestrator',
  agentStatus = 'online',
  providerLabel = 'gpt-4o',
  sandboxLabel = 'Sandbox',
  onSettingsClick,
  onToggleRightPanel,
}) => {

  return (
  <header className={styles.titlebar} data-tauri-drag-region>
    <div className={styles.brand} title="Aegis — home"><span className={styles.logo}>▲</span> Aegis</div>
    <div className={styles['tb-sep']}></div>
    <div className={styles['tb-group']}>
      <div className={styles.pill}>
        <span className={`${styles.dot} ${styles[agentStatus]}`} id="navDot"></span> <strong id="navAgent">{agentName}</strong>
      </div>
      <button className={styles.pill} title={`Provider: Ollama \u00b7 ${providerLabel} (local)`}><span className={styles.mono}>🤖 {providerLabel}</span></button>
      <button className={styles.pill} title="Local provider — no data leaves this machine" style={{color:'var(--status-online)'}}>🔒 <span>Local</span></button>
      <button className={styles.pill} title={`Sandbox: ${sandboxLabel} \u00b7 healthy`}>
        <svg width="13" height="13" viewBox="0 0 24 24" fill="currentColor" style={{color:'#4A9FE0'}}><path d="M4 11h2v2H4zm3 0h2v2H7zm3 0h2v2h-2zm-3-3h2v2H7zm3 0h2v2h-2zm0-3h2v2h-2zM2 14s.7 4.5 5.5 4.5c6.6 0 9.8-3.2 11-5 0 0 3 .4 3.5-1.5-1-.8-2.6-.6-2.6-.6s.2-1.5-1.4-2.4c-.9 1-.8 2.4-.8 2.4H2z"/></svg>
        <span className={`${styles.dot}`} style={{width:6,height:6}}></span> <span>Sandbox</span>
      </button>
    </div>
    <button className="btn icon ghost" id="rpToggle" title="Toggle agent details panel" aria-label="Toggle right panel" onClick={onToggleRightPanel}>▥</button>
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
