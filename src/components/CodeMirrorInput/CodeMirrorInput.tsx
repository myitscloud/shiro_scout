import React, { useRef, useEffect } from 'react';
import { EditorView, keymap, placeholder } from '@codemirror/view';
import { EditorState, Compartment } from '@codemirror/state';
import { defaultKeymap, historyKeymap } from '@codemirror/commands';
import { markdown } from '@codemirror/lang-markdown';
import { syntaxHighlighting, defaultHighlightStyle } from '@codemirror/language';
import { autocompletion, closeBrackets } from '@codemirror/autocomplete';
import { basicSetup } from 'codemirror';
import styles from './CodeMirrorInput.module.css';

export interface CodeMirrorInputProps {
  value: string;
  onChange: (value: string) => void;
  onCtrlEnter?: () => void;
  placeholder?: string;
  minHeight?: string;
  maxHeight?: string;
  disabled?: boolean;
}

/**
 * CodeMirror 6-based chat input component.
 * Preserves tabs/spaces, provides markdown-aware editing,
 * and avoids string scrambling issues common with plain textareas.
 */
const CodeMirrorInput: React.FC<CodeMirrorInputProps> = ({
  value,
  onChange,
  onCtrlEnter,
  placeholder: placeHolderText = 'Type a message...',
  minHeight = '44px',
  maxHeight = '200px',
  disabled = false,
}) => {
  const editorRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const onChangeRef = useRef(onChange);
  const onCtrlEnterRef = useRef(onCtrlEnter);
  const valueRef = useRef(value);
  const disabledCompartment = useRef(new Compartment());

  // Keep callback refs up to date without re-creating editor
  onChangeRef.current = onChange;
  onCtrlEnterRef.current = onCtrlEnter;
  valueRef.current = value;

  // Initialize editor once
  useEffect(() => {
    if (!editorRef.current || viewRef.current) return;

    const updateListener = EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        const newValue = update.state.doc.toString();
        valueRef.current = newValue;
        onChangeRef.current(newValue);
      }
    });

    const ctrlEnterBinding = keymap.of([
      {
        key: 'Ctrl-Enter',
        run: () => {
          onCtrlEnterRef.current?.();
          return true;
        },
      },
    ]);

    const state = EditorState.create({
      doc: value,
      extensions: [
        basicSetup,
        markdown(),
        syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
        autocompletion(),
        closeBrackets(),
        placeholder(placeHolderText),
        updateListener,
        ctrlEnterBinding,
        keymap.of([...defaultKeymap, ...historyKeymap]),
        EditorView.theme({
          '&': {
            fontSize: '13.5px',
            fontFamily: 'var(--font-ui, -apple-system, sans-serif)',
            backgroundColor: 'transparent',
            minHeight: minHeight,
            maxHeight: maxHeight,
          },
          '.cm-scroller': {
            overflow: 'auto',
            fontFamily: 'inherit',
          },
          '.cm-content': {
            padding: '8px 12px',
            caretColor: 'var(--accent-purple, #8B5CF6)',
            fontFamily: 'inherit',
            lineHeight: '1.5',
          },
          '.cm-line': {
            padding: '0',
          },
          '.cm-cursor': {
            borderLeftColor: 'var(--accent-purple, #8B5CF6)',
          },
          '.cm-selectionBackground': {
            backgroundColor: 'rgba(139, 92, 246, 0.2) !important',
          },
          '&.cm-focused .cm-selectionBackground': {
            backgroundColor: 'rgba(139, 92, 246, 0.3) !important',
          },
          '.cm-placeholder': {
            color: 'var(--text-muted, #6B6B7A)',
            fontFamily: 'inherit',
            fontSize: '13.5px',
          },
          '.cm-gutters': {
            display: 'none',
          },
          '&.cm-editor': {
            border: 'none',
            outline: 'none',
            height: '100%',
          },
          '.cm-activeLine': {
            backgroundColor: 'transparent',
          },
          '.cm-activeLineGutter': {
            backgroundColor: 'transparent',
          },
          '.cm-matchingBracket': {
            backgroundColor: 'rgba(139, 92, 246, 0.15)',
            outline: '1px solid rgba(139, 92, 246, 0.3)',
          },
        }),
        disabledCompartment.current.of(EditorState.readOnly.of(disabled)),
        EditorView.domEventHandlers({
          focus: () => {
            editorRef.current?.classList.add(styles.focused);
          },
          blur: () => {
            editorRef.current?.classList.remove(styles.focused);
          },
        }),
      ],
    });

    const view = new EditorView({
      state,
      parent: editorRef.current,
    });

    viewRef.current = view;

    return () => {
      view.destroy();
      viewRef.current = null;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Update editor content when value changes externally
  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    const currentDoc = view.state.doc.toString();
    if (currentDoc !== value) {
      view.dispatch({
        changes: {
          from: 0,
          to: currentDoc.length,
          insert: value,
        },
      });
    }
  }, [value]);

  // Update read-only/disabled state
  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    view.dispatch({
      effects: disabledCompartment.current.reconfigure(EditorState.readOnly.of(disabled)),
    });
  }, [disabled]);

  return (
    <div
      ref={editorRef}
      className={`${styles.editor} ${disabled ? styles.disabled : ''}`}
      role="textbox"
      aria-multiline="true"
      aria-label={placeHolderText}
    />
  );
};

CodeMirrorInput.displayName = 'CodeMirrorInput';
export default CodeMirrorInput;
