import { Outlet, useLocation } from "react-router";
import { AnimatePresence, motion } from "motion/react";
import { ShieldAlert } from "lucide-react";
import { Titlebar } from "@/components/titlebar";
import { Sidebar } from "@/components/sidebar";
import { pageVariants, pageTransition, spring } from "@/lib/motion";
import { useAdminStatus } from "@/hooks/use-admin-status";

function AdminBanner() {
  return (
    <motion.div
      initial={{ opacity: 0, height: 0 }}
      animate={{ opacity: 1, height: "auto" }}
      transition={spring.soft}
      className="flex items-center gap-2 border-b border-destructive/30 bg-destructive/10 px-4 py-2 text-xs text-destructive"
    >
      <ShieldAlert className="h-3.5 w-3.5 shrink-0" />
      <span>未以管理员身份运行 — WMI 风扇控制和 USB HID 灯效可能不可用</span>
    </motion.div>
  );
}

export function AppLayout() {
  const location = useLocation();
  const isAdmin = useAdminStatus();

  return (
    <div className="flex h-screen w-screen flex-col overflow-hidden">
      <Titlebar />
      {isAdmin === false && <AdminBanner />}
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
