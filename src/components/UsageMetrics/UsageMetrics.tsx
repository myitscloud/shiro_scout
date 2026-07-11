import React from 'react';
import type { TokenUsageState, TokenUsageRole } from '../../context/AppContext';
import styles from './UsageMetrics.module.css';

// ============================================================
// Props
// ============================================================

export interface UsageMetricsProps {
  tokenUsage: TokenUsageState;
  compact?: boolean;
}

const ROLE_LABELS: Record<TokenUsageRole, string> = {
  chat: 'Chat',
  utility: 'Utility',
  embedding: 'Embedding',
};

const ROLE_ORDER: TokenUsageRole[] = ['chat', 'utility', 'embedding'];

// ============================================================
// Helpers
// ============================================================

/** Format tokens with locale-aware thousands separators. */
function fmtTokens(n: number): string {
  return n.toLocaleString();
}

/** Format a cost value to a readable dollar string. */
function fmtCost(cost: number): string {
  if (cost <= 0) return '$0.00';
  if (cost < 0.01) return '< $0.01';
  return '$' + cost.toFixed(4).replace(/\.?0+$/, '');
}

/** Format role badge label. */
function roleLabel(role: TokenUsageRole): string {
  return ROLE_LABELS[role] ?? role.charAt(0).toUpperCase() + role.slice(1);
}

// ============================================================
// Component
// ============================================================

const UsageMetrics: React.FC<UsageMetricsProps> = ({
  tokenUsage,
  compact = false,
}) => {
  const hasData = tokenUsage.sessionTotal.total_tokens > 0;

  // Empty state
  if (!hasData) {
    return (
      <div
        className={`${styles.container} ${compact ? styles.compact : ''}`}
        role="region"
        aria-label="Token usage metrics"
      >
        <h4 className={styles.heading}>Token Usage</h4>
        <p className={styles.empty}>
          No usage data yet — start a conversation to see token usage
        </p>
      </div>
    );
  }

  return (
    <div
      className={`${styles.container} ${compact ? styles.compact : ''}`}
      role="region"
      aria-label="Token usage metrics"
    >
      <h4 className={styles.heading}>Token Usage</h4>

      {/* Per-role breakdown */}
      <div className={styles.table} role="table" aria-label="Per-role token breakdown">
        <div className={styles.header} role="row">
          <span className={styles.cell} role="columnheader" aria-label="Role">Role</span>
          <span className={styles.cell} role="columnheader" aria-label="Tokens used">Tokens</span>
          <span className={styles.cell} role="columnheader" aria-label="Estimated cost">Cost</span>
        </div>
        {ROLE_ORDER.map((role) => {
          const roleData = tokenUsage.byRole[role];
          if (!roleData || roleData.total_tokens === 0) return null;
          return (
            <div key={role} className={styles.row} role="row">
              <span className={styles.cell} role="cell">
                <span className={`${styles.badge} ${styles[`badge_${role}`] ?? ''}`}>
                  {roleLabel(role)}
                </span>
              </span>
              <span className={`${styles.cell} ${styles.cellMono}`} role="cell">
                {fmtTokens(roleData.total_tokens)}
              </span>
              <span className={`${styles.cell} ${styles.cellMono}`} role="cell">
                {fmtCost(roleData.cost_estimate)}
              </span>
            </div>
          );
        })}
      </div>

      {/* Session total */}
      <div className={styles.total} role="row" aria-label="Session total">
        <div className={styles.totalRow}>
          <span className={styles.totalLabel}>Session total</span>
          <span className={styles.totalTokens}>
            {fmtTokens(tokenUsage.sessionTotal.total_tokens)} tokens
          </span>
          <span className={styles.totalCost}>
            {fmtCost(tokenUsage.sessionTotal.cost_estimate)}
          </span>
        </div>
      </div>
    </div>
  );
};

UsageMetrics.displayName = 'UsageMetrics';
export default UsageMetrics;
