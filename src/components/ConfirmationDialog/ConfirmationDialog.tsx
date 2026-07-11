import React, { useState, useEffect, useCallback, useRef } from 'react';
import Modal from '../Overlay/Modal';
import styles from './ConfirmationDialog.module.css';

export interface ConfirmationDialogProps {
  isOpen: boolean;
  onApprove: (reason?: string) => void;
  onReject: (reason?: string) => void;
  operationName: string;
  operationDescription: string;
  riskLevel: 'critical' | 'high' | 'medium' | 'low';
  onClose: () => void;
}

const ACCENT_MAP: Record<string, string> = {
  critical: '#ef4444',
  high: '#f97316',
  medium: '#eab308',
  low: '#3b82f6',
};

const TIMEOUT_SECONDS = 30;

const ConfirmationDialog: React.FC<ConfirmationDialogProps> = ({
  isOpen,
  onApprove,
  onReject,
  operationName,
  operationDescription,
  riskLevel,
  onClose,
}) => {
  const [timeLeft, setTimeLeft] = useState(TIMEOUT_SECONDS);
  const [acknowledged, setAcknowledged] = useState(false);
  const [reason, setReason] = useState('');
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const isHighOrCritical = riskLevel === 'critical' || riskLevel === 'high';
  const isCritical = riskLevel === 'critical';
  const canApprove = !isHighOrCritical || (isHighOrCritical && (isCritical ? (reason.length >= 10) : acknowledged));

  const clearTimer = useCallback(() => {
    if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
  }, []);

  useEffect(() => {
    if (isOpen) {
      setTimeLeft(TIMEOUT_SECONDS);
      setAcknowledged(false);
      setReason('');
      clearTimer();
      timerRef.current = setInterval(() => {
        setTimeLeft((prev) => {
          if (prev <= 1) {
            clearTimer();
            onReject('timeout');
            return 0;
          }
          return prev - 1;
        });
      }, 1000);
    } else {
      clearTimer();
    }
    return clearTimer;
  }, [isOpen, onReject, clearTimer]);

  const handleApprove = () => {
    clearTimer();
    onApprove(isCritical ? reason : undefined);
  };

  const handleReject = () => {
    clearTimer();
    onReject(reason || undefined);
  };

  const accentColor = ACCENT_MAP[riskLevel] || ACCENT_MAP.low;

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="Confirm Dangerous Operation"
      accentColor={accentColor}
    >
      <div className={styles.dialogContent}>
        <div className={styles.riskBadge} data-risk={riskLevel}>
          {riskLevel.toUpperCase()} RISK
        </div>
        <h4 className={styles.operationName}>{operationName}</h4>
        <p className={styles.operationDescription}>{operationDescription}</p>
        <div className={styles.timer}>
          Auto-confirm in <span className={styles.timerValue}>{timeLeft}</span>s
        </div>
        {isHighOrCritical && (
          <label className={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={acknowledged}
              onChange={(e) => setAcknowledged(e.target.checked)}
              className={styles.checkbox}
            />
            I understand the consequences
          </label>
        )}
        {isCritical && (
          <textarea
            className={styles.reasonTextarea}
            placeholder="Provide a reason for this critical operation (min 10 chars)..."
            value={reason}
            onChange={(e) => setReason(e.target.value)}
            rows={3}
          />
        )}
        <div className={styles.actions}>
          <button className={styles.rejectButton} onClick={handleReject}>
            Reject
          </button>
          <button
            className={styles.approveButton}
            onClick={handleApprove}
            disabled={!canApprove}
          >
            Approve
          </button>
        </div>
      </div>
    </Modal>
  );
};

ConfirmationDialog.displayName = 'ConfirmationDialog';
export default ConfirmationDialog;
