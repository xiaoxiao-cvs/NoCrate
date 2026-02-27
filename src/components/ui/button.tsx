import { motion, type HTMLMotionProps } from "motion/react";

import { interactive, spring } from "@/lib/motion";
import { cn } from "@/lib/utils";

const variants = {
  default:
    "bg-primary text-primary-foreground hover:bg-primary/90",
  secondary:
    "bg-secondary text-secondary-foreground hover:bg-secondary/80",
  outline:
    "border border-border bg-background hover:bg-muted",
  ghost: "hover:bg-muted hover:text-foreground",
  destructive:
    "bg-destructive text-destructive-foreground hover:bg-destructive/90",
} as const;

const sizes = {
  sm: "h-8 rounded-md px-3 text-xs",
  md: "h-9 rounded-lg px-4 text-sm",
  lg: "h-11 rounded-lg px-6 text-base",
  icon: "h-9 w-9 rounded-lg",
} as const;

export interface ButtonProps
  extends Omit<HTMLMotionProps<"button">, "ref"> {
  variant?: keyof typeof variants;
  size?: keyof typeof sizes;
}

export function Button({
  variant = "default",
  size = "md",
  className,
  disabled,
  children,
  ...props
}: ButtonProps) {
  return (
    <motion.button
      whileHover={disabled ? undefined : interactive.whileHover}
      whileTap={disabled ? undefined : interactive.whileTap}
      transition={spring.default}
      className={cn(
        "inline-flex cursor-pointer items-center justify-center font-medium transition-colors",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background",
        "disabled:pointer-events-none disabled:opacity-50",
        variants[variant],
        sizes[size],
        className,
      )}
      disabled={disabled}
      {...props}
    >
      {children}
    </motion.button>
  );
}
