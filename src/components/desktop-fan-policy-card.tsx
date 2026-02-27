/**
 * Card showing a desktop fan header's policy with inline editing + curve view.
 *
 * Each card displays: fan name, mode (PWM/DC/AUTO), profile (MANUAL/STANDARD),
 * temperature source, low RPM limit, and an 8-point fan curve editor.
 */
import { motion } from "motion/react";
import { Fan, Gauge, RotateCcw, Save, Thermometer, Zap } from "lucide-react";
import { useCallback, useState } from "react";

import { FanCurveEditor } from "@/components/fan-curve-editor";
import { cn } from "@/lib/utils";
import {
  DESKTOP_FAN_NAMES,
  type DesktopFanCurve,
  type DesktopFanMode,
  type DesktopFanPolicy,
  type FanCurvePoint,
  type SioFanReading,
} from "@/lib/types";

export interface DesktopFanPolicyCardProps {
  policy: DesktopFanPolicy;
  /** 当前模式下的曲线数据（可能尚未加载）。 */
  curve: DesktopFanCurve | undefined;
  /** Super I/O 风扇转速读数 */
  sioFans: SioFanReading[];
  onUpdate: (policy: DesktopFanPolicy) => void;
  onLoadCurve: (fanType: number, mode: DesktopFanMode) => void;
  onSaveCurve: (fanType: number, mode: DesktopFanMode, points: FanCurvePoint[]) => void;
}

const MODE_OPTIONS: { value: DesktopFanMode; label: string; description: string }[] = [
  { value: "PWM", label: "PWM", description: "脉宽调制" },
  { value: "DC", label: "DC", description: "电压控制" },
  { value: "AUTO", label: "AUTO", description: "自动选择" },
];

const PROFILE_OPTIONS = [
  { value: "STANDARD" as const, label: "标准", description: "默认曲线" },
  { value: "MANUAL" as const, label: "手动", description: "自定义" },
];

export function DesktopFanPolicyCard({
  policy,
  curve,
  sioFans,
  onUpdate,
  onLoadCurve,
  onSaveCurve,
}: DesktopFanPolicyCardProps) {
  const fanName =
    DESKTOP_FAN_NAMES[policy.fan_type] ?? `风扇 ${policy.fan_type}`;

  // 编辑中的曲线点（只有用户拖拽修改后才会有值）
  const [editingPoints, setEditingPoints] = useState<FanCurvePoint[] | null>(null);
  const isDirty = editingPoints !== null;

  // 根据 fan_type 索引在 SIO 风扇列表中查找对应通道的 RPM
  const rpmReading = sioFans.find((f) => f.channel === policy.fan_type);

  // 当前显示的曲线点：优先编辑中的，否则用硬件读取的
  const displayPoints = editingPoints ?? curve?.points ?? null;

  // 切换 Mode 时重新加载对应曲线
  const handleModeChange = useCallback(
    (mode: DesktopFanMode) => {
      setEditingPoints(null); // 清除编辑中的修改
      onUpdate({ ...policy, mode });
      onLoadCurve(policy.fan_type, mode);
    },
    [policy, onUpdate, onLoadCurve],
  );

  // 保存曲线
  const handleSave = useCallback(() => {
    if (!editingPoints) return;
    onSaveCurve(policy.fan_type, policy.mode, editingPoints);
    setEditingPoints(null);
  }, [editingPoints, policy, onSaveCurve]);

  // 放弃编辑
  const handleDiscard = useCallback(() => {
    setEditingPoints(null);
  }, []);

  return (
    <div className="flex flex-col gap-3 rounded-xl border border-border bg-card p-4">
      {/* Header */}
      <div className="flex items-center gap-2">
        <Fan className="h-4 w-4 text-primary" />
        <span className="font-medium text-foreground">{fanName}</span>
        {rpmReading !== undefined && (
          <span className="ml-auto flex items-center gap-1 text-sm font-mono tabular-nums text-foreground">
            <motion.span
              key={rpmReading.rpm}
              initial={{ opacity: 0.4, y: -4 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.2 }}
            >
              {rpmReading.rpm.toLocaleString()}
            </motion.span>
            <span className="text-xs text-muted-foreground">RPM</span>
          </span>
        )}
        {rpmReading === undefined && (
          <span className="ml-auto text-xs text-muted-foreground">
            #{policy.fan_type}
          </span>
        )}
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
              onClick={() => handleModeChange(opt.value)}
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

      {/* Fan curve editor */}
      {displayPoints && (
        <div className="flex flex-col gap-2">
          <div className="flex items-center justify-between">
            <span className="text-xs font-medium text-muted-foreground">
              风扇曲线 ({policy.mode})
            </span>
            {isDirty && (
              <div className="flex gap-1.5">
                <button
                  type="button"
                  onClick={handleDiscard}
                  className="flex items-center gap-1 rounded-md border border-border px-2 py-0.5 text-xs text-muted-foreground transition-colors hover:text-foreground"
                >
                  <RotateCcw className="h-3 w-3" />
                  撤销
                </button>
                <button
                  type="button"
                  onClick={handleSave}
                  className="flex items-center gap-1 rounded-md border border-primary/40 bg-primary/10 px-2 py-0.5 text-xs text-primary transition-colors hover:bg-primary/20"
                >
                  <Save className="h-3 w-3" />
                  应用
                </button>
              </div>
            )}
          </div>
          <FanCurveEditor
            points={displayPoints}
            onChange={setEditingPoints}
          />
        </div>
      )}

      {/* No curve loaded placeholder */}
      {!displayPoints && (
        <div className="rounded-lg border border-dashed border-border py-4 text-center text-xs text-muted-foreground">
          曲线数据加载中…
        </div>
      )}
    </div>
  );
}
