import React, { useState, useEffect, useRef, useCallback, type ReactNode } from 'react';
import StreamingText from '../StreamingText/StreamingText';
import styles from './ChatMessage.module.css';

export interface ChatMessageProps {
  variant: 'user' | 'agent' | 'system';
  content?: string | ReactNode;
  who?: string;
  timestamp?: string;
  /**
   * When true, render `content` inside a StreamingText component with a
   * breathing cursor. The variant must also be `'agent'` for streaming to
   * activate (user/system messages never stream).
   */
  isStreaming?: boolean;
  /** Error message to display below content (e.g. stream disconnect). */
  error?: string | null;
  children?: ReactNode;
}

const ChatMessage: React.FC<ChatMessageProps> = ({
  variant,
  content,
  who,
  timestamp,
  isStreaming = false,
  error = null,
  children,
}) => {
  const [copied, setCopied] = useState(false);
  const copyTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    return () => {
      if (copyTimerRef.current) clearTimeout(copyTimerRef.current);
    };
  }, []);

  const handleCopy = useCallback(() => {
    if (typeof content !== 'string' || !content) return;
    navigator.clipboard.writeText(content).then(() => {
      setCopied(true);
      if (copyTimerRef.current) clearTimeout(copyTimerRef.current);
      copyTimerRef.current = setTimeout(() => setCopied(false), 1500);
    }).catch(() => {
      // silent fail — clipboard unavailable
    });
  }, [content]);

  /**
   * Render the content area.
   * - Agent messages that are streaming → use StreamingText with real tokens.
   * - Agent messages that have completed → render full content as text.
   * - User/system messages → render as plain text or ReactNode.
   * - Non-string content (ReactNode) → render directly.
   */
  const renderContent = () => {
    // If content is a ReactNode (not a plain string), render it directly
    if (typeof content !== 'string') {
      return content;
    }

    // Agent variant + streaming = use StreamingText
    if (variant === 'agent' && isStreaming) {
      return (
        <StreamingText
          content={content}
          isStreaming={isStreaming}
        />
      );
    }

    // Simple text rendering for complete messages or user/system messages
    return <p>{content}</p>;
  };

  const showCopyButton = !isStreaming && typeof content === 'string' && content.length > 0;

  return (
    <div className={`${styles.msg} ${styles[variant]}`}>
      <div className={styles.meta}>
        {variant === 'agent' && (
          <span className="dot thinking" style={{ width: 7, height: 7 }}></span>
        )}
        {who && <span className={styles.who}>{who}</span>}
        {timestamp && <span>· {timestamp}</span>}
      </div>

      {renderContent()}

      {/* Error indicator for stream disconnects or failures */}
      {error && (
        <div className={styles.errorIndicator} role="alert">
          <span className={styles.errorIcon}>⚠</span>
          <span className={styles.errorText}>{error}</span>
        </div>
      )}

      {children}

      {showCopyButton && (
        <button
          className={styles.copyBtn}
          onClick={handleCopy}
          aria-label={copied ? 'Copied' : 'Copy message'}
          title={copied ? 'Copied' : 'Copy to clipboard'}
        >
          {copied ? '✓' : '📋'}
        </button>
      )}
    </div>
  );
};

ChatMessage.displayName = 'ChatMessage';
export default ChatMessage;
