import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X } from "lucide-react";
import { motion } from "motion/react";

const appWindow = getCurrentWindow();

export function Titlebar() {
  return (
    <header
      className="flex h-8 shrink-0 items-center justify-between border-b border-border bg-bg-primary"
      data-tauri-drag-region
    >
      {/* App title */}
      <div className="flex items-center gap-2 pl-3" data-tauri-drag-region>
        <span
          className="text-xs font-medium tracking-wide text-text-secondary"
          data-tauri-drag-region
        >
          NoCrate
        </span>
      </div>

      {/* Window controls */}
      <div className="flex h-full">
        <WindowButton
          icon={<Minus size={14} />}
          label="最小化"
          onClick={() => appWindow.minimize()}
        />
        <WindowButton
          icon={<Square size={11} />}
          label="最大化"
          onClick={() => appWindow.toggleMaximize()}
        />
        <WindowButton
          icon={<X size={14} />}
          label="关闭"
          onClick={() => appWindow.close()}
          variant="close"
        />
      </div>
    </header>
  );
}

function WindowButton({
  icon,
  label,
  onClick,
  variant = "default",
}: {
  icon: React.ReactNode;
  label: string;
  onClick: () => void;
  variant?: "default" | "close";
}) {
  return (
    <motion.button
      aria-label={label}
      className={`inline-flex h-full w-11 items-center justify-center text-text-secondary transition-colors ${
        variant === "close"
          ? "hover:bg-destructive hover:text-destructive-foreground"
          : "hover:bg-bg-tertiary"
      }`}
      onClick={onClick}
      whileHover={{ scale: 1.05 }}
      whileTap={{ scale: 0.95 }}
      transition={{ type: "spring", stiffness: 400, damping: 25 }}
    >
      {icon}
    </motion.button>
  );
}
