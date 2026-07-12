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
  onSessionDelete?: (id: string) => void;
}

const Sidebar: React.FC<SidebarProps> = ({
  agents = [],
  sessions = [],
  onToggleRail,
  onNewSession,
  onAgentClick,
  onSessionClick,
  onSessionDelete,
}) => {
  const groupedSessions: Record<string, SessionItem[]> = {};
  sessions.forEach(s => {
    if (!groupedSessions[s.group]) groupedSessions[s.group] = [];
    groupedSessions[s.group].push(s);
  });

  const orchestrator = agents.find(a => a.id === 'orchestrator');
  const specialists = agents.filter(a => a.id !== 'orchestrator' && a.status !== 'err');

  return (
    <aside className={styles.sidebar} aria-label="Agents and sessions">
      {/* Orchestrator section */}
      <div className={styles['sb-section']}>
        <div className={styles['sb-label']}>ShiroScout</div>
        {orchestrator && (
          <div
            key={orchestrator.id}
            className={`${styles['agent-slot']} ${orchestrator.isActive ? styles.active : ''}`}
            data-agent={orchestrator.name}
            onClick={() => onAgentClick?.(orchestrator.id)}
            role="button"
            tabIndex={0}
            aria-label={`Agent ${orchestrator.name}, ${orchestrator.status}`}
          >
            <div className={`${styles.avatar} ${orchestrator.isThinking ? styles.thinking : ''}`} id={`av${orchestrator.id[0].toUpperCase()}`}>
              {orchestrator.avatar}
              <span className={`${styles.st} ${styles[orchestrator.status]}`}></span>
            </div>
            <span className={styles['agent-name']}>{orchestrator.name}</span>
            <span className={styles['agent-phase']} id={`ph${orchestrator.id[0].toUpperCase()}`}>{orchestrator.phase}</span>
          </div>
        )}
      </div>

      {/* Specialists section */}
      {specialists.length > 0 && (
        <div className={styles['sb-section']}>
          <div className={styles['sb-label']}>Specialists</div>
          {specialists.map(a => (
            <div
              key={a.id}
              className={`${styles['agent-slot']} ${a.isActive ? styles.active : ''}`}
              data-agent={a.name}
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
      )}

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
                <button
                  className={styles.sessDel}
                  title="Delete session"
                  aria-label="Delete session"
                  onClick={(e) => {
                    e.stopPropagation();
                    onSessionDelete?.(s.id);
                  }}
                >
                  ×
                </button>
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
