import { Outlet, useLocation } from "react-router";
import { AnimatePresence, motion } from "motion/react";
import { ShieldAlert } from "lucide-react";
import { Titlebar } from "@/components/titlebar";
import { Sidebar } from "@/components/sidebar";
import { ElevationDialog } from "@/components/elevation-dialog";
import { pageVariants, pageTransition, spring } from "@/lib/motion";
import { useAdminStatus } from "@/hooks/use-admin-status";
import { useLhmData } from "@/hooks/use-lhm-data";
import { useTempAlerts } from "@/hooks/use-temp-alerts";
import { restartAsAdmin } from "@/lib/system-commands";
import { useState } from "react";

function AdminBanner() {
  const [requesting, setRequesting] = useState(false);

  const handleClick = async () => {
    setRequesting(true);
    try {
      await restartAsAdmin();
    } catch {
      setRequesting(false);
    }
  };

  return (
    <motion.div
      initial={{ opacity: 0, height: 0 }}
      animate={{ opacity: 1, height: "auto" }}
      transition={spring.soft}
      className="flex items-center gap-2 border-b border-destructive/30 bg-destructive/10 px-4 py-2 text-xs text-destructive"
    >
      <ShieldAlert className="h-3.5 w-3.5 shrink-0" />
      <span>未以管理员身份运行 — WMI 风扇控制和 USB HID 灯效可能不可用</span>
      <button
        onClick={handleClick}
        disabled={requesting}
        className="ml-auto shrink-0 rounded-md border border-destructive/30 px-2 py-0.5 text-xs font-medium text-destructive transition-colors hover:bg-destructive/10 disabled:opacity-50"
      >
        {requesting ? "正在请求…" : "以管理员重启"}
      </button>
    </motion.div>
  );
}

export function AppLayout() {
  const location = useLocation();
  const isAdmin = useAdminStatus();
  const [elevationDismissed, setElevationDismissed] = useState(false);

  // Global LHM polling — drives temperature alerts
  const { snapshot } = useLhmData();
  useTempAlerts(snapshot);

  const showDialog = isAdmin === false && !elevationDismissed;
  const showBanner = isAdmin === false && elevationDismissed;

  return (
    <div className="flex h-screen w-screen flex-col overflow-hidden">
      <Titlebar />
      {showBanner && <AdminBanner />}
      {showDialog && <ElevationDialog onDismiss={() => setElevationDismissed(true)} />}
      <div className="flex flex-1 overflow-hidden">
        <Sidebar />
        <main className="flex-1 overflow-y-auto">
          <AnimatePresence mode="wait">
            <motion.div
              key={location.pathname}
              variants={pageVariants}
              initial="initial"
              animate="animate"
              exit="exit"
              transition={pageTransition}
              className="h-full p-6"
            >
              <Outlet />
            </motion.div>
          </AnimatePresence>
        </main>
      </div>
    </div>
  );
}
