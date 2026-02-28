/**
 * Sensor monitoring page — LibreHardwareMonitor (LHM) dashboard.
 *
 * Displays real-time temperature, fan RPM, voltage, power, and clock data
 * from LHM's WMI interface. Requires LHM to be running as admin.
 */
import { motion } from "motion/react";
import {
  Activity,
  Cpu,
  ExternalLink,
  Fan,
  Gauge,
  Loader2,
  RefreshCw,
  Thermometer,
  Zap,
} from "lucide-react";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { useLhmData } from "@/hooks/use-lhm-data";
import { spring, staggerContainer, staggerItem } from "@/lib/motion";
import type { LhmSensor } from "@/lib/types";

export default function SensorPage() {
  const { snapshot, isAvailable, loading, error, refresh } = useLhmData();

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center text-muted-foreground">
        <motion.div
          animate={{ rotate: 360 }}
          transition={{ repeat: Infinity, duration: 1, ease: "linear" }}
        >
          <Loader2 className="h-6 w-6" />
        </motion.div>
      </div>
    );
  }

  if (!isAvailable) {
    return <LhmUnavailable error={error} onRetry={refresh} />;
  }

  return (
    <motion.div
      variants={staggerContainer}
      initial="initial"
      animate="animate"
      className="flex flex-col gap-5"
    >
      {/* ── Header ──────────────────────────────────────────── */}
      <motion.div variants={staggerItem} transition={spring.soft}>
        <div className="flex items-center gap-2">
          <h1 className="flex items-center gap-2 text-xl font-semibold text-foreground">
            <Activity className="h-5 w-5" />
            传感器监控
          </h1>
          <button
            type="button"
            onClick={refresh}
            className="ml-auto rounded-md border border-border p-1.5 text-muted-foreground transition-colors hover:text-foreground"
            title="刷新"
          >
            <RefreshCw className="h-3.5 w-3.5" />
          </button>
        </div>
        <p className="mt-1 text-sm text-muted-foreground">
          LibreHardwareMonitor 实时数据 — 每 2 秒自动刷新
        </p>
      </motion.div>

      {/* ── Error banner ────────────────────────────────────── */}
      {error && (
        <motion.div
          variants={staggerItem}
          transition={spring.soft}
          className="rounded-lg border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive"
        >
          {error}
        </motion.div>
      )}

      {/* ── Temperature ─────────────────────────────────────── */}
      {snapshot.temperatures.length > 0 && (
        <motion.div variants={staggerItem} transition={spring.soft}>
          <SensorCard
            title="温度"
            icon={<Thermometer className="h-4 w-4" />}
            sensors={snapshot.temperatures}
            unit="°C"
            colorFn={tempColor}
          />
        </motion.div>
      )}

      {/* ── Fans ────────────────────────────────────────────── */}
      {snapshot.fans.length > 0 && (
        <motion.div variants={staggerItem} transition={spring.soft}>
          <SensorCard
            title="风扇转速"
            icon={<Fan className="h-4 w-4" />}
            sensors={snapshot.fans}
            unit="RPM"
            decimals={0}
            colorFn={fanColor}
          />
        </motion.div>
      )}

      {/* ── Voltages ────────────────────────────────────────── */}
      {snapshot.voltages.length > 0 && (
        <motion.div variants={staggerItem} transition={spring.soft}>
          <SensorCard
            title="电压"
            icon={<Zap className="h-4 w-4" />}
            sensors={snapshot.voltages}
            unit="V"
            decimals={3}
          />
        </motion.div>
      )}

      {/* ── Power ───────────────────────────────────────────── */}
      {snapshot.powers.length > 0 && (
        <motion.div variants={staggerItem} transition={spring.soft}>
          <SensorCard
            title="功耗"
            icon={<Gauge className="h-4 w-4" />}
            sensors={snapshot.powers}
            unit="W"
            decimals={1}
          />
        </motion.div>
      )}

      {/* ── Clocks ──────────────────────────────────────────── */}
      {snapshot.clocks.length > 0 && (
        <motion.div variants={staggerItem} transition={spring.soft}>
          <SensorCard
            title="频率"
            icon={<Cpu className="h-4 w-4" />}
            sensors={snapshot.clocks}
            unit="MHz"
            decimals={0}
          />
        </motion.div>
      )}

      {/* ── Loads ───────────────────────────────────────────── */}
      {snapshot.loads.length > 0 && (
        <motion.div variants={staggerItem} transition={spring.soft}>
          <SensorCard
            title="负载"
            icon={<Gauge className="h-4 w-4" />}
            sensors={snapshot.loads}
            unit="%"
            decimals={1}
            colorFn={loadColor}
          />
        </motion.div>
      )}

      {/* ── Controls ────────────────────────────────────────── */}
      {snapshot.controls.length > 0 && (
        <motion.div variants={staggerItem} transition={spring.soft}>
          <SensorCard
            title="风扇控制"
            icon={<Fan className="h-4 w-4" />}
            sensors={snapshot.controls}
            unit="%"
            decimals={1}
          />
        </motion.div>
      )}
    </motion.div>
  );
}

