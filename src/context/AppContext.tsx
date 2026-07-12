import { createContext, useContext, useState, useEffect, useCallback, useRef, ReactNode, Dispatch, SetStateAction } from 'react';
import { listen } from '@tauri-apps/api/event';
import {
  checkDockerDaemon,
  type AppSettings,
  type SandboxConfig,
  type SandboxCreateResult,
  type TokenUsageEntry,
  type HITLEvent,
  type AgentStatusPayload,
  type LlmSettings,
  DEFAULT_SETTINGS,
  DEFAULT_LLM_SETTINGS,
  loadSettings as loadSettingsFromBackend,
  saveSettings as saveSettingsToBackend,
  getLlmSettings,
  createSandbox,
  startSandbox,
  stopSandbox,
  removeSandbox,
  listenForHITLRequest
} from '../tauri-commands';

// ============================================================
// Token Usage & Cost Estimation types (Wave 6.6)
// ============================================================

export type TokenUsageRole = 'chat' | 'utility' | 'embedding';

export interface TokenUsageRoleData {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
  cost_estimate: number;
  model?: string;
}

export interface TokenUsageState {
  byRole: Record<TokenUsageRole, TokenUsageRoleData>;
  sessionTotal: TokenUsageRoleData;
}

export function createEmptyTokenUsage(): TokenUsageState {
  const emptyRole: TokenUsageRoleData = { prompt_tokens: 0, completion_tokens: 0, total_tokens: 0, cost_estimate: 0 };
  return {
    byRole: { chat: { ...emptyRole }, utility: { ...emptyRole }, embedding: { ...emptyRole } },
    sessionTotal: { ...emptyRole },
  };
}

// ============================================================
// Agent & Session data structures
// ============================================================

export interface AgentInfo {
  id: string;
  name: string;
  avatar: string;
  status: 'online' | 'off' | 'err' | 'pending_confirmation';
  phase: string;
  isThinking?: boolean;
}

export interface SessionInfo {
  id: string;
  title: string;
  group: string;
}

export interface DockerInfo {
  status: 'checking' | 'available' | 'unavailable' | 'error';
  version: string | null;
  error: string | null;
  containers: SandboxCreateResult[];
}

// ============================================================
// Chat Message data structure
// ============================================================

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
  /** Non-null when an error occurred during streaming or generation. */
  error?: string | null;
}

// ============================================================
// LLM Configuration
// ============================================================

export interface LlmRoleConfig {
  provider: string;
  model: string;
  api_key: string;
}

export interface LlmConfig {
  chat: LlmRoleConfig;
  utility: LlmRoleConfig;
  embedding: LlmRoleConfig;
}

export const DEFAULT_LLM_CONFIG: LlmConfig = {
  chat: { provider: 'deepseek', model: 'deepseek-v4-flash', api_key: '' },
  utility: { provider: 'deepseek', model: 'deepseek-v4-flash', api_key: '' },
  embedding: { provider: 'deepseek', model: 'deepseek-v4-flash', api_key: '' },
};

// ============================================================
// Message ID generator (simple counter, not crypto — safe for UI keys)
// ============================================================

let nextMessageId = 1;
function generateMessageId(): string {
  return `msg-${Date.now()}-${nextMessageId++}`;
}

// ============================================================
// Context type
// ============================================================

interface AppContextValue {
  // Docker daemon state
  dockerInfo: DockerInfo;
  refreshDockerStatus: () => Promise<void>;

  // Agents
  agents: AgentInfo[];
  activeAgentId: string;
  setActiveAgentId: (id: string) => void;
  updateAgentStatus: (id: string, status: AgentInfo['status']) => void;

  // Sessions
  sessions: SessionInfo[];
  activeSessionId: string;
  setActiveSessionId: (id: string) => void;
  addSession: (session: SessionInfo) => void;
  removeSession: (id: string) => void;

  // Settings (persisted)
  settings: AppSettings;
  updateSettings: (partial: Partial<AppSettings>) => Promise<void>;
  loadSettings: () => Promise<void>;

  // LLM Provider Config
  llmConfig: LlmConfig;
  updateLlmConfig: (partial: Partial<LlmConfig>) => void;

