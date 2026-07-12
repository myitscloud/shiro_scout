import React, { useState, useCallback } from 'react';
import { useAppContext } from '../../context/AppContext';
import type { LlmRoleConfig } from '../../context/AppContext';
import { testLlmConnection } from '../../tauri-commands';
import styles from './LLMProviderSettings.module.css';

// ============================================================
// Types
// ============================================================

type RoleKey = 'chat' | 'utility' | 'embedding';
type TestStatus = 'idle' | 'loading' | 'connected' | 'failed';

interface RoleSection {
  key: RoleKey;
  label: string;
  icon: string;
  description: string;
}

const ROLE_SECTIONS: RoleSection[] = [
  { key: 'chat', label: 'Chat LLM', icon: '💬', description: 'Primary reasoning, tool use, and conversation' },
  { key: 'utility', label: 'Utility LLM', icon: '⚡', description: 'Summaries, memory queries, compression, filtering' },
  { key: 'embedding', label: 'Embedding LLM', icon: '🧠', description: 'Vector embeddings for memory and knowledge' },
];

const PROVIDERS = ['CLOUD', 'DeepSeek', 'OpenAI', 'Groq', 'Together', 'Ollama', 'LiteLLM'] as const;

const PROVIDER_MAP: Record<string, string> = {
  'CLOUD': 'deepseek',
  'DeepSeek': 'deepseek',
  'OpenAI': 'openai',
  'Groq': 'groq',
  'Together': 'together',
  'Ollama': 'ollama',
  'LiteLLM': 'litellm',
};

const REVERSE_PROVIDER_MAP: Record<string, string> = Object.fromEntries(
  Object.entries(PROVIDER_MAP).map(([k, v]) => [v, k])
);

// ============================================================
// Role config component
// ============================================================

interface RoleCardProps {
  section: RoleSection;
  config: LlmRoleConfig;
  onChange: (partial: Partial<LlmRoleConfig>) => void;
}

const RoleCard: React.FC<RoleCardProps> = ({ section, config, onChange }) => {
  const [showKey, setShowKey] = useState(false);
  const [testStatus, setTestStatus] = useState<TestStatus>('idle');
  const [testMsg, setTestMsg] = useState('');

  const handleTestConnection = useCallback(async () => {
    setTestStatus('loading');
    setTestMsg('');
    try {
      const result = await testLlmConnection({
        provider: config.provider,
        model: config.model,
        api_key: config.api_key || null,
      });
      if (result.success) {
        setTestStatus('connected');
        setTestMsg(result.latency_ms !== null ? `${result.latency_ms.toFixed(0)}ms` : 'Connected');
      } else {
        setTestStatus('failed');
        setTestMsg(result.error || 'Connection failed');
      }
    } catch (err) {
      setTestStatus('failed');
      setTestMsg(err instanceof Error ? err.message : 'Unknown error');
    }
  }, [config.provider, config.model, config.api_key]);

  const handleProviderChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const normalized = PROVIDER_MAP[e.target.value] || e.target.value.toLowerCase();
    onChange({ provider: normalized });
  };

  const selectedLabel = REVERSE_PROVIDER_MAP[config.provider] || config.provider.charAt(0).toUpperCase() + config.provider.slice(1);

  return (
    <div className={styles.roleCard} role="region" aria-label={`${section.label} configuration`}>
      <div className={styles.roleHeader}>
        <div className={styles.roleIcon} aria-hidden="true">{section.icon}</div>
        <div>
          <div className={styles.roleTitle}>{section.label}</div>
          <div style={{ fontSize: '11px', color: 'var(--text-muted)', marginTop: '2px' }}>{section.description}</div>
        </div>
      </div>

      <div className={styles.formGrid}>
        <div className={styles.field}>
          <label htmlFor={`${section.key}-provider`}>Provider</label>
          <select
            id={`${section.key}-provider`}
            className={styles.providerSelect}
            value={selectedLabel}
            onChange={handleProviderChange}
            aria-label={`${section.label} provider`}
          >
            {PROVIDERS.map(p => (
              <option key={p} value={p}>{p}</option>
            ))}
          </select>
        </div>

        <div className={styles.field}>
          <label htmlFor={`${section.key}-model`}>Model</label>
          <input
            id={`${section.key}-model`}
            className={styles.input}
            type="text"
            value={config.model}
            onChange={e => onChange({ model: e.target.value })}
            placeholder="e.g. deepseek-v4-flash"
            aria-label={`${section.label} model name`}
          />
        </div>

        <div className={`${styles.field} ${styles.fullWidth}`}>
          <label htmlFor={`${section.key}-apikey`}>API Key</label>
          <div className={styles.apiRow}>
            <input
              id={`${section.key}-apikey`}
              className={styles.input}
              type={showKey ? 'text' : 'password'}
              value={config.api_key}
              onChange={e => onChange({ api_key: e.target.value })}
              placeholder={config.provider === 'ollama' ? 'Local — no key needed' : 'sk-...'}
              aria-label={`${section.label} API key`}
            />
            <button
              className={`${styles.toggleBtn} ${showKey ? styles.visible : ''}`}
              onClick={() => setShowKey(v => !v)}
              aria-label={showKey ? 'Hide API key' : 'Show API key'}
              type="button"
            >
              {showKey ? '👁' : '👁‍🗨'}
            </button>
          </div>
        </div>
      </div>

      <div className={styles.testRow}>
        <button
          className={styles.testBtn}
          onClick={handleTestConnection}
          disabled={testStatus === 'loading'}
          type="button"
          aria-label={`Test ${section.label} connection`}
        >
          {testStatus === 'loading' && <span className={styles.spinner} />}
          {testStatus === 'loading' ? 'Testing...' : 'Test Connection'}
        </button>
        {testStatus === 'connected' && (
          <span className={`${styles.indicator} ${styles.indicatorOk}`} role="status">
            ✓ {testMsg}
          </span>
        )}
        {testStatus === 'failed' && (
          <span className={`${styles.indicator} ${styles.indicatorFail}`} role="alert">
            ✗ {testMsg}
          </span>
        )}
      </div>
    </div>
  );
};

// ============================================================
// Main component
// ============================================================

const LLMProviderSettings: React.FC = () => {
  const { llmConfig, updateLlmConfig } = useAppContext();

  const handleRoleChange = useCallback(
    (role: RoleKey, partial: Partial<LlmRoleConfig>) => {
      updateLlmConfig({
        [role]: { ...llmConfig[role], ...partial },
      });
    },
    [llmConfig, updateLlmConfig]
  );

  return (
    <div className={styles.container}>
      {ROLE_SECTIONS.map(section => (
        <RoleCard
          key={section.key}
          section={section}
          config={llmConfig[section.key]}
          onChange={partial => handleRoleChange(section.key, partial)}
        />
      ))}
    </div>
  );
};

LLMProviderSettings.displayName = 'LLMProviderSettings';
export default LLMProviderSettings;
