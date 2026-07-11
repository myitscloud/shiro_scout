import React, { useState, useEffect } from 'react';
import { useAppContext } from '../../context/AppContext';
import LLMProviderSettings from './LLMProviderSettings';
import styles from '../Overlay/Modal.module.css';

export interface SettingsProps {
  isOpen: boolean;
  onClose: () => void;
}

const SettingsView: React.FC<SettingsProps> = ({ isOpen, onClose }) => {
  const { settings, updateSettings } = useAppContext();

  const [theme, setTheme] = useState<'dark' | 'light'>(settings.theme);
  const [reduceMotion, setReduceMotion] = useState(settings.reduce_motion);
  const [provider, setProvider] = useState<'local' | 'cloud'>(settings.provider);
  const [model, setModel] = useState(settings.model);
  const [apiKey, setApiKey] = useState(settings.api_key);
  const [sandboxOnLaunch, setSandboxOnLaunch] = useState(settings.sandbox_on_launch);
  const [mountWorkspace, setMountWorkspace] = useState(settings.mount_workspace);
  const [activeTab, setActiveTab] = useState<'general' | 'llm'>('general');

  // Sync local state when settings change from context
  useEffect(() => {
    setTheme(settings.theme);
    setReduceMotion(settings.reduce_motion);
    setProvider(settings.provider);
    setModel(settings.model);
    setApiKey(settings.api_key);
    setSandboxOnLaunch(settings.sandbox_on_launch);
    setMountWorkspace(settings.mount_workspace);
  }, [settings]);

  const handleSave = async () => {
    await updateSettings({
      theme,
      reduce_motion: reduceMotion,
      provider,
      model,
      api_key: apiKey,
      sandbox_on_launch: sandboxOnLaunch,
      mount_workspace: mountWorkspace,
    });

    // Apply theme and motion preferences immediately
    if (theme === 'light') {
      document.body.classList.add('light');
    } else {
      document.body.classList.remove('light');
    }
    document.body.classList.toggle('reduce-motion', reduceMotion);

    onClose();
  };

  const handleCancel = () => {
    // Reset local state to saved settings
    setTheme(settings.theme);
    setReduceMotion(settings.reduce_motion);
    setProvider(settings.provider);
    setModel(settings.model);
    setApiKey(settings.api_key);
    setSandboxOnLaunch(settings.sandbox_on_launch);
    setMountWorkspace(settings.mount_workspace);
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className={`${styles.scrim} ${isOpen ? styles.open : ''}`} onClick={(e) => { if (e.target === e.currentTarget) handleCancel(); }}>
      <div className={styles.modal} role="dialog" aria-modal="true" aria-labelledby="setTitle">
        <h3 id="setTitle">⚙ Settings</h3>
        <div role="tablist" aria-label="Settings tabs" style={{display:'flex',gap:'4px',marginBottom:'14px',borderBottom:'1px solid var(--border-glass)',paddingBottom:'10px'}}>
          <button
            role="tab"
            aria-selected={activeTab === 'general'}
            aria-controls="settings-general-panel"
            id="settings-tab-general"
            onClick={() => setActiveTab('general')}
            style={{
              padding:'6px 14px',borderRadius:'6px',border:'1px solid',
              background: activeTab === 'general' ? 'rgba(139,92,246,.2)' : 'transparent',
              borderColor: activeTab === 'general' ? 'rgba(139,92,246,.4)' : 'var(--border-glass-light)',
              color: activeTab === 'general' ? 'var(--accent-purple-glow)' : 'var(--text-secondary)',
              fontFamily:'var(--font-ui)',fontSize:'12.5px',cursor:'pointer'
            }}
          >General</button>
          <button
            role="tab"
            aria-selected={activeTab === 'llm'}
            aria-controls="settings-llm-panel"
            id="settings-tab-llm"
            onClick={() => setActiveTab('llm')}
            style={{
              padding:'6px 14px',borderRadius:'6px',border:'1px solid',
              background: activeTab === 'llm' ? 'rgba(139,92,246,.2)' : 'transparent',
              borderColor: activeTab === 'llm' ? 'rgba(139,92,246,.4)' : 'var(--border-glass-light)',
              color: activeTab === 'llm' ? 'var(--accent-purple-glow)' : 'var(--text-secondary)',
              fontFamily:'var(--font-ui)',fontSize:'12.5px',cursor:'pointer'
            }}
          >LLM Providers</button>
        </div>
        {activeTab === 'general' && (
        <>
        <div className={styles.field}>
          <label>Appearance</label>
          <div className={styles.seg}>
            <button className={theme === 'dark' ? styles.active : ''} onClick={() => setTheme('dark')}>Dark (default)</button>
            <button className={theme === 'light' ? styles.active : ''} onClick={() => { setTheme('light'); document.body.classList.toggle('light', true); }}>Light | high contrast</button>
          </div>
        </div>
        <div className={styles.field}>
          <label>Motion</label>
          <label className={styles.check}>
            <input type="checkbox" checked={reduceMotion} onChange={e => { setReduceMotion(e.target.checked); document.body.classList.toggle('reduce-motion', e.target.checked); }} />
            Reduce motion - static indicators only
          </label>
        </div>
        <div className={styles.field}>
          <label>LLM provider</label>
          <div className={styles.seg}>
            <button className={provider === 'local' ? styles.active : ''} onClick={() => setProvider('local')}>?? Local</button>
            <button className={provider === 'cloud' ? styles.active : ''} onClick={() => setProvider('cloud')}>? Cloud</button>
          </div>
        </div>
        <div className={styles.field}>
          <label>API key <span style={{color:'var(--text-muted)'}}>(cloud only - stored in OS keyring)</span></label>
          <input className={styles.input} type="password" value={apiKey} onChange={e => setApiKey(e.target.value)} aria-label="API key" />
        </div>
        <div className={styles.field}>
          <label>Model</label>
          <select className={styles.input} value={model} onChange={e => setModel(e.target.value)}>
            <option>gpt-4o</option>
            <option>llama3.3-70b (local)</option>
            <option>deepseek-v3</option>
            <option>claude-sonnet</option>
          </select>
        </div>
        <div className={styles.field}>
          <label>Sandbox</label>
          <label className={styles.check}><input type="checkbox" checked={sandboxOnLaunch} onChange={e => setSandboxOnLaunch(e.target.checked)} /> Start Docker sandbox on launch</label>
          <label className={styles.check}><input type="checkbox" checked={mountWorkspace} onChange={e => setMountWorkspace(e.target.checked)} /> Mount /workspace read-write</label>
        </div>
        <div className={styles.foot}>
          <button className="btn ghost" onClick={handleCancel}>Cancel</button>
          <button className="btn primary" onClick={handleSave}>Save changes</button>
        </div>
        </>
        )}
        {activeTab === 'llm' && (
          <LLMProviderSettings />
        )}
      </div>
    </div>
  );
};

SettingsView.displayName = 'SettingsView';
export default SettingsView;
