import { useState, useEffect, useRef } from 'react';
import { createHighlighter, type HighlighterCore } from 'shiki';

// Languages we want to support in code blocks
const SUPPORTED_LANGUAGES: string[] = [
  'rust',
  'typescript',
  'javascript',
  'tsx',
  'jsx',
  'python',
  'bash',
  'shell',
  'json',
  'yaml',
  'html',
  'css',
  'markdown',
  'sql',
  'toml',
  'docker',
  'powershell',
  'terminal',
];

// Cache for the highlighter singleton
let highlighterInstance: HighlighterCore | null = null;
let highlighterPromise: Promise<HighlighterCore> | null = null;

async function getHighlighter(): Promise<HighlighterCore> {
  if (highlighterInstance) return highlighterInstance;
  if (!highlighterPromise) {
    highlighterPromise = createHighlighter({
      langs: SUPPORTED_LANGUAGES,
      themes: ['github-dark', 'github-light'],
    });
  }
  highlighterInstance = await highlighterPromise;
  return highlighterInstance;
}

// Detect dark mode from body class
function isDarkMode(): boolean {
  return !document.body.classList.contains('light');
}

export interface HighlightedCode {
  html: string;
  language: string;
}

/**
 * Hook that uses shiki to highlight code blocks with syntax colors.
 * Returns highlighted HTML strings for rendering via dangerouslySetInnerHTML.
 *
 * Usage:
 *   const { highlight } = useShikiHighlighter();
 *   const result = await highlight('let x = 1;', 'typescript');
 *   // result.html contains the syntax-highlighted HTML
 */
export function useShikiHighlighter() {
  const [ready, setReady] = useState(!!highlighterInstance);
  const highlighterRef = useRef<HighlighterCore | null>(highlighterInstance);

  useEffect(() => {
    if (highlighterInstance) {
      setReady(true);
      return;
    }

    let cancelled = false;
    getHighlighter().then((hl) => {
      if (!cancelled) {
        highlighterRef.current = hl;
        setReady(true);
      }
    });

    return () => {
      cancelled = true;
    };
  }, []);

  const highlight = async (
    code: string,
    language: string,
  ): Promise<HighlightedCode> => {
    const hl = highlighterRef.current || (await getHighlighter());
    const dark = isDarkMode();
    const normalizedLang = language === 'terminal' ? 'bash' : language;

    try {
      const html = hl.codeToHtml(code, {
        lang: normalizedLang,
        theme: dark ? 'github-dark' : 'github-light',
        structure: 'classic',
      });
      return { html, language };
    } catch {
      // Fallback: if language is not supported, render as plain text
      const html = hl.codeToHtml(code, {
        lang: 'text',
        theme: dark ? 'github-dark' : 'github-light',
        structure: 'classic',
      });
      return { html, language };
    }
  };

  /**
   * Highlight code synchronously if highlighter is already loaded.
   * Returns null if highlighter is not ready yet.
   */
  const highlightSync = (
    code: string,
    language: string,
  ): HighlightedCode | null => {
    const hl = highlighterRef.current;
    if (!hl) return null;

    const dark = isDarkMode();
    const normalizedLang = language === 'terminal' ? 'bash' : language;

    try {
      const html = hl.codeToHtml(code, {
        lang: normalizedLang,
        theme: dark ? 'github-dark' : 'github-light',
        structure: 'classic',
      });
      return { html, language };
    } catch {
      const html = hl.codeToHtml(code, {
        lang: 'text',
        theme: dark ? 'github-dark' : 'github-light',
        structure: 'classic',
      });
      return { html, language };
    }
  };

  return { ready, highlight, highlightSync };
}
