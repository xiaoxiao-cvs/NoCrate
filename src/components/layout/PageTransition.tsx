import { AnimatePresence, motion } from "motion/react";
import { useLocation } from "react-router";

export function PageTransition({ children }: { children: React.ReactNode }) {
  const location = useLocation();

  return (
    <AnimatePresence mode="wait">
      <motion.div
        key={location.pathname}
        animate={{ opacity: 1, y: 0 }}
        className="h-full w-full"
        exit={{ opacity: 0 }}
        initial={{ opacity: 0, y: 8 }}
        transition={{ duration: 0.15, ease: "easeOut" }}
      >
        {children}
      </motion.div>
    </AnimatePresence>
  );
}
