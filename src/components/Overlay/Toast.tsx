import React, { createContext, useContext, useState, useCallback, useRef, useEffect, type ReactNode } from 'react';
import styles from './Toast.module.css';

export interface ToastItem {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  message: string;
  duration?: number;
}

interface ToastContextValue {
  addToast: (toast: Omit<ToastItem, 'id'>) => string;
  removeToast: (id: string) => void;
  toasts: ToastItem[];
}

const ToastContext = createContext<ToastContextValue | null>(null);

const TOAST_LIMIT = 5;
const DEFAULT_DURATIONS: Record<ToastItem['type'], number> = {
  success: 4000,
  info: 6000,
  warning: 8000,
  error: Infinity,
};

const TOAST_ICONS: Record<ToastItem['type'], string> = {
  success: '✓',
  error: '✗',
  warning: '⚠',
  info: 'ℹ',
};

interface ToastProviderProps { children: ReactNode }

const ToastProvider: React.FC<ToastProviderProps> = ({ children }) => {
  const [toasts, setToasts] = useState<ToastItem[]>([]);
  const timersRef = useRef<Map<string, ReturnType<typeof setTimeout>>>(new Map());
  const idCounterRef = useRef(0);

  const removeToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
    const timer = timersRef.current.get(id);
    if (timer) { clearTimeout(timer); timersRef.current.delete(id); }
  }, []);

  const addToast = useCallback((toast: Omit<ToastItem, 'id'>): string => {
    const id = `toast-${++idCounterRef.current}-${Date.now()}`;
    const duration = toast.duration ?? DEFAULT_DURATIONS[toast.type];
    setToasts((prev) => {
      const updated = [...prev, { ...toast, id }];
      if (updated.length > TOAST_LIMIT) {
        updated.slice(0, updated.length - TOAST_LIMIT).forEach(t => {
          const timer = timersRef.current.get(t.id);
          if (timer) clearTimeout(timer);
          timersRef.current.delete(t.id);
        });
        return updated.slice(-TOAST_LIMIT);
      }
      return updated;
    });
    if (Number.isFinite(duration) && duration > 0) {
      timersRef.current.set(id, setTimeout(() => removeToast(id), duration));
    }
    return id;
  }, [removeToast]);

  useEffect(() => () => { timersRef.current.forEach(t => clearTimeout(t)); timersRef.current.clear(); }, []);

  return (
    <ToastContext.Provider value={{ addToast, removeToast, toasts }}>
      {children}
      <div className={styles.toasts} aria-live="polite">
        {toasts.map(toast => (
          <div key={toast.id} className={`${styles.toast} ${styles[toast.type]}`}>
            <span className={styles.ti}>{TOAST_ICONS[toast.type]}</span>
            <span>{toast.message}</span>
            <button className={styles.tx} aria-label="Dismiss" onClick={() => removeToast(toast.id)}>✕</button>
          </div>
        ))}
      </div>
    </ToastContext.Provider>
  );
};

ToastProvider.displayName = 'ToastProvider';

function useToast(): ToastContextValue {
  const ctx = useContext(ToastContext);
  if (!ctx) throw new Error('useToast must be used within a ToastProvider');
  return ctx;
}

export { ToastProvider, useToast };
export default ToastProvider;
