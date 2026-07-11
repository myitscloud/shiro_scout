import React, { type ButtonHTMLAttributes, type ReactNode, forwardRef } from 'react';
import styles from './Button.module.css';

export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'icon' | 'link';
export type ButtonSize = 'sm' | 'md';

export interface ButtonProps extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'type'> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  loading?: boolean;
  disabled?: boolean;
  icon?: ReactNode;
  children?: ReactNode;
  onClick?: React.MouseEventHandler<HTMLButtonElement>;
  type?: 'button' | 'submit' | 'reset';
  ariaLabel?: string;
}

const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({
    variant = 'primary',
    size = 'md',
    loading = false,
    disabled = false,
    icon,
    children,
    onClick,
    type = 'button',
    ariaLabel,
    className,
    ...rest
  }, ref) => {
    const isDisabled = disabled || loading;
    const classes = [
      styles.btn,
      styles[variant],
      size === 'sm' ? styles.sm : '',
      loading ? styles.loading : '',
      className ?? '',
    ].filter(Boolean).join(' ');

    return (
      <button
        ref={ref}
        type={type}
        className={classes}
        disabled={isDisabled}
        onClick={!isDisabled ? onClick : undefined}
        aria-label={ariaLabel}
        aria-busy={loading || undefined}
        {...rest}
      >
        {loading && <span className="btn-loading"></span>}
        {!loading && icon && icon}
        {children}
      </button>
    );
  },
);

Button.displayName = 'Button';
export default Button;
