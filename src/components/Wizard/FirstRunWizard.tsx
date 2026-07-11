import React from 'react';
import styles from '../Overlay/Modal.module.css';

export interface FirstRunWizardProps {
  isOpen: boolean;
  onComplete: () => void;
  onSkip: () => void;
}

const FirstRunWizard: React.FC<FirstRunWizardProps> = ({ isOpen, onComplete, onSkip }) => {
  if (!isOpen) return null;

  return (
    <div className={`${styles.scrim} ${isOpen ? styles.open : ''}`} onClick={(e) => { if (e.target === e.currentTarget) onSkip(); }}>
      <div className={styles.modal} role="dialog" aria-modal="true" aria-label="First run setup">
        <h3>▲ Welcome to Aegis</h3>
        <div className={styles.steps}>
          <span className={`${styles.step} ${styles.done}`}>✓</span><span className={styles['step-line']}></span>
          <span className={`${styles.step} ${styles.cur}`}>2</span><span className={styles['step-line']}></span>
          <span className={styles.step}>3</span><span className={styles['step-line']}></span>
          <span className={styles.step}>4</span>
        </div>
        <div style={{fontFamily:'var(--font-head)',fontWeight:600,marginBottom:8}}>Step 2 · Docker check</div>
        <div className={styles['wiz-ok']}><span style={{color:'var(--status-online)'}}>✓</span> Docker Desktop 4.38 detected — engine running</div>
        <div className={styles['wiz-ok']}><span style={{color:'var(--status-online)'}}>✓</span> Sandbox image aegis/sbx:1.4 pulled (198 MB)</div>
        <div style={{fontSize:'12px',color:'var(--text-muted)',marginTop:6}}>Next: choose an LLM provider and model.</div>
        <div className={styles.foot}>
          <button className="btn ghost" onClick={onSkip}>Skip setup</button>
          <button className="btn secondary">← Back</button>
          <button className="btn primary" onClick={onComplete}>Continue →</button>
        </div>
      </div>
    </div>
  );
};

FirstRunWizard.displayName = 'FirstRunWizard';
export default FirstRunWizard;
