import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { AnimatePresence, motion } from "motion/react";
import { CheckCircle, X, XCircle, Info, AlertTriangle } from "lucide-react";

import { toastVariants, spring, tween } from "@/lib/motion";
import { cn } from "@/lib/utils";

// ─── Types ───────────────────────────────────────────────────
type ToastType = "success" | "error" | "info" | "warning";

interface Toast {
  id: string;
  type: ToastType;
  title: string;
  description?: string;
  duration?: number;
}

interface ToastContextValue {
  toast: (options: Omit<Toast, "id">) => void;
  success: (title: string, description?: string) => void;
  error: (title: string, description?: string) => void;
  info: (title: string, description?: string) => void;
  warning: (title: string, description?: string) => void;
  dismiss: (id: string) => void;
}

const ToastContext = createContext<ToastContextValue | undefined>(undefined);

let counter = 0;

// ─── Icons ───────────────────────────────────────────────────
const icons: Record<ToastType, typeof CheckCircle> = {
  success: CheckCircle,
  error: XCircle,
  info: Info,
  warning: AlertTriangle,
};

const typeStyles: Record<ToastType, string> = {
  success: "border-foreground/20 text-foreground",
  error: "border-destructive/40 text-destructive",
  info: "border-foreground/20 text-foreground",
  warning: "border-foreground/30 text-foreground",
};

// ─── ToastItem ───────────────────────────────────────────────
function ToastItem({
  toast: t,
  onDismiss,
}: {
  toast: Toast;
  onDismiss: (id: string) => void;
}) {
  const Icon = icons[t.type];

  return (
    <motion.div
      layout
      variants={toastVariants}
      initial="initial"
      animate="animate"
      exit="exit"
      transition={spring.snappy}
      className={cn(
        "pointer-events-auto flex w-80 items-start gap-3 rounded-lg border bg-card p-4 shadow-lg",
        typeStyles[t.type],
      )}
    >
      <Icon className="mt-0.5 h-4 w-4 shrink-0" />
      <div className="flex-1 space-y-1">
        <p className="text-sm font-medium leading-none">{t.title}</p>
        {t.description && (
          <p className="text-xs text-muted-foreground">{t.description}</p>
        )}
      </div>
      <motion.button
        whileHover={{ scale: 1.1 }}
        whileTap={{ scale: 0.9 }}
        transition={tween.micro}
        onClick={() => onDismiss(t.id)}
        className="shrink-0 cursor-pointer rounded p-0.5 text-muted-foreground hover:text-foreground"
      >
        <X className="h-3.5 w-3.5" />
      </motion.button>
    </motion.div>
  );
}

// ─── Provider ────────────────────────────────────────────────
const DEFAULT_DURATION = 4000;

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const dismiss = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const addToast = useCallback(
    (options: Omit<Toast, "id">) => {
      const id = `toast-${++counter}`;
      const duration = options.duration ?? DEFAULT_DURATION;
      setToasts((prev) => [...prev, { ...options, id }]);
      if (duration > 0) {
        setTimeout(() => dismiss(id), duration);
      }
    },
    [dismiss],
  );

  const value = useMemo<ToastContextValue>(
    () => ({
      toast: addToast,
      success: (title, description) =>
        addToast({ type: "success", title, description }),
      error: (title, description) =>
        addToast({ type: "error", title, description }),
      info: (title, description) =>
        addToast({ type: "info", title, description }),
      warning: (title, description) =>
        addToast({ type: "warning", title, description }),
      dismiss,
    }),
    [addToast, dismiss],
  );

  return (
    <ToastContext.Provider value={value}>
      {children}
      {/* Toast container — top-right, below titlebar */}
      <div className="pointer-events-none fixed right-4 top-12 z-50 flex flex-col gap-2">
        <AnimatePresence mode="popLayout">
          {toasts.map((t) => (
            <ToastItem key={t.id} toast={t} onDismiss={dismiss} />
          ))}
        </AnimatePresence>
      </div>
    </ToastContext.Provider>
  );
}

export function useToast(): ToastContextValue {
  const ctx = useContext(ToastContext);
  if (!ctx) {
    throw new Error("useToast must be used within a ToastProvider");
  }
  return ctx;
}
