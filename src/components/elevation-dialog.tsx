import { useEffect, useState } from "react";
import { motion, AnimatePresence } from "motion/react";
import { ShieldAlert, ShieldCheck, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { spring } from "@/lib/motion";
import { restartAsAdmin } from "@/lib/system-commands";

interface ElevationDialogProps {
  /** Called when the user dismisses the dialog (continue without admin). */
  onDismiss: () => void;
}

/**
 * Modal dialog prompting the user to restart with administrator privileges.
 * Similar to Armoury Crate's UAC confirmation flow.
 */
export function ElevationDialog({ onDismiss }: ElevationDialogProps) {
  const [status, setStatus] = useState<"idle" | "requesting" | "denied">(
    "idle",
  );

  const handleElevate = async () => {
    setStatus("requesting");
    try {
      await restartAsAdmin();
      // If we get here, the app is about to exit — just wait.
    } catch {
      // User cancelled the UAC prompt
      setStatus("denied");
    }
  };

  // Auto-dismiss the "denied" hint after a few seconds
  useEffect(() => {
    if (status === "denied") {
      const t = setTimeout(() => setStatus("idle"), 3000);
      return () => clearTimeout(t);
    }
  }, [status]);

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm"
      >
        <motion.div
          initial={{ opacity: 0, scale: 0.92, y: 20 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.92, y: 20 }}
          transition={spring.soft}
          className="relative mx-4 w-full max-w-md rounded-xl border border-border bg-card p-6 shadow-2xl"
        >
          {/* Close button */}
          <button
            onClick={onDismiss}
            className="absolute right-3 top-3 rounded-md p-1 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          >
            <X className="h-4 w-4" />
          </button>

          {/* Icon */}
          <div className="mb-4 flex justify-center">
            <div className="flex h-14 w-14 items-center justify-center rounded-full bg-amber-500/10">
              <ShieldAlert className="h-7 w-7 text-amber-500" />
            </div>
          </div>

          {/* Title */}
          <h2 className="mb-2 text-center text-lg font-semibold">
            需要管理员权限
          </h2>

          {/* Description */}
          <p className="mb-1 text-center text-sm text-muted-foreground">
            NoCrate 需要管理员权限来访问硬件控制接口：
          </p>
          <ul className="mb-5 space-y-1 text-center text-xs text-muted-foreground">
            <li className="flex items-center justify-center gap-1.5">
              <ShieldCheck className="h-3 w-3 text-emerald-500" />
              <span>WMI 风扇转速监控与温控策略切换</span>
            </li>
            <li className="flex items-center justify-center gap-1.5">
              <ShieldCheck className="h-3 w-3 text-emerald-500" />
              <span>USB HID 设备的 AURA RGB 灯效控制</span>
            </li>
          </ul>

          {/* Denied hint */}
          <AnimatePresence>
            {status === "denied" && (
              <motion.p
                initial={{ opacity: 0, height: 0 }}
                animate={{ opacity: 1, height: "auto" }}
                exit={{ opacity: 0, height: 0 }}
                className="mb-3 text-center text-xs text-destructive"
              >
                UAC 提权被取消，请重试或以受限模式继续
              </motion.p>
            )}
          </AnimatePresence>

          {/* Actions */}
          <div className="flex gap-3">
            <Button
              variant="outline"
              size="md"
              className="flex-1"
              onClick={onDismiss}
            >
              以受限模式继续
            </Button>
            <Button
              variant="default"
              size="md"
              className="flex-1"
              onClick={handleElevate}
              disabled={status === "requesting"}
            >
              {status === "requesting" ? "正在请求…" : "以管理员重启"}
            </Button>
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}