  // Container lifecycle
  createContainer: (config: SandboxConfig) => Promise<SandboxCreateResult>;
  startContainer: (id: string) => Promise<void>;
  stopContainer: (id: string) => Promise<void>;
  removeContainer: (id: string) => Promise<void>;

  // UI state
  showRightPanel: boolean;
  setShowRightPanel: Dispatch<SetStateAction<boolean>>;
  drawerCollapsed: boolean;
  setDrawerCollapsed: Dispatch<SetStateAction<boolean>>;

  // ============================================================
  // Token Usage & Cost Estimation (Wave 6.6)
  // ============================================================

  /** Per-role aggregated token usage for the current session. */
  tokenUsage: TokenUsageState;
  /** Update token usage with a new entry from the Rust `token-usage` event. */
  updateTokenUsage: (entry: TokenUsageEntry) => void;
  /** Reset token usage for a new session. */
  resetTokenUsage: () => void;

  // ============================================================
  // Streaming — conversation & real-time token state
  // ============================================================

  /** All messages in the active conversation. */
  messages: ChatMessage[];
  /** Text being accumulated token-by-token from the current stream. */
  streamingMessage: string;
  /** Whether the assistant is currently streaming tokens. */
  isStreaming: boolean;
  /** Error from the last stream attempt (null = no error / resolved). */
  streamError: string | null;

  /**
   * Start a new assistant response stream.
   * Clears `streamError` and `streamingMessage`, sets `isStreaming` to true.
   * The caller should attach a Tauri `listen('llm-token', ...)` handler
   * that calls `appendToken(token, role)` and `finalizeStream()` on done.
   */
  startStream: () => void;

  /** Append a token fragment to the current streaming message. */
  appendToken: (token: string) => void;

  /**
   * Finalize the current stream: move `streamingMessage` into `messages` as
   * a completed assistant message, reset streaming state.
   */
  finalizeStream: () => void;

  /**
   * Abort the current stream without finalizing.
   * Preserves partial text in the message for error display.
   */
  abortStream: () => void;

  /** Add a user message to the conversation. */
  addUserMessage: (content: string) => void;

  /** Add a system message (e.g. status/error info) to the conversation. */
  addSystemMessage: (content: string) => void;

  /** Clear all messages and streaming state. */
  clearMessages: () => void;

  /** The current pending HITL confirmation request, or null if none. */
  currentPendingHITL: HITLEvent | null;
  /** Set the pending HITL confirmation request. */
  setCurrentPendingHITL: (event: HITLEvent | null) => void;
}

const AppContext = createContext<AppContextValue | null>(null);

export function useAppContext(): AppContextValue {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error('useAppContext must be used within AppProvider');
  return ctx;
}

// ============================================================
// Default agents and sessions
// ============================================================

const DEFAULT_AGENTS: AgentInfo[] = [
  { id: 'orchestrator', name: 'ShiroScout', avatar: '\u26a1', status: 'online', phase: '\u25cf', isThinking: false },
  { id: 'architect', name: 'Windows Architect', avatar: '\u2699', status: 'off', phase: '\u25cf', isThinking: false },
  { id: 'frontend', name: 'Frontend Engineer', avatar: '\u2728', status: 'off', phase: '\u25cf' },
  { id: 'security', name: 'Security Engineer', avatar: '\u274c', status: 'off', phase: '\u25cf' },
  { id: 'qa', name: 'QA Engineer', avatar: '\u2661', status: 'off', phase: '\u25cf' },
  { id: 'docs', name: 'Docs Engineer', avatar: '\u2714', status: 'off', phase: '\u25cf' },
  { id: 'devops', name: 'Release/DevOps', avatar: '\u267b', status: 'off', phase: '\u25cf' },
  { id: 'reviewer', name: 'Code Reviewer', avatar: '\u272f', status: 'off', phase: '\u25cf' },
];

export const DEFAULT_SESSIONS: SessionInfo[] = [];

// ============================================================
// Provider component
// ============================================================

interface AppProviderProps {
  children: ReactNode;
}

