import { Outlet, useLocation } from "react-router";
import { AnimatePresence, motion } from "motion/react";
import { Titlebar } from "@/components/titlebar";
import { Sidebar } from "@/components/sidebar";
import { pageVariants, pageTransition } from "@/lib/motion";

export function AppLayout() {
  const location = useLocation();

  return (
    <div className="flex h-screen w-screen flex-col overflow-hidden">
      <Titlebar />
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
