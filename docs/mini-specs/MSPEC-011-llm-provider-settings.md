# MSPEC-011: LLM Provider Settings Page

## Overview
Settings page for configuring LLM providers following Agent Zero's 3-role pattern (chat, utility, embedding). Each role gets independent provider selection, model name, and API key.

## Layout Pattern
- Tab-based navigation: **General** | **LLM Providers**
- LLM Providers tab shows 3 role cards stacked vertically
- Each card: dropdown for provider + text field for model + password field for API key + Test Connection button

## Per-Role Configuration

| Role | Purpose | Default Provider | Default Model |
|------|---------|-----------------|---------------|
| **Chat LLM** | Primary reasoning & tool use | `deepseek` | `deepseek-v4-flash` |
| **Utility LLM** | Summaries, memory, compression | `deepseek` | `deepseek-v4-flash` |
| **Embedding LLM** | Vector embeddings | `deepseek` | `deepseek-v4-flash` |

## Provider Options

| Provider | Backend | Default Base URL |
|----------|---------|-----------------|
| DeepSeek | `rig::providers::openai` | `https://api.deepseek.com/v1` |
| OpenAI | `rig::providers::openai` | `https://api.openai.com/v1` |
| Groq | `rig::providers::openai` | `https://api.groq.com/openai/v1` |
| Together | `rig::providers::openai` | `https://api.together.xyz/v1` |
| Ollama | `rig::providers::openai` | `http://localhost:11434/v1` |
| LiteLLM | `rig::providers::openai` | Custom (user enters URL) |

## UI Elements (per role card)

1. **Provider dropdown** (`<select>`) — 6 options above, styled with glass-morphism
2. **Model name** (`<input>`) — free text, default per role
3. **API key** (`<input type="password">`) — with show/hide toggle button
4. **Test Connection** button — 3 states: idle (default), loading (spinning), connected (green ✓), failed (red ✗)

## Persistence
- Rust struct: `LlmSettings { chat: ProviderSetting, utility: ProviderSetting, embedding: ProviderSetting }`
- Saved to `{app_config_dir}/llm_settings.json` via `save_llm_settings` Tauri command
- Loaded on app startup via `load_llm_settings` command
- Defaults returned if file missing

## Data Flow
```
LLMProviderSettings.tsx
  → AppContext (llmConfig, updateLlmConfig)
    → getLlmSettings() / saveLlmSettings()
      → Tauri IPC (invoke)
        → settings.rs (read/write JSON)
          → {config_dir}/llm_settings.json
```

## References
- Agent Zero Settings Web UI (source of truth for 3-role pattern)
- `src-tauri/src/settings.rs` — Rust persistence
- `src/tauri-commands.ts` — IPC wrappers
- `src/context/AppContext.tsx` — shared state