import React, { useMemo } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import styles from './StreamingText.module.css';

export interface StreamingTextProps {
  content: string;
  /** Whether the LLM is still emitting tokens. When true, show breathing cursor. */
  isStreaming: boolean;
  /** Called on stream completion. */
  onComplete?: (text: string) => void;
}

/**
 * Renders streaming LLM output token-by-token as it arrives via props.
 *
 * - While streaming: raw text + breathing cursor.
 * - On completion: renders full content through ReactMarkdown (code blocks,
 *   lists, bold, emojis, tables, etc.) with custom code block styling.
 *
 * Code fences (```terminal, ```json, etc.) render as styled `<pre>`
 * blocks with a language badge.
 */
const StreamingText: React.FC<StreamingTextProps> = ({
  content,
  isStreaming,
  onComplete,
}) => {
  const completedRef = React.useRef(false);

  React.useEffect(() => {
    if (!isStreaming && content && !completedRef.current) {
      completedRef.current = true;
      onComplete?.(content);
    }
    if (isStreaming) {
      completedRef.current = false;
    }
  }, [isStreaming, content, onComplete]);

  // Detect if we're inside a code fence during streaming for cursor placement
  const cursorInsideCodeBlock = useMemo(() => {
    if (!isStreaming) return false;
    const trimmed = content.trimEnd();
    const fenceOpeners = trimmed.match(/```/g);
    const count = fenceOpeners ? fenceOpeners.length : 0;
    // Odd counts mean we're inside a code block
    return count % 2 === 1;
  }, [content, isStreaming]);

  // During streaming: render raw text with cursor
  if (isStreaming) {
    return (
      <span className={styles.wrapper}>
        <span className={styles.content}>
          {content}
        </span>
        <span
          className={`${styles.cursor} ${cursorInsideCodeBlock ? styles.cursorTerminal : ''}`}
        />
      </span>
    );
  }

  // Completed content: render via ReactMarkdown with custom components
  return (
    <div className={styles.markdownBody}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          // Custom code block renderer for ```fenced blocks
          code({ className, children, ...props }) {
            const match = /language-(\w+)/.exec(className || '');
            const lang = match ? match[1] : '';
            const codeStr = String(children).replace(/\n$/, '');

            // Multi-line code fence → terminal block
            if (codeStr.includes('\n') || lang) {
              return (
                <div className={styles.codeBlock}>
                  {lang && (
                    <div className={styles.codeBlockHeader}>
                      <span className={styles.codeBlockLang}>{lang}</span>
                    </div>
                  )}
                  <pre className={`${styles.codeBlockPre} ${lang === 'terminal' ? styles.terminalPre : ''}`}>
                    <code className={className}>{children}</code>
                  </pre>
                </div>
              );
            }

            // Inline code
            return (
              <code className={styles.inlineCode} {...props}>
                {children}
              </code>
            );
          },
          // Custom pre — wrapped by code component above, so empty
          pre({ children }) {
            return <>{children}</>;
          },
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
};

StreamingText.displayName = 'StreamingText';
export default StreamingText;
