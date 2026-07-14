import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// ============================================================
// LLM Streaming — Rust event interface (document only — do NOT modify Rust)
// ============================================================
//
// Tauri event name: `llm-token`
// Emitted per-token by the Rust SSE stream parser.
// `done: true` signals stream completion (no token value with done).
//
export interface LlmTokenPayload {
  role: string;
  token: string;
  done: boolean;
}

// ============================================================
// Types matching Rust structs
// ============================================================

export interface DockerDaemonStatus {
  available: boolean;
  version: string | null;
  error: string | null;
}

export interface SandboxConfig {
  image: string;
  workspace_path: string;
  memory_mb: number;
  cpu_shares: number;
  network_mode: 'bridge' | 'none';
}

export interface SandboxCreateResult {
  container_id: string;
  container_name: string;
}

export interface AppSettings {
  theme: 'dark' | 'light';
  workspacePath: string;
  reduce_motion: boolean;
  provider: 'local' | 'cloud';
  model: string;
  api_key: string;
  sandbox_on_launch: boolean;
  mount_workspace: boolean;
  last_session_id: string | null;
}

export const DEFAULT_SETTINGS: AppSettings = {
  theme: 'dark',
  workspacePath: 'C:\\projects',
  reduce_motion: false,
  provider: 'local',
  model: 'deepseek-v4-flash',
  api_key: '',
  sandbox_on_launch: true,
  mount_workspace: true,
  last_session_id: null,
};

// ============================================================
// Docker daemon commands
// ============================================================

/** Check if the Docker daemon is reachable and return its version. */
export async function checkDockerDaemon(): Promise<DockerDaemonStatus> {
  return invoke<DockerDaemonStatus>('check_docker_daemon');
}

// ============================================================
// Sandbox container lifecycle commands
// ============================================================

/** Create a new sandbox container with the given configuration. */
export async function createSandbox(config: SandboxConfig): Promise<SandboxCreateResult> {
  return invoke<SandboxCreateResult>('create_sandbox', { config });
}

/** Start a sandbox container by ID. */
export async function startSandbox(id: string): Promise<void> {
  return invoke<void>('start_sandbox', { id });
}

/** Stop a sandbox container by ID with a 10-second graceful timeout. */
export async function stopSandbox(id: string): Promise<void> {
  return invoke<void>('stop_sandbox', { id });
}

/** Remove a sandbox container by ID (force remove if running). */
export async function removeSandbox(id: string): Promise<void> {
  return invoke<void>('remove_sandbox', { id });
}

/** Pull a Docker image. */
export async function pullImage(imageName: string): Promise<void> {
  return invoke<void>('pull_image', { imageName });
}

// ============================================================
// Settings persistence commands (Rust side)
// ============================================================

/** Load saved settings from the app config directory. */
export async function loadSettings(): Promise<AppSettings | null> {
  return invoke<AppSettings | null>('load_settings');
}

/** Save settings to the app config directory. */
export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke<void>('save_settings', { settings });
}

// ============================================================
// LLM Provider settings commands
// ============================================================

export interface ProviderSetting {
  provider: string;
  model: string;
  api_key: string | null;
}

export interface LlmSettings {
  chat: ProviderSetting;
  utility: ProviderSetting;
  embedding: ProviderSetting;
}

export const DEFAULT_LLM_SETTINGS: LlmSettings = {
  chat: { provider: 'deepseek', model: 'deepseek-v4-flash', api_key: null },
  utility: { provider: 'deepseek', model: 'deepseek-v4-flash', api_key: null },
  embedding: { provider: 'deepseek', model: 'deepseek-v4-flash', api_key: null },
};

/** Load LLM provider settings from the app config directory. */
export async function getLlmSettings(): Promise<LlmSettings> {
  return invoke<LlmSettings>('load_llm_settings');
}

/** Save LLM provider settings to the app config directory. */
export async function saveLlmSettings(settings: LlmSettings): Promise<void> {
  return invoke<void>('save_llm_settings', { settings });
}

