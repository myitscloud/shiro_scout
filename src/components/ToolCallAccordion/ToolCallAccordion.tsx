import React, { useState } from 'react';
import styles from './ToolCallAccordion.module.css';

export type ToolCallStatus = 'ok' | 'fail' | 'running';

export interface ToolCallAccordionProps {
  name: string;
  input?: string;
  output?: string;
  duration?: string;
  status: ToolCallStatus;
  error?: string;
  onRetry?: () => void;
}

const ToolCallAccordion: React.FC<ToolCallAccordionProps> = ({
  name,
  input,
  output,
  duration,
  status,
  error,
  onRetry,
}) => {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className={`${styles.tool} ${styles[status]} ${isOpen ? styles.open : ''}`}>
      <button
        className={styles['tool-head']}
        aria-expanded={isOpen}
        onClick={() => setIsOpen(!isOpen)}
      >
        <span className={styles.caret}>▶</span>
        <span>Tool call: </span>
        <span className={styles.tname}>{name}</span>
        <span style={{color: status === 'ok' ? 'var(--status-online)' : status === 'fail' ? 'var(--status-error)' : 'var(--accent-purple-glow)'}}>
          {status === 'ok' ? ' ✓' : status === 'fail' ? ' ✗' : ' ⚡'}
        </span>
        <span className={styles.tdur}>{duration || 'running'}</span>
      </button>
      <div className={styles['tool-body']}>
        {input && <><b>Input</b>&nbsp;&nbsp;{input}<br /></>}
        {output && <><b>Output</b>&nbsp;{output}<br /></>}
        {error && <div className={styles.terr}>{error}</div>}
        {status === 'running' && (
          <div>
            <div className={styles.tprog}><i></i></div>
            <span style={{color:'var(--text-muted)'}}>68% · applying patch inside sandbox</span>
          </div>
        )}
        {status === 'fail' && onRetry && (
          <div style={{marginTop:8}}>
            <button className="btn sm secondary" onClick={onRetry}>↻ Retry</button>
          </div>
        )}
      </div>
    </div>
  );
};

ToolCallAccordion.displayName = 'ToolCallAccordion';
export default ToolCallAccordion;
