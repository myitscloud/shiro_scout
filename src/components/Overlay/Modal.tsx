import React, { type ReactNode } from 'react';
import styles from './Modal.module.css';

export interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title: string | ReactNode;
  children: ReactNode;
  actions?: ReactNode;
  accentColor?: string;
  palette?: boolean;
}

const Modal: React.FC<ModalProps> = ({
  isOpen,
  onClose,
  title,
  children,
  actions,
  accentColor,
  palette = false,
}) => {
  if (!isOpen) return null;

  const modalClasses = [styles.modal];
  if (palette) modalClasses.push(styles.palette);

  const scrimClasses = [styles.scrim, isOpen ? styles.open : ''].filter(Boolean).join(' ');

  return (
    <div className={scrimClasses} role="dialog" aria-modal="true" aria-labelledby="modal-title" onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}>
      <div className={modalClasses.join(' ')} style={accentColor ? { borderLeft: `3px solid ${accentColor}` } : undefined}>
        {title && <h3 id="modal-title">{title}</h3>}
        {children}
        {actions && <div className={styles.foot}>{actions}</div>}
      </div>
    </div>
  );
};

Modal.displayName = 'Modal';
export default Modal;
