import React, { useState, useCallback, useEffect, useRef } from 'react';
import { useShikiHighlighter } from '../../hooks/useShikiHighlighter';
import styles from './CodeBlock.module.css';

interface CodeBlockProps {
  code: string;
  language?: string;
  filename?: string;
  showRun?: boolean;
  onRun?: () => void;
}

const CodeBlock: React.FC<CodeBlockProps> = ({
  code,
  language,
  filename,
  showRun = false,
  onRun,
}) => {
  const [copied, setCopied] = useState(false);
  const [running, setRunning] = useState(false);
  const [highlightedHtml, setHighlightedHtml] = useState<string | null>(null);
  const { ready, highlight } = useShikiHighlighter();
  const prevCodeRef = useRef(code);
  const prevLangRef = useRef(language);

  const lineCount = code.split('\n').length;

  // Highlight code with shiki when highlighter is ready or code changes
  useEffect(() => {
    if (!ready) return;
    if (prevCodeRef.current === code && prevLangRef.current === language && highlightedHtml) return;
    prevCodeRef.current = code;
    prevLangRef.current = language;

    const lang = language || 'text';
    highlight(code, lang).then((result) => {
      setHighlightedHtml(result.html);
    });
  }, [code, language, ready, highlight, highlightedHtml]);

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {}
  }, [code]);

  const handleRun = useCallback(() => {
    if (onRun) {
      setRunning(true);
      onRun();
      setTimeout(() => setRunning(false), 1500);
    }
  }, [onRun]);

  return (
    <div className={styles.codeblock}>
      <div className={styles['cb-head']}>
        📄 <span className={styles.fname}>{filename || 'code'}</span>
        {language && <span className={styles.lang}>{language}</span>}
        <button className="btn sm ghost copy-btn" title="Copy code" onClick={handleCopy}>
          {copied ? '✓ Copied' : '⧉ Copy'}
        </button>
        {showRun && (
          <button className="btn sm secondary run-btn" title="Run in sandbox" onClick={handleRun} disabled={running}>
            {running ? '…' : '▶ Run'}
          </button>
        )}
      </div>
      <div className={styles['cb-body']}>
        {highlightedHtml ? (
          <div
            className={styles.highlighted}
            dangerouslySetInnerHTML={{ __html: highlightedHtml }}
          />
        ) : (
          <pre><code>{code}</code></pre>
        )}
      </div>
      <div className={styles['cb-foot']}>Lines 1–{lineCount} · {language || 'text'}</div>
    </div>
  );
};

CodeBlock.displayName = 'CodeBlock';
export default CodeBlock;
