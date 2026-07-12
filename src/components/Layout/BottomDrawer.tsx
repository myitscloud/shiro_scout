import React, { useState } from 'react';
import styles from './BottomDrawer.module.css';
import UsageMetrics from '../UsageMetrics/UsageMetrics';
import type { UsageMetricsProps } from '../UsageMetrics/UsageMetrics';

export type DrawerTab = 'logs' | 'term' | 'tele' | 'token-usage';

export interface LogEntry {
  ts: string;
  level: 'info' | 'tool' | 'warn' | 'err' | 'ok';
  source: string;
  message: string;
}

export interface TelemetryStat {
  value: string;
  label: string;
}

export interface BarItem {
  label: string;
  height: number;
}

export interface BottomDrawerProps {
  collapsed?: boolean;
  onToggleCollapse?: () => void;
  logs?: LogEntry[];
  telemetryStats?: TelemetryStat[];
  bars?: BarItem[];
  tokenUsage?: UsageMetricsProps['tokenUsage'] | null;
}

const LOG_DEFAULTS: LogEntry[] = [];

const BottomDrawer: React.FC<BottomDrawerProps> = ({
  collapsed = false,
  onToggleCollapse,
  logs = LOG_DEFAULTS,
  telemetryStats = [] as TelemetryStat[],
  bars = [] as BarItem[],
  tokenUsage = null,
}) => {
  const [activeTab, setActiveTab] = useState<DrawerTab>('logs');
  const [filterLevel, setFilterLevel] = useState<string>('all');
  const [filterText, setFilterText] = useState('');

  const filteredLogs = logs.filter(l => {
    if (filterLevel !== 'all' && l.level !== filterLevel) return false;
    if (filterText && !l.message.toLowerCase().includes(filterText.toLowerCase())) return false;
    return true;
  });

  return (
    <section className={`${styles.drawer} ${collapsed ? styles.collapsed : ''}`} aria-label="Logs, terminal and telemetry">
      <div className={styles.drawerBar}>
        <button className={`${styles.tab} ${activeTab === 'logs' ? styles.active : ''}`} data-pane="logs" onClick={() => setActiveTab('logs')}>Logs</button>
        <button className={`${styles.tab} ${activeTab === 'term' ? styles.active : ''}`} data-pane="term" onClick={() => setActiveTab('term')}>Terminal <span className={styles.tdot} title="activity"></span></button>
        <button className={`${styles.tab} ${activeTab === 'tele' ? styles.active : ''}`} data-pane="tele" onClick={() => setActiveTab('tele')}>Telemetry</button>
        <button className={`${styles.tab} ${activeTab === 'token-usage' ? styles.active : ''}`} data-pane="token-usage" onClick={() => setActiveTab('token-usage')}>Token Usage</button>
        <div className={styles.drawerRight}>
          <kbd>Ctrl</kbd><kbd>`</kbd>
          <button className="btn icon ghost sm" id="drawerToggle" title="Collapse drawer" aria-label="Toggle drawer" onClick={onToggleCollapse}>{collapsed ? '▴' : '▾'}</button>
        </div>
      </div>
      {!collapsed && (
        <div className={styles.drawerBody}>
          <div className={`${styles.pane} ${activeTab === 'logs' ? styles.active : ''}`} id="pane-logs">
            <div className={styles.logFilter}>
              <input
                className={styles.input}
                style={{height:'22px',width:'190px',fontSize:'11px'}}
                placeholder="Filter logs…"
                aria-label="Filter logs"
                value={filterText}
                onChange={e => setFilterText(e.target.value)}
              />
              {['all','info','tool','warn','err'].map(lv => (
                <button
                  key={lv}
                  className={`${styles.chip} ${filterLevel === lv ? styles.active : ''}`}
                  data-lv={lv}
                  onClick={() => setFilterLevel(lv)}
                >{lv === 'all' ? 'ALL' : lv.toUpperCase()}</button>
              ))}
            </div>
            <div id="logList">
              {filteredLogs.map((l, i) => (
                <div key={i} className={styles.logline} data-lv={l.level}>
                  <span className={styles.ts}>{l.ts}</span>  <span className={`${styles.lv} ${styles[l.level]}`}>{l.level.toUpperCase().padEnd(4)}</span> {l.source}   {l.message}
                </div>
              ))}
            </div>
          </div>
          <div className={`${styles.pane} ${styles.term} ${activeTab === 'term' ? styles.active : ''}`} id="pane-term">
            <span style={{color:'var(--text-muted)',fontStyle:'italic',fontSize:'12px',padding:'8px 0',display:'block'}}>No active terminal session — start a conversation to see terminal output</span>
          </div>
          <div className={`${styles.pane} ${activeTab === 'tele' ? styles.active : ''}`} id="pane-tele">
            <div className={styles.telegrid}>
              {telemetryStats.map((s, i) => (
                <div key={i} className={styles.stat}>
                  <div className={styles.sv}>{s.value}</div>
                  <div className={styles.sl}>{s.label}</div>
                </div>
              ))}
            </div>
            <div style={{fontSize:'10.5px',color:'var(--text-muted)',letterSpacing:'.08em',textTransform:'uppercase',marginBottom:'4px'}}>Tool duration (s)</div>
            <div className={styles.bars}>
              {bars.map((b, i) => (
                <span key={i}><i style={{height: b.height + 'px', display:'block'}}></i><div className={styles.lbl}>{b.label}</div></span>
              ))}
            </div>
          </div>
          <div className={`${styles.pane} ${activeTab === 'token-usage' ? styles.active : ''}`} id="pane-token-usage" role="tabpanel" aria-label="Token usage breakdown">
            {tokenUsage ? (
              <UsageMetrics tokenUsage={tokenUsage} />
            ) : (
              <p style={{color:'var(--text-muted)',fontStyle:'italic',padding:'12px 0',textAlign:'center',fontSize:'12px'}}>
                No usage data yet — start a conversation to see token usage
              </p>
            )}
          </div>
        </div>
      )}
    </section>
  );
};

BottomDrawer.displayName = 'BottomDrawer';
export default BottomDrawer;
