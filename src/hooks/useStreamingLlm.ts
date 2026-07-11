import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useCallback, useEffect, useRef, useState } from 'react';

// ============================================================
// Types — mirrors the Rust `llm-token` event payload
// ============================================================

export interface LlmTokenPayload {
  role: string;
  token: string;
  done: boolean;
}

// ============================================================
// Hook options
// ============================================================

export interface UseStreamingLlmOptions {
  /** Called for each received token fragment. */
  onToken?: (token: string, role: string) => void;
  /** Called when the stream signals `done: true`. */
  onDone?: (fullText: string, role: string) => void;
  /** Called on listen-setup error or stream disconnect. */
  onError?: (error: string) => void;
  /**
   * Milliseconds without a token before the stream is considered dropped.
   * Default 120 000 (2 minutes). Set to 0 to disable the timeout.
   */
  streamTimeoutMs?: number;
}

// ============================================================
// Hook
// ============================================================

/**
 * Hook that manages the lifecycle of listening to Tauri `llm-token` events.
 *
 * - `startStream()` clears accumulated text and registers the IPC listener.
 * - Tokens are accumulated into `streamingMessage` in real time.
 * - When `done: true` is received, `isStreaming` flips to `false` and
 *   `onDone` is called.
 * - If no token or `done` arrives within `streamTimeoutMs`, a disconnect
 *   error is raised and the listener is cleaned up.
 * - Calling `stopStream()` manually unlistens and preserves partial text.
 * - The listener is always cleaned up on unmount (F5 rule).
 */
export function useStreamingLlm(options?: UseStreamingLlmOptions) {
  const [streamingMessage, setStreamingMessage] = useState('');
  const [isStreaming, setIsStreaming] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeRole, setActiveRole] = useState<string | null>(null);

  // --- refs to avoid stale closures ---
  const optionsRef = useRef(options);
  optionsRef.current = options;

  const unlistenRef = useRef<UnlistenFn | null>(null);
  const accumulatedRef = useRef('');
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isMountedRef = useRef(true);

  // ---------- helpers ----------

  const clearTimeout_ = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }
  }, []);

  const resetTimeout_ = useCallback(
    (timeoutMs: number) => {
      clearTimeout_();
      if (timeoutMs > 0) {
        timeoutRef.current = setTimeout(() => {
          if (!isMountedRef.current) return;
          const partial = accumulatedRef.current;
          setError(`Stream disconnected after ${(timeoutMs / 1000).toFixed(0)}s of inactivity`);
          setIsStreaming(false);
          optionsRef.current?.onError?.(`Stream timeout — partial content preserved (${partial.length} chars)`);
        }, timeoutMs);
      }
    },
    [clearTimeout_],
  );

  const cleanupListener = useCallback(() => {
    if (unlistenRef.current) {
      unlistenRef.current();
      unlistenRef.current = null;
    }
  }, []);

  const finalize = useCallback((fullText: string, role: string) => {
    setIsStreaming(false);
    clearTimeout_();
    cleanupListener();
    optionsRef.current?.onDone?.(fullText, role);
  }, [clearTimeout_, cleanupListener]);

  // ---------- start ----------

  const startStream = useCallback(() => {
    if (!isMountedRef.current) return;

    // Reset everything
    cleanupListener();
    clearTimeout_();
    accumulatedRef.current = '';
    setStreamingMessage('');
    setIsStreaming(true);
    setError(null);
    setActiveRole(null);

    const timeoutMs = optionsRef.current?.streamTimeoutMs ?? 120_000;

    let roleSet = false;

    listen<LlmTokenPayload>('llm-token', (event) => {
      if (!isMountedRef.current) return;

      const { role, token, done } = event.payload;

      // Set role on first event only (local let avoids stale closure)
      if (role && !roleSet) {
        roleSet = true;
        setActiveRole(role);
      }

      if (done) {
        const fullText = accumulatedRef.current;
        finalize(fullText, role);
        return;
      }

      // Accumulate token
      accumulatedRef.current += token;
      setStreamingMessage(accumulatedRef.current);

      // Feed callback
      optionsRef.current?.onToken?.(token, role);

      // Reset inactivity timeout
      resetTimeout_(timeoutMs);
    })
      .then((unlisten) => {
        if (!isMountedRef.current) {
          unlisten();
          return;
        }
        unlistenRef.current = unlisten;
        resetTimeout_(timeoutMs);
      })
      .catch((err: unknown) => {
        if (!isMountedRef.current) return;
        const msg = err instanceof Error ? err.message : String(err);
        setError(`Failed to attach stream listener: ${msg}`);
        setIsStreaming(false);
        optionsRef.current?.onError?.(msg);
      });
  }, [cleanupListener, clearTimeout_, finalize, resetTimeout_]);

  // ---------- stop ----------

  const stopStream = useCallback(() => {
    cleanupListener();
    clearTimeout_();
    setIsStreaming(false);
    // Keep partial text available
  }, [cleanupListener, clearTimeout_]);

  // ---------- cleanup on unmount ----------

  useEffect(() => {
    return () => {
      isMountedRef.current = false;
      cleanupListener();
      clearTimeout_();
    };
  }, [cleanupListener, clearTimeout_]);

  return {
    /** Current accumulated message text (updated per token). */
    streamingMessage,
    /** Whether the stream is actively receiving tokens. */
    isStreaming,
    /** The role from the last/current event (e.g. 'assistant'). */
    activeRole,
    /** Non-null when the stream encountered an error. */
    error,
    /** Start listening for `llm-token` events. Call when the assistant begins responding. */
    startStream,
    /** Manually stop the stream (preserves partial text). */
    stopStream,
  };
}
