import React from 'react';
import styles from './StreamingText.module.css';

export interface StreamingTextProps {
  /**
   * The current accumulated text to display.
   * During streaming this grows token-by-token from IPC events;
   * on completion it contains the full message.
   */
  content: string;
  /** Whether the LLM is still emitting tokens. When true, show breathing cursor. */
  isStreaming: boolean;
  /** Called on stream completion (kept for backward compatibility). */
  onComplete?: (text: string) => void;
}

/**
 * Renders streaming LLM output token-by-token as it arrives via props.
 *
 * Previously this component used a simulated setInterval-based typing effect.
 * Now it displays the real accumulated text directly — tokens arrive from the
 * Rust `llm-token` Tauri event and are rendered immediately via `content`.
 *
 * The breathing cursor animation (Steady Cursor) is shown only while
 * `isStreaming` is true.
 */
const StreamingText: React.FC<StreamingTextProps> = ({
  content,
  isStreaming,
  onComplete,
}) => {
  // Notify onComplete once when streaming finishes and content is present
  const completedRef = React.useRef(false);

  React.useEffect(() => {
    if (!isStreaming && content && !completedRef.current) {
      completedRef.current = true;
      onComplete?.(content);
    }
    if (isStreaming) {
      // Reset flag when a new stream starts
      completedRef.current = false;
    }
  }, [isStreaming, content, onComplete]);

  return (
    <span className={styles.wrapper}>
      <span className={isStreaming ? styles.content : styles.markdownContent}>
        {content}
      </span>
      {isStreaming && <span className={styles.cursor} />}
    </span>
  );
};

StreamingText.displayName = 'StreamingText';
export default StreamingText;