// ─── Sensor card component ────────────────────────────────────

interface SensorCardProps {
  title: string;
  icon: React.ReactNode;
  sensors: LhmSensor[];
  unit: string;
  decimals?: number;
  colorFn?: (value: number) => string;
}

function SensorCard({
  title,
  icon,
  sensors,
  unit,
  decimals = 1,
  colorFn,
}: SensorCardProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          {icon}
          {title}
          <span className="ml-auto text-xs font-normal text-muted-foreground">
            {sensors.length} 个
          </span>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {sensors.map((s) => (
            <div
              key={s.identifier}
              className="flex items-center justify-between rounded-lg border px-4 py-3"
            >
              <div className="flex flex-col gap-0.5">
                <span className="text-sm text-foreground">{s.name}</span>
                <span className="text-xs text-muted-foreground">
                  {s.min.toFixed(decimals)} ~ {s.max.toFixed(decimals)} {unit}
                </span>
              </div>
              <span
                className={`text-lg font-semibold tabular-nums ${
                  colorFn ? colorFn(s.value) : "text-foreground"
                }`}
              >
                {s.value.toFixed(decimals)}
                <span className="ml-0.5 text-xs font-normal text-muted-foreground">
                  {unit}
                </span>
              </span>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}

// ─── Color helpers ────────────────────────────────────────────

function tempColor(v: number): string {
  if (v >= 85) return "text-destructive";
  if (v >= 70) return "text-amber-500";
  return "text-emerald-500";
}

function fanColor(v: number): string {
  if (v === 0) return "text-muted-foreground";
  if (v >= 2000) return "text-amber-500";
  return "text-foreground";
}

function loadColor(v: number): string {
  if (v >= 90) return "text-destructive";
  if (v >= 70) return "text-amber-500";
  return "text-emerald-500";
}

// ─── LHM unavailable view ─────────────────────────────────────

function LhmUnavailable({
  error,
  onRetry,
}: {
  error: string | null;
  onRetry: () => void;
}) {
  return (
    <motion.div
      variants={staggerContainer}
      initial="initial"
      animate="animate"
      className="flex flex-col gap-5"
    >
      <motion.div variants={staggerItem} transition={spring.soft}>
        <h1 className="flex items-center gap-2 text-xl font-semibold text-foreground">
          <Activity className="h-5 w-5" />
          传感器监控
        </h1>
      </motion.div>

      <motion.div
        variants={staggerItem}
        transition={spring.soft}
        className="rounded-lg border border-border bg-card px-6 py-10 text-center"
      >
        <Activity className="mx-auto mb-4 h-10 w-10 text-muted-foreground/50" />
        <h2 className="text-lg font-medium text-foreground">
          LibreHardwareMonitor 未检测到
        </h2>
        <p className="mt-2 text-sm text-muted-foreground">
          请安装并以管理员身份运行{" "}
          <a
            href="https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/releases"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-1 text-primary underline underline-offset-2 hover:text-primary/80"
          >
            LibreHardwareMonitor
            <ExternalLink className="h-3 w-3" />
          </a>
          ，然后点击重试。
        </p>
        {error && (
          <p className="mt-3 font-mono text-xs text-destructive">{error}</p>
        )}
        <button
          type="button"
          onClick={onRetry}
          className="mt-4 rounded-md border border-primary/40 bg-primary/10 px-4 py-1.5 text-sm text-primary transition-colors hover:bg-primary/20"
        >
          重试检测
        </button>
      </motion.div>
    </motion.div>
  );
}