/** Result of an LLM provider connection test. */
export interface TestConnectionResult {
  success: boolean;
  latency_ms: number | null;
  error: string | null;
}

/** Provider health status entry. */
export interface ProviderHealth {
  provider: string;
  healthy: boolean;
  latency_ms: number | null;
  last_checked: string;
  error: string | null;
}

/** Test an LLM provider connection via the Rust backend. */
export async function testLlmConnection(setting: ProviderSetting): Promise<TestConnectionResult> {
  const json = await invoke<string>('test_llm_connection', { settings: setting });
  return JSON.parse(json) as TestConnectionResult;
}

/** Get current health status for all configured providers. */
export async function getProviderHealth(): Promise<Record<string, ProviderHealth>> {
  return invoke<Record<string, ProviderHealth>>('get_provider_health');
}

// ============================================================
// Token Usage & Cost Estimation (Wave 6.6)
// ============================================================

/** Per-call token usage entry emitted by Rust via `token-usage` event. */
export interface TokenUsageEntry {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
  cost_estimate: number;
  model: string;
  role: 'chat' | 'utility' | 'embedding';
}

/** Default pricing rates (DeepSeek-v4-flash). Configurable per settings. */
export const DEEPSEEK_PRICING = {
  input_per_mtok: 0.15,
  output_per_mtok: 0.60,
} as const;

/**
 * Calculate cost estimate from token counts for display when Rust doesn't provide it.
 * Returns 0 if rates or token counts are missing.
 */
export function calculateCostEstimate(
  promptTokens: number,
  completionTokens: number,
  rates?: { input: number; output: number },
): number {
  const inputRate = rates?.input ?? DEEPSEEK_PRICING.input_per_mtok;
  const outputRate = rates?.output ?? DEEPSEEK_PRICING.output_per_mtok;
  return (promptTokens / 1_000_000) * inputRate + (completionTokens / 1_000_000) * outputRate;
}

// ============================================================
// HITL (Human-In-The-Loop) Confirmation — Wave 7.1
// ============================================================

/** Payload carried by the `hitl-confirmation-request` Tauri event from Rust. */
export interface HITLEvent {
  session_id: string;
  operation_name: string;
  operation_description: string;
  risk_level: string;
  payload: unknown;
  nonce: string;
}

// ============================================================
// Agent status events — sidekick LED indicators
// ============================================================

/** Payload carried by the `agent-status` Tauri event from Rust. */
export interface AgentStatusPayload {
  agent_id: string;
  status: 'online' | 'off' | 'err';
}

/** Request a HITL confirmation from the backend for a dangerous operation. */
export async function requestHITLConfirmation(
  operation: string,
  description: string,
  risk_level: string,
  payload: unknown,
): Promise<string> {
  return invoke<string>('request_hitl_confirmation', { operation, description, riskLevel: risk_level, payload });
}

/** Respond to an active HITL confirmation session. */
export async function respondHITL(
  sessionId: string,
  approved: boolean,
  reason?: string,
  nonce: string = '',
): Promise<boolean> {
  return invoke<boolean>('respond_hitl', { sessionId, approved, reason: reason || null, nonce });
}

/** Listen for incoming HITL confirmation requests from the backend. */
export function listenForHITLRequest(
  callback: (event: HITLEvent) => void,
): Promise<() => void> {
  return listen<HITLEvent>('hitl-confirmation-request', (e) => callback(e.payload));
}

// ============================================================
// LLM Streaming — Send a message to the agent backend
// ============================================================

export interface ChatTurn {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

/** Send conversation history to the LLM; tokens arrive via `llm-token` events. */
export async function sendMessage(
  history: ChatTurn[],
  provider: string,
  model: string,
  apiKey: string,
  systemPrompt?: string,
): Promise<void> {
  return invoke<void>('stream_llm_completion', {
    input: {
      provider,
      model,
      api_key: apiKey || undefined,
      system_prompt: systemPrompt || undefined,
      messages: history,
      max_tokens: 8192,
      temperature: 0.7,
      role: 'chat',
    },
  });
}
