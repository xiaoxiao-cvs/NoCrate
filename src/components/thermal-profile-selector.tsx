/**
 * Thermal-profile picker — three selectable cards with a shared
 * Motion `layoutId` indicator for smooth transitions.
 */
import { motion } from "motion/react";
import { Flame, Snowflake, Wind } from "lucide-react";
import type { ComponentType } from "react";

import { spring } from "@/lib/motion";
import { cn } from "@/lib/utils";
import { THERMAL_PROFILES, type ThermalProfile } from "@/lib/types";

export interface ThermalProfileSelectorProps {
  active: ThermalProfile;
  onChange: (profile: ThermalProfile) => void;
  disabled?: boolean;
}

const ICONS: Record<ThermalProfile, ComponentType<{ className?: string }>> = {
  silent: Snowflake,
  standard: Wind,
  performance: Flame,
};

export function ThermalProfileSelector({
  active,
  onChange,
  disabled,
}: ThermalProfileSelectorProps) {
  return (
    <div className="flex gap-3">
      {THERMAL_PROFILES.map((p) => {
        const Icon = ICONS[p.id];
        const isActive = p.id === active;

        return (
          <button
            key={p.id}
            type="button"
            disabled={disabled}
            onClick={() => onChange(p.id)}
            className={cn(
              "relative flex flex-1 cursor-pointer flex-col items-center gap-1.5 rounded-xl border p-4 transition-colors",
              "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
              "disabled:pointer-events-none disabled:opacity-50",
              isActive
                ? "border-foreground/20 text-foreground"
                : "border-border text-muted-foreground hover:border-foreground/10 hover:text-foreground",
            )}
          >
            {/* Active indicator background */}
            {isActive && (
              <motion.div
                layoutId="thermal-profile-indicator"
                className="absolute inset-0 rounded-xl bg-muted"
                transition={spring.snappy}
                style={{ zIndex: 0 }}
              />
            )}

            <Icon className="relative z-10 h-5 w-5" />
            <span className="relative z-10 text-sm font-medium">
              {p.label}
            </span>
            <span className="relative z-10 text-[11px] text-muted-foreground">
              {p.description}
            </span>
          </button>
        );
      })}
    </div>
  );
}
