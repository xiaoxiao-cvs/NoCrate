/**
 * Card showing a desktop fan header's policy with inline editing.
 *
 * Each card displays: fan name, mode (PWM/AUTO), profile (MANUAL/STANDARD),
 * temperature source, and low RPM limit. Mode and profile can be toggled.
 */
import { motion } from "motion/react";
import { Fan, Gauge, Thermometer, Zap } from "lucide-react";

import { cn } from "@/lib/utils";
import { DESKTOP_FAN_NAMES, type DesktopFanPolicy } from "@/lib/types";

export interface DesktopFanPolicyCardProps {
  policy: DesktopFanPolicy;
  onUpdate: (policy: DesktopFanPolicy) => void;
}

const MODE_OPTIONS = [
  { value: "PWM" as const, label: "PWM", description: "电压控制" },
  { value: "AUTO" as const, label: "AUTO", description: "自动调速" },
];

const PROFILE_OPTIONS = [
  { value: "STANDARD" as const, label: "标准", description: "默认曲线" },
  { value: "MANUAL" as const, label: "手动", description: "自定义" },
];

export function DesktopFanPolicyCard({
  policy,
  onUpdate,
}: DesktopFanPolicyCardProps) {
  const fanName =
    DESKTOP_FAN_NAMES[policy.fan_type] ?? `风扇 ${policy.fan_type}`;

  return (
    <div className="flex flex-col gap-3 rounded-xl border border-border bg-card p-4">
      {/* Header */}
      <div className="flex items-center gap-2">
        <Fan className="h-4 w-4 text-primary" />
        <span className="font-medium text-foreground">{fanName}</span>
        <span className="ml-auto text-xs text-muted-foreground">
          #{policy.fan_type}
        </span>
      </div>

      {/* Info row */}
      <div className="flex flex-wrap gap-3 text-xs text-muted-foreground">
        <span className="flex items-center gap-1">
          <Thermometer className="h-3 w-3" />
          源: {policy.source || "—"}
        </span>
        <span className="flex items-center gap-1">
          <Gauge className="h-3 w-3" />
          最低: {policy.low_limit} RPM
        </span>
      </div>

      {/* Mode toggle */}
      <div className="flex flex-col gap-1.5">
        <span className="flex items-center gap-1 text-xs font-medium text-muted-foreground">
          <Zap className="h-3 w-3" />
          控制模式
        </span>
        <div className="flex gap-1.5">
          {MODE_OPTIONS.map((opt) => (
            <button
              key={opt.value}
              type="button"
              onClick={() =>
                onUpdate({ ...policy, mode: opt.value })
              }
              className={cn(
                "relative flex-1 rounded-lg border px-2.5 py-1.5 text-xs transition-colors",
                "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                policy.mode === opt.value
                  ? "border-primary/40 text-foreground"
                  : "border-border text-muted-foreground hover:border-foreground/10 hover:text-foreground",
              )}
            >
              {policy.mode === opt.value && (
                <motion.div
                  layoutId={`mode-bg-${policy.fan_type}`}
                  className="absolute inset-0 rounded-lg bg-primary/10"
                  transition={{ type: "spring", stiffness: 400, damping: 30 }}
                />
              )}
              <span className="relative z-10 font-medium">{opt.label}</span>
              <span className="relative z-10 ml-1 opacity-60">
                {opt.description}
              </span>
            </button>
          ))}
        </div>
      </div>

      {/* Profile toggle */}
      <div className="flex flex-col gap-1.5">
        <span className="text-xs font-medium text-muted-foreground">
          风扇策略
        </span>
        <div className="flex gap-1.5">
          {PROFILE_OPTIONS.map((opt) => (
            <button
              key={opt.value}
              type="button"
              onClick={() =>
                onUpdate({ ...policy, profile: opt.value })
              }
              className={cn(
                "relative flex-1 rounded-lg border px-2.5 py-1.5 text-xs transition-colors",
                "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                policy.profile === opt.value
                  ? "border-primary/40 text-foreground"
                  : "border-border text-muted-foreground hover:border-foreground/10 hover:text-foreground",
              )}
            >
              {policy.profile === opt.value && (
                <motion.div
                  layoutId={`profile-bg-${policy.fan_type}`}
                  className="absolute inset-0 rounded-lg bg-primary/10"
                  transition={{ type: "spring", stiffness: 400, damping: 30 }}
                />
              )}
              <span className="relative z-10 font-medium">{opt.label}</span>
              <span className="relative z-10 ml-1 opacity-60">
                {opt.description}
              </span>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
