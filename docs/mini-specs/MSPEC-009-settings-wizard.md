# Mini-Spec 009: Settings & First-Run Wizard Components

**Task:** Create the Settings view (LLM provider config, Docker path, theme, preferences) and FirstRunWizard component (4-step stepper overlay for first launch).

**Layers touched:** Frontend (React/TypeScript + CSS Modules)

**Owning agent:** Frontend Engineer

## Acceptance Criteria

### Settings View
1. Renders in the Main Panel as an overlay replacing ChatView

2. Navigation sidebar with 7 sections:
   - General (theme, preferences)
   - LLM Providers (OpenAI, Anthropic, DeepSeek, Ollama, LM Studio)
   - Docker (path, daemon status, resource limits)
   - Agents (default config, timeouts)
   - Workspace (path, auto-save)
   - Keyboard Shortcuts (reference list)
   - About (version, credits)

3. Settings persisted via Tauri backend (tauri-plugin-store — IPC commands defined in Wave 5)

4. LLM Provider section: per-provider config with name, model, API key field, base URL (for local providers)

5. Docker section: Docker path input, connection test button, resource limits (CPU/mem sliders)

6. Empty state when no API key configured: "No LLM provider configured. Add an API key in Settings." with navigate-to-settings link

7. CSS Module scoped — uses `design-tokens.css` tokens

8. TypeScript state management for all config fields

### First-Run Wizard
9. 4-step stepper overlay on first launch (detected by absence of config):
    | Step | Title | Content |
    |------|-------|---------|
    | 1 | Welcome | App branding, value proposition, "Get Started" CTA |
    | 2 | Docker Check | Detect Docker daemon, show install link if missing, retry button |
    | 3 | LLM Setup | Provider selection (dropdown), API key entry (masked), test connection button |
    | 4 | Agent Config | Default agent name, default model, optional preferences |

10. Each step has inline help text and retry buttons for failures

11. Skip button for experienced users (skips to main app)

12. Progress indicator showing current step (1/4, 2/4, etc.)

13. Next/Back navigation between steps

14. On completion: save config, dismiss wizard, show main app

15. CSS Module — glass elevated overlay (`--elevation-overlay`) matching Modal pattern

## Out of Scope

- Provider health check API calls (Wave 5)
- Docker connection actual testing backend (Wave 2)

## Review Triggers Expected

- Code Reviewer
- A11y & UX Specialist
