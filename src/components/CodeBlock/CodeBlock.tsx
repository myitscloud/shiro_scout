import React, { useState, useCallback } from 'react';
import styles from './CodeBlock.module.css';

interface CodeBlockProps {
  code: string;
  language?: string;
  filename?: string;
  showRun?: boolean;
  onRun?: () => void;
}

function tokenize(code: string, lang?: string): React.ReactNode[] {
  const tokens: React.ReactNode[] = [];
  const lines = code.split('\n');
  lines.forEach((line, li) => {
    if (li > 0) tokens.push('\n');
    if (!lang || lang === 'text') {
      tokens.push(line);
      return;
    }
    let remaining = line;
    const parts: React.ReactNode[] = [];
    const commentMatch = remaining.match(/^(\/\/.*)/);
    if (commentMatch) {
      parts.push(<span className={styles['tk-cm']} key={`${li}-c`}>{commentMatch[0]}</span>);
      remaining = remaining.slice(commentMatch[0].length);
    }
    const stringRegex = /^("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*')/;
    const strMatch = remaining.match(stringRegex);
    if (strMatch) {
      parts.push(<span className={styles['tk-str']} key={`${li}-s`}>{strMatch[0]}</span>);
      remaining = remaining.slice(strMatch[0].length);
    }
    const keywords = new Set([
      'fn','let','mut','const','if','else','for','while','return',
      'pub','use','mod','struct','enum','impl','trait','async','await',
      'import','export','default','from','class','function','var',
      'type','interface','extends','new','this','try','catch','throw','match',
    ]);
    const wordRegex = /([\w$]+|[^\w$]+)/g;
    let match;
    while ((match = wordRegex.exec(remaining)) !== null) {
      const word = match[1];
      if (keywords.has(word)) {
        parts.push(<span className={styles['tk-kw']} key={`${li}-${match.index}`}>{word}</span>);
      } else if (/^\d+(\.\d+)?$/.test(word)) {
        parts.push(<span className={styles['tk-num']} key={`${li}-${match.index}`}>{word}</span>);
      } else {
        parts.push(word);
      }
    }
    if (parts.length === 0) tokens.push(line);
    else tokens.push(...parts);
  });
  return tokens;
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
  const lineCount = code.split('\n').length;
  const tokens = React.useMemo(() => tokenize(code, language), [code, language]);

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
        <button className="btn sm ghost copy-btn" title="Copy code" onClick={handleCopy}>
          {copied ? '✓ Copied' : '⧉ Copy'}
        </button>
        {showRun && (
          <button className="btn sm secondary run-btn" title="Run in sandbox" onClick={handleRun} disabled={running}>
            {running ? '…' : '▶ Run'}
          </button>
        )}
      </div>
      <pre><code>{tokens}</code></pre>
      <div className={styles['cb-foot']}>Lines 1–{lineCount} · {language || 'text'} · +1 −1</div>
    </div>
  );
};

CodeBlock.displayName = 'CodeBlock';
export default CodeBlock;
