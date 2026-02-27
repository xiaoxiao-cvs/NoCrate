import type { Transition, Variants } from "motion/react";

/**
 * Centralized animation configuration for NoCrate.
 *
 * All Motion animations reference these constants to ensure
 * consistent timing, easing, and spring physics across the app.
 */

// ─── Spring Presets ──────────────────────────────────────────
export const spring = {
  /** Default interactive spring (buttons, toggles) */
  default: { type: "spring", stiffness: 400, damping: 25 } as Transition,
  /** Soft spring for layout animations */
  soft: { type: "spring", stiffness: 300, damping: 30 } as Transition,
  /** Snappy spring for card selection */
  snappy: { type: "spring", stiffness: 500, damping: 30 } as Transition,
  /** Gentle spring for numeric value changes */
  gentle: { type: "spring", stiffness: 100, damping: 15 } as Transition,
  /** Gauge arc animation */
  gauge: { type: "spring", stiffness: 80, damping: 20 } as Transition,
} as const;

// ─── Tween Presets ───────────────────────────────────────────
export const tween = {
  /** Fast fade/slide for page transitions */
  page: { duration: 0.3, ease: "easeOut" } as Transition,
  /** Quick micro-interaction */
  micro: { duration: 0.15, ease: "easeOut" } as Transition,
} as const;

// ─── Interactive States ──────────────────────────────────────
export const interactive = {
  /** Standard button hover */
  whileHover: { scale: 1.03 },
  /** Standard button tap */
  whileTap: { scale: 0.97 },
} as const;

// ─── Page Transition Variants ────────────────────────────────
export const pageVariants: Variants = {
  initial: { opacity: 0, y: 8 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -8 },
};

export const pageTransition: Transition = tween.page;

// ─── Staggered List Variants ─────────────────────────────────
export const staggerContainer: Variants = {
  animate: {
    transition: { staggerChildren: 0.05 },
  },
};

export const staggerItem: Variants = {
  initial: { opacity: 0, y: 12 },
  animate: { opacity: 1, y: 0 },
};

// ─── Sidebar Variants ────────────────────────────────────────
export const sidebarVariants: Variants = {
  expanded: { width: 200 },
  collapsed: { width: 56 },
};

// ─── Toast Variants ──────────────────────────────────────────
export const toastVariants: Variants = {
  initial: { opacity: 0, y: -100 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -100 },
};