export function AppProvider({ children }: AppProviderProps) {
  const [dockerInfo, setDockerInfo] = useState<DockerInfo>({
    status: 'checking',
    version: null,
    error: null,
    containers: [],
  });

  const [agents, setAgents] = useState<AgentInfo[]>(DEFAULT_AGENTS);
  const [activeAgentId, setActiveAgentId] = useState('alpha');

  const [sessions, setSessions] = useState<SessionInfo[]>(DEFAULT_SESSIONS);
  const [activeSessionId, setActiveSessionId] = useState('s1');

  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);

  const [showRightPanel, setShowRightPanel] = useState(true);
  const [drawerCollapsed, setDrawerCollapsed] = useState(false);

  const [llmConfig, setLlmConfig] = useState<LlmConfig>(DEFAULT_LLM_CONFIG);

  const [currentPendingHITL, setCurrentPendingHITL] = useState<HITLEvent | null>(null);

  // ============================================================
  // Token Usage & Cost Estimation (Wave 6.6)
  // ============================================================

  const [tokenUsage, setTokenUsage] = useState<TokenUsageState>(createEmptyTokenUsage);

  const updateTokenUsage = useCallback((entry: TokenUsageEntry) => {
    setTokenUsage(prev => {
      const role = entry.role as TokenUsageRole;
      const roleData = prev.byRole[role];
      if (!roleData) {
        // Unknown role — use chat as fallback
        const chatData = prev.byRole.chat;
        return {
          ...prev,
          byRole: {
            ...prev.byRole,
            chat: {
              prompt_tokens: chatData.prompt_tokens + entry.prompt_tokens,
              completion_tokens: chatData.completion_tokens + entry.completion_tokens,
              total_tokens: chatData.total_tokens + entry.total_tokens,
              cost_estimate: chatData.cost_estimate + entry.cost_estimate,
              model: entry.model || chatData.model,
            },
          },
          sessionTotal: {
            prompt_tokens: prev.sessionTotal.prompt_tokens + entry.prompt_tokens,
            completion_tokens: prev.sessionTotal.completion_tokens + entry.completion_tokens,
            total_tokens: prev.sessionTotal.total_tokens + entry.total_tokens,
            cost_estimate: prev.sessionTotal.cost_estimate + entry.cost_estimate,
          },
        };
      }
      return {
        ...prev,
        byRole: {
          ...prev.byRole,
          [role]: {
            prompt_tokens: roleData.prompt_tokens + entry.prompt_tokens,
            completion_tokens: roleData.completion_tokens + entry.completion_tokens,
            total_tokens: roleData.total_tokens + entry.total_tokens,
            cost_estimate: roleData.cost_estimate + entry.cost_estimate,
            model: entry.model || roleData.model,
          },
        },
        sessionTotal: {
          prompt_tokens: prev.sessionTotal.prompt_tokens + entry.prompt_tokens,
          completion_tokens: prev.sessionTotal.completion_tokens + entry.completion_tokens,
          total_tokens: prev.sessionTotal.total_tokens + entry.total_tokens,
          cost_estimate: prev.sessionTotal.cost_estimate + entry.cost_estimate,
        },
      };
    });
  }, []);

  const resetTokenUsage = useCallback(() => {
    setTokenUsage(createEmptyTokenUsage());
  }, []);

  // ============================================================
  // Streaming state
  // ============================================================

  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [streamingMessage, setStreamingMessage] = useState('');
  const [isStreaming, setIsStreaming] = useState(false);
  const [streamError, setStreamError] = useState<string | null>(null);
  // Track the message ID of the in-progress assistant message for finalization
  const activeStreamMsgIdRef = useRef<string | null>(null);
  const accumulatedRef = useRef('');
  const isMountedRef = useRef(true);

  useEffect(() => {
    isMountedRef.current = true;
    return () => {
      isMountedRef.current = false;
    };
  }, []);

  const updateLlmConfig = useCallback((partial: Partial<LlmConfig>) => {
    setLlmConfig(prev => ({ ...prev, ...partial }));
  }, []);

  // Refresh Docker daemon status
  const refreshDockerStatus = useCallback(async () => {
    try {
      const status = await checkDockerDaemon();
      setDockerInfo({
        status: status.available ? 'available' : 'unavailable',
        version: status.version,
        error: status.error,
        containers: [],
      });
      // Update agent statuses based on Docker availability
      if (status.available) {
        setAgents(prev => prev.map(a =>
          a.status === 'err' ? { ...a, status: 'online' as const } : a
        ));
      }
    } catch (err) {
      setDockerInfo({
        status: 'error',
        version: null,
        error: String(err),
        containers: [],
      });
    }
  }, []);

  // Update a single agent's status
  const updateAgentStatus = useCallback((id: string, status: AgentInfo['status']) => {
    setAgents(prev => prev.map(a => a.id === id ? { ...a, status } : a));
  }, []);

  // Add a new session
  const addSession = useCallback((session: SessionInfo) => {
    setSessions(prev => [session, ...prev]);
    setActiveSessionId(session.id);
  }, []);

  const removeSession = useCallback((id: string) => {
    setSessions(prev => prev.filter(s => s.id !== id));
    setActiveSessionId(prev => prev === id ? (sessions.find(s => s.id !== id)?.id ?? '') : prev);
  }, [sessions]);

  // Load settings from backend
  const loadSettings = useCallback(async () => {
    try {
      const saved = await loadSettingsFromBackend();
      if (saved) {
        setSettings(saved);
      }
    } catch {
      // Use defaults
    }
    // Also restore LLM provider config (provider, model, API keys) from disk
    try {
      const llmSaved = await getLlmSettings();
      setLlmConfig({
        chat: {
          provider: llmSaved.chat.provider,
          model: llmSaved.chat.model,
          api_key: llmSaved.chat.api_key || '',
        },
        utility: {
          provider: llmSaved.utility.provider,
          model: llmSaved.utility.model,
          api_key: llmSaved.utility.api_key || '',
        },
        embedding: {
          provider: llmSaved.embedding.provider,
          model: llmSaved.embedding.model,
          api_key: llmSaved.embedding.api_key || '',
        },
      });
    } catch {
      // Use defaults
    }
  }, []);

  // Update and persist settings
  const updateSettings = useCallback(async (partial: Partial<AppSettings>) => {
    const newSettings = { ...settings, ...partial };
    setSettings(newSettings);
    try {
      await saveSettingsToBackend(newSettings);
    } catch {
      // Failed to persist - still use in-memory
    }
  }, [settings]);

  // Container lifecycle
  const createContainer = useCallback(async (config: SandboxConfig): Promise<SandboxCreateResult> => {
    const result = await createSandbox(config);
    setDockerInfo(prev => ({
      ...prev,
      containers: [...prev.containers, result],
    }));
    return result;
  }, []);

  const startContainer = useCallback(async (id: string): Promise<void> => {
    await startSandbox(id);
  }, []);

  const stopContainer = useCallback(async (id: string): Promise<void> => {
    await stopSandbox(id);
  }, []);

  const removeContainer = useCallback(async (id: string): Promise<void> => {
    await removeSandbox(id);
    setDockerInfo(prev => ({
      ...prev,
      containers: prev.containers.filter(c => c.container_id !== id),
    }));
  }, []);

  // ============================================================
  // Streaming actions
  // ============================================================

  const startStream = useCallback(() => {
    if (!isMountedRef.current) return;
    setStreamError(null);
    setStreamingMessage('');
    accumulatedRef.current = '';
    setIsStreaming(true);
    const msgId = generateMessageId();
    activeStreamMsgIdRef.current = msgId;
  }, []);

  const appendToken = useCallback((token: string) => {
    if (!isMountedRef.current) return;
    accumulatedRef.current += token;
    setStreamingMessage(accumulatedRef.current);
  }, []);

  const finalizeStream = useCallback(() => {
    if (!isMountedRef.current) return;
    const msgId = activeStreamMsgIdRef.current;
    activeStreamMsgIdRef.current = null;
    const content = accumulatedRef.current;
    accumulatedRef.current = '';

    if (content) {
      setMessages(prev => [
        ...prev,
        {
          id: msgId ?? generateMessageId(),
          role: 'assistant',
          content,
          timestamp: new Date().toISOString(),
          error: null,
        },
      ]);
    }

    setIsStreaming(false);
    setStreamingMessage('');
  }, []);

  const abortStream = useCallback(() => {
    if (!isMountedRef.current) return;
    const msgId = activeStreamMsgIdRef.current;
    activeStreamMsgIdRef.current = null;

    const partialContent = accumulatedRef.current;
    accumulatedRef.current = '';
    if (partialContent && msgId) {
      setMessages(prev => [
        ...prev,
        {
          id: msgId,
          role: 'assistant',
          content: partialContent,
          timestamp: new Date().toISOString(),
          error: 'Stream disconnected — response may be truncated',
        },
      ]);
    }

    setIsStreaming(false);
    setStreamingMessage('');
    setStreamError('Stream disconnected — partial content preserved');
  }, []);

  const addUserMessage = useCallback((content: string) => {
    if (!isMountedRef.current) return;
    const newMsg: ChatMessage = {
      id: generateMessageId(),
      role: 'user',
      content,
      timestamp: new Date().toISOString(),
    };
    setMessages(prev => [...prev, newMsg]);
  }, []);

  const addSystemMessage = useCallback((content: string) => {
    if (!isMountedRef.current) return;
    const newMsg: ChatMessage = {
      id: generateMessageId(),
      role: 'system',
      content,
      timestamp: new Date().toISOString(),
    };
    setMessages(prev => [...prev, newMsg]);
  }, []);

  const clearMessages = useCallback(() => {
    activeStreamMsgIdRef.current = null;
    setIsStreaming(false);
    setStreamingMessage('');
    setStreamError(null);
    setMessages([]);
  }, []);

  // Ref is no longer needed — accumulatedRef handles streaming accumulation directly

  // Check Docker on mount and load settings
  useEffect(() => {
    refreshDockerStatus();
    loadSettings();
  }, [refreshDockerStatus, loadSettings]);

  // ============================================================
  // Token usage event listener (F5: cleanup on unmount)
  // ============================================================
  useEffect(() => {
    const unlisten = listen<TokenUsageEntry>('token-usage', (event) => {
      updateTokenUsage(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [updateTokenUsage]);

 
  // ============================================================
  // LLM token streaming event listener
  // ============================================================
  useEffect(() => {
    const unlisten = listen<{ token: string; done: boolean }>('llm-token', (event) => {
      if (event.payload.done) {
        finalizeStream();
      } else if (event.payload.token) {
        appendToken(event.payload.token);
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [appendToken, finalizeStream]);
 // ============================================================
  // HITL event listener — Wave 7.1
  // ============================================================
  useEffect(() => {
    const unlisten = listenForHITLRequest((event) => {
      setCurrentPendingHITL(event);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // ============================================================
  // Agent status event listener — sidebar LED indicators
  // ============================================================
  useEffect(() => {
    const unlisten = listen<AgentStatusPayload>('agent-status', (event) => {
      updateAgentStatus(event.payload.agent_id, event.payload.status);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [updateAgentStatus]);

  const value: AppContextValue = {
    dockerInfo,
    refreshDockerStatus,
    agents,
    activeAgentId,
    setActiveAgentId,
    updateAgentStatus,
    sessions,
    activeSessionId,
    setActiveSessionId,
    addSession,
    removeSession,
    settings,
    updateSettings,
    loadSettings,
    llmConfig,
    updateLlmConfig,
    createContainer,
    startContainer,
    stopContainer,
    removeContainer,
    showRightPanel,
    setShowRightPanel,
    drawerCollapsed,
    setDrawerCollapsed,

    // Token Usage
    tokenUsage,
    updateTokenUsage,
    resetTokenUsage,

    // Streaming
    messages,
    streamingMessage,
    isStreaming,
    streamError,
    startStream,
    appendToken,
    finalizeStream,
    abortStream,
    addUserMessage,
    addSystemMessage,
    clearMessages,
    currentPendingHITL,
    setCurrentPendingHITL,
  };

  return (
    <AppContext.Provider value={value}>
      {children}
    </AppContext.Provider>
  );
}

