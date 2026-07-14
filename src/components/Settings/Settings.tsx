import React, { useState, useEffect, useCallback } from 'react';
import { useAppContext } from '../../context/AppContext';
import { saveLlmSettings } from '../../tauri-commands';
import { open } from '@tauri-apps/plugin-dialog';
import LLMProviderSettings from './LLMProviderSettings';
import styles from '../Overlay/Modal.module.css';

export interface SettingsProps {
  isOpen: boolean;
  onClose: () => void;
}

const SettingsView: React.FC<SettingsProps> = ({ isOpen, onClose }) => {
  const { settings, updateSettings, llmConfig, stopContainer, createContainer, startContainer, removeContainer, refreshDockerStatus } = useAppContext();

  const [theme, setTheme] = useState<'dark' | 'light'>(settings.theme);
  const [reduceMotion, setReduceMotion] = useState(settings.reduce_motion);
  const [provider, setProvider] = useState<'local' | 'cloud'>(settings.provider);
  const [model, setModel] = useState(settings.model);
  const [apiKey, setApiKey] = useState(settings.api_key);
  const [workspacePath, setWorkspacePath] = useState(settings.workspacePath);
  const [activeTab, setActiveTab] = useState<'general' | 'llm'>('general');
  const [restarting, setRestarting] = useState(false);

  // Sync local state when settings change from context
  useEffect(() => {
    setTheme(settings.theme);
    setReduceMotion(settings.reduce_motion);
    setProvider(settings.provider);
    setModel(settings.model);
    setApiKey(settings.api_key);
    setWorkspacePath(settings.workspacePath);
  }, [settings]);

  const handleSave = async () => {
    await updateSettings({
      theme,
      workspacePath,
      reduce_motion: reduceMotion,
      provider,
      model,
      api_key: apiKey,
    });

    // Also persist LLM provider config (including API keys) to disk
    try {
      await saveLlmSettings(llmConfig);
    } catch (e) {
      console.error('Failed to save LLM settings:', e);
    }

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
    setWorkspacePath(settings.workspacePath);
    onClose();
  };

  const handleBrowse = useCallback(async () => {
    const selected = await open({ directory: true, multiple: false, title: 'Select Workspace Folder' });
    if (selected) {
      setWorkspacePath(selected);
    }
  }, []);

  const handleApplyRestart = useCallback(async () => {
    setRestarting(true);
    try {
      // Persist workspace path first
      await updateSettings({
        theme,
        workspacePath,
        reduce_motion: reduceMotion,
        provider,
        model,
        api_key: apiKey,
      });

      // Stop existing sandbox if running
      const containerId = 'aegis-sandbox';
      try {
        console.log('[Settings] Stopping container:', containerId);
        await stopContainer(containerId);
        console.log('[Settings] Container stopped successfully');
        // Small delay to let container stop
        await new Promise(r => setTimeout(r, 500));
      } catch (stopErr) {
        console.warn('[Settings] StopContainer failed (may already be stopped):', stopErr);
      }

      // Remove the old container to avoid 409 Conflict on re-create
      try {
        console.log('[Settings] Removing container:', containerId);
        await removeContainer(containerId);
        console.log('[Settings] Container removed successfully');
      } catch (removeErr) {
        console.warn('[Settings] RemoveContainer failed (may not exist):', removeErr);
      }

      // Create new sandbox with updated workspace path
      console.log('[Settings] Creating container with workspace:', workspacePath);
      await createContainer({
        image: 'aegis-sandbox:latest',
        workspace_path: workspacePath || '',
        memory_mb: 2048,
        cpu_shares: 512,
        network_mode: 'none',
      });
      console.log('[Settings] Container created successfully');

      // Start it
      console.log('[Settings] Starting container:', containerId);
      await startContainer(containerId);
      console.log('[Settings] Container started successfully');

      // Refresh Docker status so UI updates
      await refreshDockerStatus();
      console.log('[Settings] Docker status refreshed');
    } catch (e) {
      console.error('[Settings] Failed to restart sandbox with new workspace path:', e);
    } finally {
      setRestarting(false);
    }
  }, [theme, workspacePath, reduceMotion, provider, model, apiKey, updateSettings, stopContainer, createContainer, startContainer, refreshDockerStatus]);

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
            <option>deepseek-v4-flash (default)</option>
            <option>gpt-4o</option>
            <option>llama3.3-70b (local)</option>
            <option>claude-sonnet</option>
          </select>
        </div>
        <div className={styles.field}>
          <label>Sandbox</label>

        </div>
        <div className={styles.field}>
          <label>📁 Workspace</label>
          <div style={{fontSize:'11.5px',color:'var(--text-muted)',marginBottom:'6px'}}>Path where the sandbox can access files</div>
          <div style={{display:'flex',gap:'6px',alignItems:'center'}}>
            <input
              className={styles.input}
              type="text"
              value={workspacePath}
              onChange={e => setWorkspacePath(e.target.value)}
              placeholder="C:\Projects"
              aria-label="Workspace folder path"
              style={{flex:1}}
            />
            <button
              className="btn secondary"
              onClick={handleBrowse}
              style={{padding:'5px 12px',fontSize:'12px',whiteSpace:'nowrap'}}
            >
              Browse…
            </button>
          </div>
          <div style={{display:'flex',gap:'8px',marginTop:'8px',alignItems:'center'}}>
            <button
              className="btn secondary"
              onClick={handleApplyRestart}
              disabled={restarting}
              style={{padding:'5px 12px',fontSize:'12px',cursor:restarting?'wait':'pointer'}}
            >
              {restarting ? 'Restarting…' : 'Apply & Restart Sandbox'}
            </button>
          </div>
          <div style={{fontSize:'11px',color:'var(--text-muted)',marginTop:'6px'}}>
            {workspacePath
              ? `Current: ${workspacePath}`
              : 'Current: No workspace set'}
          </div>
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