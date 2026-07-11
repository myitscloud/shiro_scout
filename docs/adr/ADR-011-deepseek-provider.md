# ADR-011: DeepSeek as First-Class LLM Provider

**Status:** Accepted
**Date:** 2026-07-07
**Deciders:** Tech Lead, Security Engineer

## Context
Project Aegis is designed to be LLM-agnostic and streaming-first, supporting both cloud and local LLM providers. The system currently uses DeepSeek models (deepseek-v4-flash, deepseek-chat, deepseek-coder) as the primary runtime provider. The architecture must treat DeepSeek as a first-class, deeply integrated provider rather than a generic OpenAI-compatible endpoint.

## Decision Drivers
- DeepSeek offers specialized models (chat, coder, flash) with different latency/cost profiles
- API keys are stored in the OS keychain and proxied through Tauri host — DeepSeek keys must never enter the container
- DeepSeek streaming format differs from OpenAI in specific fields (usage, finish_reason)
- The system should support provider-specific features (e.g., DeepSeek's prompt caching, context window)
- Provider abstraction layer must still allow future non-OpenAI-compatible providers

## Considered Options
- **Option A:** Generic OpenAI-compatible wrapper — works for basic cases, but loses DeepSeek-specific features and requires mapping quirks
- **Option B:** DeepSeek as first-class provider with dedicated client — full feature support, clean abstraction, but more code
- **Option C:** Pluggable provider interface with DeepSeek as reference implementation — most flexible, supports future providers with same pattern

## Decision
Chosen: **Option C — Pluggable provider interface with DeepSeek as reference implementation**

Define a `Provider` trait/interface in the Rust backend with these methods:
- `stream_chat() -> Vec<StreamEvent>` (typed streaming response)
- `complete() -> String` (non-streaming fallback)
- `models() -> Vec<ModelInfo>` (available models with capabilities)

Implement `DeepSeekProvider` as the first concrete implementation, supporting:
- deepseek-v4-flash (fast, cost-effective for most tasks)
- deepseek-chat (general conversation)
- deepseek-coder (code generation with extended context)

All provider credentials are managed in the Tauri keychain via tauri-plugin-safe-storage. The provider selection is a runtime configuration passed from frontend.

## Consequences
- Positive: Full access to DeepSeek-specific features (prompt caching, model-specific context windows)
- Positive: Clean provider abstraction — adding OpenAI, Anthropic, or local models follows the same pattern
- Positive: Keys never enter the container, managed entirely by Tauri host
- Positive: Streaming works correctly with DeepSeek's specific stream format
- Negative: More initial code than a generic wrapper
- Negative: Each new provider requires implementing the full Provider interface
- Negative: Provider-specific quirks must be documented for each implementation

## Compliance
- All LLM provider integrations must implement the Provider trait/interface
- No provider credentials shall be passed to the Docker container under any circumstances
- Streaming support is mandatory for all cloud providers
- Provider selection must be configurable at runtime from the frontend, not hardcoded
