import React from 'react';
import styles from './Sidebar.module.css';

export interface AgentSlot {
  id: string;
  name: string;
  avatar: string;
  status: 'online' | 'off' | 'err' | 'pending_confirmation';
  phase: string;
  isThinking?: boolean;
  isActive?: boolean;
}

export interface SessionItem {
  id: string;
  title: string;
  group: string;
  isActive?: boolean;
}

export interface SidebarProps {
  agents?: AgentSlot[];
  sessions?: SessionItem[];
  onToggleRail?: () => void;
  onNewSession?: () => void;
  onAgentClick?: (id: string) => void;
  onSessionClick?: (id: string) => void;
}

const Sidebar: React.FC<SidebarProps> = ({
  agents = [
    { id: 'alpha', name: 'Alpha', avatar: 'α', status: 'online', phase: '⚡', isThinking: true, isActive: true },
    { id: 'beta', name: 'Beta', avatar: 'β', status: 'off', phase: '◉', isActive: false },
    { id: 'gamma', name: 'Gamma', avatar: 'γ', status: 'err', phase: '⚠', isActive: false },
  ],
  sessions = [
    { id: 's1', title: 'Refactor API routes', group: 'Today', isActive: true },
    { id: 's2', title: 'Debug WMI provider init', group: 'Today' },
    { id: 's3', title: 'Telemetry noise filter port', group: 'Yesterday' },
    { id: 's4', title: 'Sysmon config review', group: 'Yesterday' },
  ],
  onToggleRail,
  onNewSession,
  onAgentClick,
  onSessionClick,
}) => {
  const groupedSessions: Record<string, SessionItem[]> = {};
  sessions.forEach(s => {
    if (!groupedSessions[s.group]) groupedSessions[s.group] = [];
    groupedSessions[s.group].push(s);
  });

  return (
    <aside className={styles.sidebar} aria-label="Agents and sessions">
      <div className={styles['sb-section']}>
        <div className={styles['sb-label']}>Agents</div>
        {agents.map(a => (
          <div
            key={a.id}
            className={`${styles['agent-slot']} ${a.isActive ? styles.active : ''}`}
            data-agent={a.name}
            onClick={() => onAgentClick?.(a.id)}
            role="button"
            tabIndex={0}
            aria-label={`Agent ${a.name}, ${a.status}`}
          >
            <div className={`${styles.avatar} ${a.isThinking ? styles.thinking : ''}`} id={`av${a.id[0].toUpperCase()}`}>
              {a.avatar}
              <span className={`${styles.st} ${styles[a.status]}`}></span>
            </div>
            <span className={styles['agent-name']}>{a.name}</span>
            <span className={styles['agent-phase']} id={`ph${a.id[0].toUpperCase()}`}>{a.phase}</span>
          </div>
        ))}
      </div>

      <div className={styles.sessions} role="list" aria-label="Chat sessions">
        {Object.entries(groupedSessions).map(([group, items]) => (
          <React.Fragment key={group}>
            <div className={styles['sess-group']}>{group}</div>
            {items.map(s => (
              <div
                key={s.id}
                className={`${styles['sess-item']} ${s.isActive ? styles.active : ''}`}
                role="listitem"
                onClick={() => onSessionClick?.(s.id)}
              >
                <span className={styles.sd}></span> {s.title}
              </div>
            ))}
          </React.Fragment>
        ))}
      </div>

      <div className={styles['sb-footer']}>
        <button className={`btn secondary ${styles['new-session']}`} id="newSession" onClick={onNewSession}>＋ <span>New session</span></button>
        <button className="btn icon ghost" id="railToggle" title="Collapse sidebar" aria-label="Collapse sidebar" onClick={onToggleRail}>⇤</button>
      </div>
    </aside>
  );
};

Sidebar.displayName = 'Sidebar';
export default Sidebar;
