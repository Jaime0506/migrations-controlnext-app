import { XCircle, AlertTriangle, Info, X } from "lucide-react";
import { ReactNode } from "react";

interface ErrorBannerProps {
  title?: string;
  message?: string | ReactNode;
  variant?: "error" | "warning" | "info";
  code?: string | number;
  onDismiss?: () => void;
  action?: {
    label: string;
    onClick: () => void;
  };
  className?: string;
}

export default function ErrorBanner({
  title,
  message,
  variant = "error",
  code,
  onDismiss,
  action,
  className = "",
}: ErrorBannerProps) {
  const styles = {
    error: {
      border: "border-rose-500/30",
      bg: "bg-rose-950/20",
      text: "text-rose-200",
      titleText: "text-rose-300",
      icon: <XCircle className="size-4.5 text-rose-400 shrink-0 mt-0.5" />,
      badge: "bg-rose-500/15 text-rose-300 border-rose-500/30",
    },
    warning: {
      border: "border-amber-500/30",
      bg: "bg-amber-950/20",
      text: "text-amber-200",
      titleText: "text-amber-300",
      icon: <AlertTriangle className="size-4.5 text-amber-400 shrink-0 mt-0.5" />,
      badge: "bg-amber-500/15 text-amber-300 border-amber-500/30",
    },
    info: {
      border: "border-cerulean-500/30",
      bg: "bg-cerulean-950/20",
      text: "text-cerulean-200",
      titleText: "text-cerulean-300",
      icon: <Info className="size-4.5 text-cerulean-400 shrink-0 mt-0.5" />,
      badge: "bg-cerulean-500/15 text-cerulean-300 border-cerulean-500/30",
    },
  }[variant];

  return (
    <div
      role="alert"
      className={`relative flex items-start gap-3 p-3.5 rounded-lg border backdrop-blur-md transition-all ${styles.bg} ${styles.border} ${className}`}
    >
      {styles.icon}
      
      <div className="flex-1 min-w-0 space-y-1 text-xs">
        <div className="flex items-center gap-2 flex-wrap">
          {title && (
            <span className={`font-semibold tracking-wide ${styles.titleText}`}>
              {title}
            </span>
          )}
          {code && (
            <span className={`px-1.5 py-0.5 rounded font-mono text-[10px] font-medium border ${styles.badge}`}>
              Código: {code}
            </span>
          )}
        </div>

        {message && (
          <div className={`font-mono text-[11px] leading-relaxed wrap-break-word opacity-95 ${styles.text}`}>
            {message}
          </div>
        )}

        {action && (
          <div className="pt-1.5">
            <button
              type="button"
              onClick={action.onClick}
              className="text-[11px] font-medium underline underline-offset-2 hover:opacity-80 transition-opacity cursor-pointer text-cerulean-300"
            >
              {action.label}
            </button>
          </div>
        )}
      </div>

      {onDismiss && (
        <button
          type="button"
          onClick={onDismiss}
          className="p-1 rounded text-ink-black-400 hover:text-white transition-colors cursor-pointer"
          aria-label="Cerrar alerta"
        >
          <X className="size-3.5" />
        </button>
      )}
    </div>
  );
}
