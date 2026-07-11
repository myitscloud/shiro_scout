import React from 'react';
import styles from './RightPanel.module.css';

export interface RecentTool {
  name: string;
  status: 'ok' | 'bad' | 'run';
  duration: string;
}

export interface RightPanelProps {
  agentName?: string;
  statusLabel?: string;
  model?: string;
  provider?: string;
  toolsEnabled?: number;
  sessionTime?: string;
  hitlMode?: string;
  tokensUsed?: number;
  tokenLimit?: number;
  recentTools?: RecentTool[];
  costSession?: string;
  costDetail?: string;
  onViewConfig?: () => void;
  onKillAgent?: () => void;
}

const RightPanel: React.FC<RightPanelProps> = ({
  agentName = 'Alpha',
  statusLabel = '● Active',
  model = 'gpt-4o',
  provider = 'Local · Ollama',
  toolsEnabled = 5,
  sessionTime = '4m 12s',
  hitlMode = 'Ask every write',
  tokensUsed = 12401,
  tokenLimit = 128000,
  recentTools = [
    { name: 'search_files', status: 'ok', duration: '0.3s' },
    { name: 'read_file ×3', status: 'ok', duration: '0.1s' },
    { name: 'cargo_check', status: 'bad', duration: '1.8s' },
    { name: 'write_file', status: 'run', duration: '…' },
  ],
  costSession = '$0.00 (local)',
  costDetail = 'disabled',
  onViewConfig,
  onKillAgent,
}) => {
  const tokenPct = Math.round((tokensUsed / tokenLimit) * 100);

  return (
    <aside className={styles.rightpanel} aria-label="Agent details">
      <div className={styles['rp-head']}><span className="dot online"></span> Agent details — <span id="rpName">{agentName}</span></div>
      <div className={styles['rp-sec']}>
        <div className={styles.kv}><span>Status</span><b style={{color:'var(--status-online)'}}>{statusLabel}</b></div>
        <div className={styles.kv}><span>Model</span><b>{model}</b></div>
        <div className={styles.kv}><span>Provider</span><b>{provider}</b></div>
        <div className={styles.kv}><span>Tools enabled</span><b>{toolsEnabled}</b></div>
        <div className={styles.kv}><span>Session time</span><b>{sessionTime}</b></div>
        <div className={styles.kv}><span>HITL mode</span><b>{hitlMode}</b></div>
      </div>
      <div className={styles['rp-sec']}>
        <h4>Context window</h4>
        <div className={styles.tokbar}><i style={{width: tokenPct + '%'}}></i></div>
        <div className={styles.kv}><span>{tokensUsed.toLocaleString()} / {tokenLimit.toLocaleString()} tokens</span><b>{tokenPct}%</b></div>
      </div>
      <div className={styles['rp-sec']}>
        <h4>Recent tools</h4>
        {recentTools.map((t, i) => (
          <div key={i} className={styles.rtool}>
            <span className={styles[t.status]}>{t.status === 'ok' ? '✓' : t.status === 'bad' ? '✗' : '⚡'}</span>
            <span className={styles.rn}>{t.name}</span>
            <span>{t.duration}</span>
          </div>
        ))}
      </div>
      <div className={styles['rp-sec']}>
        <h4>Cost estimate</h4>
        <div className={styles.kv}><span>This session</span><b>{costSession}</b></div>
        <div className={styles.kv}><span>Cloud fallback</span><b>{costDetail}</b></div>
      </div>
      <div className={styles['rp-actions']}>
        <button className="btn secondary" onClick={onViewConfig}>⚙ View agent config</button>
        <button className="btn danger" id="killBtn" onClick={onKillAgent}>■ Kill agent</button>
      </div>
    </aside>
  );
};

RightPanel.displayName = 'RightPanel';
export default RightPanel;
