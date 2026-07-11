import React from 'react';
import styles from './AgentCard.module.css';

export interface AgentCardProps {
  /** Agent identifier */
  agent: string;
  /** Current status text */
  status: string;
  /** Current agent phase */
  phase: string;
  /** Progress percentage (0-100) */
  progress: number;
  /** Human-readable last activity timestamp */
  lastActivity: string;
  /** Current session name */
  sessionName: string;
  /** Number of active tools */
  toolCount: number;
  /** Click handler for card */
  onClick?: () => void;
}

/**
 * Phase → status dot mapping per MSPEC-010 §5.2
 */
const PHASE_DOT_MAP: Record<string, string> = {
  idle: '◉',
  thinking: '◐',
  gathering_context: '◎',
  running_tool: '⚡',
  reviewing_output: '◉',
  error: '⚠',
  awaiting_human: '✋',
};

const PHASE_STATUS_CLASS: Record<string, string> = {
  idle: 'statusOnline',
  thinking: 'statusThinking',
  gathering_context: 'statusGathering',
  running_tool: 'statusRunningTool',
  reviewing_output: 'statusOnline',
  error: 'statusError',
  awaiting_human: 'statusAwaitingHuman',
};

const PROGRESS_PHASES = new Set(['thinking', 'gathering_context', 'running_tool', 'reviewing_output']);

const AgentCard: React.FC<AgentCardProps> = ({
  agent,
  status,
  phase,
  progress,
  lastActivity,
  sessionName,
  toolCount,
  onClick,
}) => {
  const dotClass = styles[PHASE_STATUS_CLASS[phase] ?? 'statusOnline'] ?? styles.statusOnline;
  const phaseIcon = PHASE_DOT_MAP[phase] ?? '◉';
  const showProgress = PROGRESS_PHASES.has(phase);
  const isSpinning = phase === 'reviewing_output' || phase === 'thinking';

  const progressClass = phase === 'running_tool' ? styles.progressExecuting : styles.progressThinking;

  return (
    <div
      className={styles.card}
      onClick={onClick}
      onKeyDown={(e) => {
        if (onClick && (e.key === 'Enter' || e.key === ' ')) {
          e.preventDefault();
          onClick();
        }
      }}
      role="button"
      tabIndex={0}
      aria-label={`Agent ${agent}: ${status}, phase: ${phase}`}
    >
      {/* Top row: status dot + name + badges */}
      <div className={styles.topRow}>
        <span
          className={`${styles.statusDot} ${dotClass}`}
          aria-hidden="true"
        />
        <span className={styles.agentName}>{agent}</span>
        <span className={styles.modelBadge}>{status}</span>
        {toolCount > 0 && (
          <span className={styles.toolCountBadge}>
            ⚡ {toolCount} tool{toolCount !== 1 ? 's' : ''}
          </span>
        )}
      </div>

      {/* Middle row: phase text + progress */}
      <div className={styles.middleRow}>
        <span className={styles.phaseText}>
          <span
            className={`${styles.phaseIcon} ${isSpinning ? styles.phaseSpinning : ''}`}
            aria-hidden="true"
          >
            {phaseIcon}
          </span>
          {' '}
          {phase.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())}
        </span>
        {showProgress && (
          <div className={styles.progressContainer} role="progressbar" aria-valuenow={progress} aria-valuemin={0} aria-valuemax={100}>
            <div
              className={`${styles.progressBar} ${progressClass}`}
              style={{ width: `${Math.min(100, Math.max(0, progress))}%` }}
            />
          </div>
        )}
      </div>

      {/* Bottom row: activity + session */}
      <div className={styles.bottomRow}>
        <span className={styles.lastActivity}>Last activity: {lastActivity}</span>
        <span className={styles.sessionName} title={sessionName}>
          Session: {sessionName}
        </span>
      </div>
    </div>
  );
};

AgentCard.displayName = 'AgentCard';

export default AgentCard;
