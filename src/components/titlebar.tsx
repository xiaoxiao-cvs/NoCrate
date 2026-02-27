import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X } from "lucide-react";
import { motion } from "motion/react";
import { useCallback } from "react";
import { cn } from "@/lib/utils";
import { interactive, spring } from "@/lib/motion";

const appWindow = getCurrentWindow();

function WindowButton({
  onClick,
  label,
  variant = "default",
  children,
}: {
  onClick: () => void;
  label: string;
  variant?: "default" | "destructive";
  children: React.ReactNode;
}) {
  return (
    <motion.button
      type="button"
      onClick={onClick}
      aria-label={label}
      whileHover={{
        ...interactive.whileHover,
        backgroundColor:
          variant === "destructive"
            ? "hsl(0 84% 60%)"
            : "var(--color-muted)",
      }}
      whileTap={interactive.whileTap}
      transition={spring.default}
      className={cn(
        "inline-flex h-8 w-10 items-center justify-center rounded-md",
        "text-muted-foreground transition-colors",
        variant === "destructive" && "hover:text-white",
      )}
    >
      {children}
    </motion.button>
  );
}

export function Titlebar() {
  const handleMinimize = useCallback(() => {
    void appWindow.minimize();
  }, []);

  const handleMaximize = useCallback(() => {
    void appWindow.toggleMaximize();
  }, []);

  const handleClose = useCallback(() => {
    void appWindow.close();
  }, []);

  return (
    <header
      className="flex h-10 shrink-0 items-center border-b border-border bg-background"
      data-tauri-drag-region
    >
      {/* App title */}
      <div className="flex items-center gap-2 px-4" data-tauri-drag-region>
        <span
          className="text-sm font-semibold tracking-tight text-foreground"
          data-tauri-drag-region
        >
          NoCrate
        </span>
      </div>

      {/* Spacer — draggable */}
      <div className="flex-1" data-tauri-drag-region />

      {/* Window controls */}
      <div className="flex items-center">
        <WindowButton onClick={handleMinimize} label="最小化">
          <Minus className="h-3.5 w-3.5" />
        </WindowButton>
        <WindowButton onClick={handleMaximize} label="最大化">
          <Square className="h-3 w-3" />
        </WindowButton>
        <WindowButton
          onClick={handleClose}
          label="关闭"
          variant="destructive"
        >
          <X className="h-3.5 w-3.5" />
        </WindowButton>
      </div>
    </header>
  );
}
