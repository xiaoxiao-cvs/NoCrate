import { motion } from "motion/react";
import { Fan, Loader2, Monitor } from "lucide-react";
import { useEffect, useState } from "react";

import { DesktopFanPolicyCard } from "@/components/desktop-fan-policy-card";
import { FanCurveEditor } from "@/components/fan-curve-editor";
import { FanGauge } from "@/components/fan-gauge";
import { ThermalProfileSelector } from "@/components/thermal-profile-selector";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { useDesktopFanData } from "@/hooks/use-desktop-fan-data";
import { useFanData } from "@/hooks/use-fan-data";
import { getWmiBackend } from "@/lib/tauri-commands";
import { spring, staggerContainer, staggerItem } from "@/lib/motion";
import { FAN_TARGET_LABELS, type FanCurvePoint, type WmiBackend } from "@/lib/types";

/** Default editable fan curve. */
const DEFAULT_CURVE: FanCurvePoint[] = [
  { temp_c: 30, duty_pct: 30 },
  { temp_c: 40, duty_pct: 35 },
  { temp_c: 50, duty_pct: 45 },
  { temp_c: 60, duty_pct: 55 },
  { temp_c: 70, duty_pct: 65 },
  { temp_c: 75, duty_pct: 75 },
  { temp_c: 80, duty_pct: 85 },
  { temp_c: 90, duty_pct: 100 },
];

export default function FanPage() {
  const [backend, setBackend] = useState<WmiBackend | null>(null);
  const [backendError, setBackendError] = useState<string | null>(null);

  useEffect(() => {
    getWmiBackend()
      .then(setBackend)
      .catch((e) => setBackendError(String(e)));
  }, []);

  // Show loading spinner until backend is detected
  if (!backend && !backendError) {
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

  // Backend detection failed (e.g. not running as admin)
  if (backendError) {
    return (
      <motion.div
        variants={staggerContainer}
        initial="initial"
        animate="animate"
        className="flex flex-col gap-5"
      >
        <motion.div variants={staggerItem} transition={spring.soft}>
          <h1 className="flex items-center gap-2 text-xl font-semibold text-foreground">
            <Fan className="h-5 w-5" />
            风扇控制
          </h1>
        </motion.div>
        <motion.div
          variants={staggerItem}
          transition={spring.soft}
          className="rounded-lg border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive"
        >
          无法连接 WMI 接口，风扇控制不可用。请以管理员权限重新启动应用。
        </motion.div>
      </motion.div>
    );
  }

  return backend === "desktop" ? <DesktopFanView /> : <LaptopFanView />;
}

// ===========================================================================
// Desktop fan view — policy-based control
// ===========================================================================

function DesktopFanView() {
  const { policies, loading, error, updatePolicy } = useDesktopFanData();

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

  return (
    <motion.div
      variants={staggerContainer}
      initial="initial"
      animate="animate"
      className="flex flex-col gap-5"
    >
      {/* ── Header ──────────────────────────────────────────── */}
      <motion.div variants={staggerItem} transition={spring.soft}>
        <h1 className="flex items-center gap-2 text-xl font-semibold text-foreground">
          <Fan className="h-5 w-5" />
          风扇控制
        </h1>
        <p className="mt-1 flex items-center gap-1.5 text-sm text-muted-foreground">
          <Monitor className="h-3.5 w-3.5" />
          桌面主板模式 — 管理各风扇接口的控制策略
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

      {/* ── Fan policy cards ────────────────────────────────── */}
      {policies.length > 0 ? (
        <motion.div
          variants={staggerItem}
          transition={spring.soft}
          className="grid gap-4 sm:grid-cols-2"
        >
          {policies.map((p) => (
            <DesktopFanPolicyCard
              key={p.fan_type}
              policy={p}
              onUpdate={updatePolicy}
            />
          ))}
        </motion.div>
      ) : (
        <motion.div variants={staggerItem} transition={spring.soft}>
          <Card>
            <CardContent className="py-8">
              <p className="text-center text-sm text-muted-foreground">
                未检测到风扇接口。请确认以管理员权限运行且 ASUS WMI 驱动已安装。
              </p>
            </CardContent>
          </Card>
        </motion.div>
      )}
    </motion.div>
  );
}

// ===========================================================================
// Laptop fan view — RPM gauges + thermal profiles + curve editor
// ===========================================================================

function LaptopFanView() {
  const { fans, profile, loading, error, changeProfile } = useFanData();
  const [curvePoints, setCurvePoints] = useState(DEFAULT_CURVE);

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

  return (
    <motion.div
      variants={staggerContainer}
      initial="initial"
      animate="animate"
      className="flex flex-col gap-5"
    >
      {/* ── Header ──────────────────────────────────────────── */}
      <motion.div variants={staggerItem} transition={spring.soft}>
        <h1 className="flex items-center gap-2 text-xl font-semibold text-foreground">
          <Fan className="h-5 w-5" />
          风扇控制
        </h1>
        <p className="mt-1 text-sm text-muted-foreground">
          监控风扇转速，调整温控策略与自定义曲线
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

      {/* ── Fan gauges ──────────────────────────────────────── */}
      <motion.div variants={staggerItem} transition={spring.soft}>
        <Card>
          <CardHeader>
            <CardTitle>实时转速</CardTitle>
          </CardHeader>
          <CardContent>
            {fans.length > 0 ? (
              <div className="flex items-end justify-around gap-4">
                {fans.map((f) => (
                  <FanGauge
                    key={f.target}
                    label={FAN_TARGET_LABELS[f.target]}
                    rpm={f.rpm}
                  />
                ))}
              </div>
            ) : (
              <p className="py-8 text-center text-sm text-muted-foreground">
                未检测到风扇。请确认 ASUS ATK 驱动已安装。
              </p>
            )}
          </CardContent>
        </Card>
      </motion.div>

      {/* ── Thermal profile ─────────────────────────────────── */}
      <motion.div variants={staggerItem} transition={spring.soft}>
        <Card>
          <CardHeader>
            <CardTitle>温控策略</CardTitle>
          </CardHeader>
          <CardContent>
            <ThermalProfileSelector
              active={profile}
              onChange={changeProfile}
            />
          </CardContent>
        </Card>
      </motion.div>

      {/* ── Fan curve editor ────────────────────────────────── */}
      <motion.div variants={staggerItem} transition={spring.soft}>
        <Card>
          <CardHeader>
            <CardTitle>自定义曲线</CardTitle>
          </CardHeader>
          <CardContent>
            <FanCurveEditor
              points={curvePoints}
              onChange={setCurvePoints}
            />
          </CardContent>
        </Card>
      </motion.div>
    </motion.div>
  );
}

