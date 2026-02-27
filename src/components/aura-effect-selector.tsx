/**
 * AURA effect mode selector with speed control.
 */
import { motion } from "motion/react";

import { spring } from "@/lib/motion";
import { cn } from "@/lib/utils";
import {
  AURA_EFFECTS,
  AURA_SPEEDS,
  type AuraEffect,
  type AuraSpeed,
} from "@/lib/aura-commands";

export interface AuraEffectSelectorProps {
  effect: AuraEffect;
  speed: AuraSpeed;
  onEffectChange: (effect: AuraEffect) => void;
  onSpeedChange: (speed: AuraSpeed) => void;
  disabled?: boolean;
}

export function AuraEffectSelector({
  effect,
  speed,
  onEffectChange,
  onSpeedChange,
  disabled,
}: AuraEffectSelectorProps) {
  return (
    <div className="flex flex-col gap-4">
      {/* ── Effect grid ────────────────────────────────────── */}
      <div className="grid grid-cols-3 gap-2">
        {AURA_EFFECTS.map((e) => {
          const isActive = e.id === effect;
          return (
            <button
              key={e.id}
              type="button"
              disabled={disabled}
              onClick={() => onEffectChange(e.id)}
              className={cn(
                "relative rounded-lg border px-3 py-2 text-sm font-medium transition-colors",
                "cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                "disabled:pointer-events-none disabled:opacity-50",
                isActive
                  ? "border-foreground/20 text-foreground"
                  : "border-border text-muted-foreground hover:border-foreground/10 hover:text-foreground",
              )}
            >
              {isActive && (
                <motion.div
                  layoutId="aura-effect-indicator"
                  className="absolute inset-0 rounded-lg bg-muted"
                  transition={spring.snappy}
                  style={{ zIndex: 0 }}
                />
              )}
              <span className="relative z-10">{e.label}</span>
            </button>
          );
        })}
      </div>

      {/* ── Speed selector ─────────────────────────────────── */}
      {effect !== "off" && (
        <div className="flex items-center gap-2">
          <span className="text-xs text-muted-foreground">速度</span>
          <div className="flex gap-1">
            {AURA_SPEEDS.map((s) => (
              <button
                key={s.id}
                type="button"
                disabled={disabled}
                onClick={() => onSpeedChange(s.id)}
                className={cn(
                  "rounded-md px-3 py-1 text-xs font-medium transition-colors",
                  "cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                  "disabled:pointer-events-none disabled:opacity-50",
                  s.id === speed
                    ? "bg-foreground text-background"
                    : "bg-muted text-muted-foreground hover:text-foreground",
                )}
              >
                {s.label}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
