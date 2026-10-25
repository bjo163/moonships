import {
  forwardRef,
  useEffect,
  useId,
  useRef,
  type ButtonHTMLAttributes,
  type HTMLAttributes,
  type InputHTMLAttributes,
  type ReactNode,
} from "react";
import "./ui-primitives.css";

type ButtonVariant = "primary" | "secondary" | "ghost" | "danger";
type ButtonSize = "sm" | "md" | "lg";

export type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: ButtonVariant;
  size?: ButtonSize;
  loading?: boolean;
  loadingLabel?: string;
};

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button({
  variant = "primary",
  size = "md",
  loading = false,
  loadingLabel = "Loading",
  disabled,
  className = "",
  children,
  type = "button",
  ...props
}: ButtonProps, ref) {
  return (
    <button
      {...props}
      ref={ref}
      type={type}
      className={`ms-button ms-button--${variant} ms-button--${size}${className ? ` ${className}` : ""}`}
      disabled={disabled || loading}
      aria-busy={loading || undefined}
    >
      {loading ? (
        <>
          <span className="ms-spinner" aria-hidden="true" />
          <span>{loadingLabel}</span>
        </>
      ) : (
        children
      )}
    </button>
  );
});

export type InputProps = InputHTMLAttributes<HTMLInputElement> & {
  label: string;
  hint?: string;
  error?: string;
};

export function Input({
  id,
  label,
  hint,
  error,
  className = "",
  "aria-describedby": describedBy,
  ...props
}: InputProps) {
  const inputId = id ?? props.name;
  const hintId = hint && inputId ? `${inputId}-hint` : undefined;
  const errorId = error && inputId ? `${inputId}-error` : undefined;
  const descriptionIds = [describedBy, hintId, errorId].filter(Boolean).join(" ") || undefined;

  return (
    <div className="ms-field">
      <label className="ms-label" htmlFor={inputId}>
        {label}
      </label>
      <input
        {...props}
        id={inputId}
        className={`ms-input${error ? " ms-input--invalid" : ""}${className ? ` ${className}` : ""}`}
        aria-invalid={error ? true : props["aria-invalid"]}
        aria-describedby={descriptionIds}
      />
      {hint && !error ? <p className="ms-hint" id={hintId}>{hint}</p> : null}
      {error ? <p className="ms-error" id={errorId} role="alert">{error}</p> : null}
    </div>
  );
}

type BadgeVariant = "neutral" | "success" | "warning" | "danger" | "info";

export function Badge({
  variant = "neutral",
  children,
  className = "",
  ...props
}: HTMLAttributes<HTMLSpanElement> & { variant?: BadgeVariant }) {
  return (
    <span
      {...props}
      className={`ms-badge ms-badge--${variant}${className ? ` ${className}` : ""}`}
    >
      {children}
    </span>
  );
}

export type DialogProps = {
  open: boolean;
  title: string;
  description?: string;
  onClose: () => void;
  children: ReactNode;
  labelledBy?: string;
};

export function Dialog({
  open,
  title,
  description,
  onClose,
  children,
  labelledBy,
}: DialogProps) {
  const generatedId = useId();
  const titleId = labelledBy ?? `ms-dialog-title-${generatedId.replace(/:/g, "")}`;
  const descriptionId = description ? `ms-dialog-description-${generatedId.replace(/:/g, "")}` : undefined;
  const dialogRef = useRef<HTMLElement>(null);
  const closeButtonRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!open) return;

    const previouslyFocused = document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null;
    const dialog = dialogRef.current;
    const closeButton = closeButtonRef.current;
    (closeButton ?? dialog)?.focus();

    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        event.preventDefault();
        onClose();
        return;
      }

      if (event.key !== "Tab" || !dialog) return;
      const focusable = Array.from(
        dialog.querySelectorAll<HTMLElement>(
          'a[href], button:not(:disabled), input:not(:disabled), select:not(:disabled), textarea:not(:disabled), [tabindex]:not([tabindex="-1"])',
        ),
      ).filter((element) => element.getAttribute("aria-hidden") !== "true");

      if (focusable.length === 0) {
        event.preventDefault();
        dialog.focus();
        return;
      }

      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (event.shiftKey && (document.activeElement === first || !dialog.contains(document.activeElement))) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && (document.activeElement === last || !dialog.contains(document.activeElement))) {
        event.preventDefault();
        first.focus();
      }
    }

    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("keydown", onKeyDown);
      previouslyFocused?.focus();
    };
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div className="ms-dialog-backdrop" onMouseDown={(event) => {
      if (event.target === event.currentTarget) onClose();
    }}>
      <section
        ref={dialogRef}
        className="ms-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        aria-describedby={descriptionId}
        tabIndex={-1}
      >
        <header className="ms-dialog__header">
          <div>
            <h2 id={titleId}>{title}</h2>
            {description ? <p id={descriptionId}>{description}</p> : null}
          </div>
          <Button ref={closeButtonRef} variant="ghost" size="sm" aria-label="Close dialog" onClick={onClose}>×</Button>
        </header>
        <div className="ms-dialog__content">{children}</div>
      </section>
    </div>
  );
}

export type ToastProps = {
  title: string;
  message?: string;
  tone?: "status" | "error";
};

export function Toast({ title, message, tone = "status" }: ToastProps) {
  return (
    <div className={`ms-toast ms-toast--${tone}`} role={tone === "error" ? "alert" : "status"} aria-live={tone === "error" ? "assertive" : "polite"}>
      <strong>{title}</strong>
      {message ? <p>{message}</p> : null}
    </div>
  );
}

export function EmptyState({
  title,
  description,
  action,
}: {
  title: string;
  description: string;
  action?: ReactNode;
}) {
  return (
    <section className="ms-state ms-empty-state">
      <span className="ms-state__mark" aria-hidden="true">—</span>
      <h2>{title}</h2>
      <p>{description}</p>
      {action ? <div className="ms-state__action">{action}</div> : null}
    </section>
  );
}

export function LoadingState({ label = "Loading content" }: { label?: string }) {
  return (
    <div className="ms-state ms-loading-state" role="status" aria-live="polite">
      <span className="ms-spinner" aria-hidden="true" />
      <span>{label}</span>
    </div>
  );
}

export function ErrorState({
  title = "Something went wrong",
  description,
  onRetry,
}: {
  title?: string;
  description: string;
  onRetry?: () => void;
}) {
  return (
    <section className="ms-state ms-error-state" role="alert">
      <h2>{title}</h2>
      <p>{description}</p>
      {onRetry ? <Button variant="secondary" onClick={onRetry}>Try again</Button> : null}
    </section>
  );
}

export function Progress({
  value,
  label,
}: {
  value: number;
  label: string;
}) {
  const boundedValue = Number.isFinite(value) ? Math.max(0, Math.min(100, value)) : 0;

  return (
    <div className="ms-progress">
      <div className="ms-progress__label">
        <span>{label}</span>
        <span>{boundedValue}%</span>
      </div>
      <progress className="ms-progress__track" max={100} value={boundedValue} aria-label={label}>
        {boundedValue}%
      </progress>
    </div>
  );
}
