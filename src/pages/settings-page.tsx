import { motion } from "motion/react";
import { useState, useEffect, useCallback } from "react";
import {
  Settings,
  Monitor,
  Moon,
  Sun,
  Info,
  RefreshCw,
  ShieldCheck,
  ShieldAlert,
  Power,
  Activity,
  CheckCircle,
  XCircle,
  ExternalLink,
  Download,
  Upload,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";

import { staggerContainer, staggerItem, spring } from "@/lib/motion";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { Select, type SelectOption } from "@/components/ui/select";
import { Button } from "@/components/ui/button";
import { useConfig } from "@/hooks/use-config";
import { useTheme } from "@/hooks/use-theme";
import { useToast } from "@/hooks/use-toast";
import { useAdminStatus } from "@/hooks/use-admin-status";
import { getLhmStatus } from "@/lib/tauri-commands";
import type { LhmStatus } from "@/lib/types";

// ─── Theme Options ───────────────────────────────────────────
const themeOptions: SelectOption[] = [
  { value: "system", label: "跟随系统", description: "自动" },
  { value: "light", label: "浅色" },
  { value: "dark", label: "深色" },
];

// ─── Poll Interval Options ───────────────────────────────────
const pollOptions: SelectOption[] = [
  { value: "1000", label: "1 秒" },
  { value: "2000", label: "2 秒" },
  { value: "5000", label: "5 秒" },
  { value: "10000", label: "10 秒" },
];

// ─── Setting Row ─────────────────────────────────────────────
function SettingRow({
  icon: Icon,
  label,
  description,
  children,
}: {
  icon: typeof Settings;
  label: string;
  description?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between gap-4 py-3">
      <div className="flex items-center gap-3">
        <Icon className="h-4 w-4 text-muted-foreground" />
        <div>
          <p className="text-sm font-medium">{label}</p>
          {description && (
            <p className="text-xs text-muted-foreground">{description}</p>
          )}
        </div>
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  );
}

export default function SettingsPage() {
  const { config, loading, update } = useConfig();
  const { setTheme } = useTheme();
  const toast = useToast();
  const isAdmin = useAdminStatus();

  // ─── Auto-start state ──────────────────────────────────
  const [autoStartEnabled, setAutoStartEnabled] = useState<boolean | null>(null);

  useEffect(() => {
    invoke<boolean>("get_auto_start_enabled")
      .then(setAutoStartEnabled)
      .catch(() => setAutoStartEnabled(false));
  }, []);

  const handleAutoStart = useCallback(
    async (enabled: boolean) => {
      try {
        await invoke("set_auto_start", { enabled });
        setAutoStartEnabled(enabled);
        toast.success(enabled ? "已设置开机自启" : "已取消开机自启");
      } catch (e) {
        toast.error(`设置失败: ${e}`);
      }
    },
    [toast],
  );

  // ─── LHM status ────────────────────────────────────────
  const [lhmStatus, setLhmStatus] = useState<LhmStatus | null>(null);

  useEffect(() => {
    getLhmStatus()
      .then(setLhmStatus)
      .catch(() => setLhmStatus("Unavailable"));
  }, []);

  // ─── Handlers ──────────────────────────────────────────

  const handleThemeChange = async (value: string) => {
    try {
      setTheme(value as "light" | "dark" | "system");
      await update({ theme: value });
    } catch {
      toast.error("主题切换失败");
    }
  };

  const handleToggle = async (
    key: "close_to_tray",
    value: boolean,
  ) => {
    try {
      await update({ [key]: value });
      toast.success("设置已保存");
    } catch {
      toast.error("保存失败");
    }
  };

  const handlePollChange = async (value: string) => {
    try {
      await update({ fan_poll_interval_ms: Number(value) });
      toast.success("轮询间隔已更新");
    } catch {
      toast.error("保存失败");
    }
  };

  // ─── Fan curve export/import ───────────────────────────
  const handleExportCurves = useCallback(async () => {
    try {
      const { getDesktopFanPolicies, probeDesktopFanTypes, getDesktopFanCurve } = await import("@/lib/tauri-commands");
      const policies = await getDesktopFanPolicies();
      const fanTypes = await probeDesktopFanTypes();
      const curves: Record<string, unknown> = {};

      for (const [fanType, modes] of fanTypes) {
        for (const mode of modes) {
          const curve = await getDesktopFanCurve(fanType, mode);
          if (curve) {
            curves[`fan${fanType}_${mode}`] = curve;
          }
        }
      }

      const exportData = {
        version: 1,
        timestamp: new Date().toISOString(),
        policies,
        curves,
      };

      const blob = new Blob([JSON.stringify(exportData, null, 2)], {
        type: "application/json",
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `nocrate-fan-profile-${Date.now()}.json`;
      a.click();
      URL.revokeObjectURL(url);
      toast.success("风扇配置已导出");
    } catch (e) {
      toast.error(`导出失败: ${e}`);
    }
  }, [toast]);

  const handleImportCurves = useCallback(() => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".json";
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return;
      try {
        const text = await file.text();
        const data = JSON.parse(text);
        if (data.version !== 1 || !data.curves) {
          toast.error("无效的配置文件格式");
          return;
        }

        const { setDesktopFanCurve } = await import("@/lib/tauri-commands");
        let count = 0;
        for (const curve of Object.values(data.curves)) {
          await setDesktopFanCurve(curve as Parameters<typeof setDesktopFanCurve>[0]);
          count++;
        }
        toast.success(`已导入 ${count} 条风扇曲线`);
      } catch (e) {
        toast.error(`导入失败: ${e}`);
      }
    };
    input.click();
  }, [toast]);

  if (loading || !config) {
    return (
      <div className="flex h-48 items-center justify-center text-muted-foreground">
        <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
        加载配置...
      </div>
    );
  }

  return (
    <motion.div
      variants={staggerContainer}
      initial="initial"
      animate="animate"
      className="flex flex-col gap-6"
    >
      {/* Header */}
      <motion.div variants={staggerItem} transition={spring.soft}>
        <h1 className="flex items-center gap-2 text-xl font-semibold text-foreground">
          <Settings className="h-5 w-5" />
          设置
        </h1>
        <p className="mt-1 text-sm text-muted-foreground">
          应用配置与系统偏好
        </p>
      </motion.div>

      {/* Appearance */}
      <motion.div variants={staggerItem} transition={spring.soft}>
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Sun className="h-4 w-4" />
              外观
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="divide-y divide-border">
              <SettingRow
                icon={Monitor}
                label="主题"
                description="选择界面配色方案"
              >
                <Select
                  value={config.theme}
                  onValueChange={handleThemeChange}
                  options={themeOptions}
                  className="w-36"
                />
              </SettingRow>
            </div>
          </CardContent>
        </Card>
      </motion.div>

      {/* Behavior */}
      <motion.div variants={staggerItem} transition={spring.soft}>
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Settings className="h-4 w-4" />
              行为
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="divide-y divide-border">
              <SettingRow
                icon={Moon}
                label="关闭时最小化到托盘"
                description="点击关闭按钮时隐藏到系统托盘"
              >
                <Switch
                  checked={config.close_to_tray}
                  onCheckedChange={(v) => handleToggle("close_to_tray", v)}
                />
              </SettingRow>
              <SettingRow
                icon={Power}
                label="开机自启"
                description="登录 Windows 时自动启动 NoCrate"
              >
                <Switch
                  checked={autoStartEnabled ?? false}
                  disabled={autoStartEnabled === null}
                  onCheckedChange={handleAutoStart}
                />
              </SettingRow>
              <SettingRow
                icon={RefreshCw}
                label="风扇轮询间隔"
                description="数据刷新频率"
              >
                <Select
                  value={String(config.fan_poll_interval_ms)}
                  onValueChange={handlePollChange}
                  options={pollOptions}
                  className="w-28"
                />
              </SettingRow>
            </div>
          </CardContent>
        </Card>
      </motion.div>

      {/* Sensor Service */}
      <motion.div variants={staggerItem} transition={spring.soft}>
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Activity className="h-4 w-4" />
              传感器服务
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2 text-sm">
                  <span className="text-muted-foreground">LibreHardwareMonitor</span>
                </div>
                {lhmStatus === "Available" ? (
                  <span className="inline-flex items-center gap-1 text-xs font-medium text-foreground">
                    <CheckCircle className="h-3.5 w-3.5 text-green-500" />
                    已连接
                  </span>
                ) : lhmStatus === "NoSensors" ? (
                  <span className="inline-flex items-center gap-1 text-xs font-medium text-yellow-600">
                    <CheckCircle className="h-3.5 w-3.5" />
                    已连接（无传感器数据）
                  </span>
                ) : (
                  <span className="inline-flex items-center gap-1 text-xs font-medium text-destructive">
                    <XCircle className="h-3.5 w-3.5" />
                    未检测到
                  </span>
                )}
              </div>
              {lhmStatus !== "Available" && (
                <div className="rounded-md border border-border bg-muted/50 p-3 text-xs text-muted-foreground">
                  <p>
                    实时传感器监控需要{" "}
                    <button
                      type="button"
                      className="inline-flex items-center gap-0.5 font-medium text-primary hover:underline"
                      onClick={() =>
                        openUrl(
                          "https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/releases",
                        )
                      }
                    >
                      LibreHardwareMonitor
                      <ExternalLink className="h-3 w-3" />
                    </button>{" "}
                    以管理员权限运行。
                  </p>
                  <p className="mt-1">
                    安装后以管理员身份启动 LHM，即可在「传感器」页面查看 CPU 温度、风扇转速等实时数据。
                  </p>
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      </motion.div>

      {/* Fan Profile Export/Import */}
      <motion.div variants={staggerItem} transition={spring.soft}>
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Download className="h-4 w-4" />
              风扇配置
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              <p className="text-xs text-muted-foreground">
                导出当前所有风扇曲线和策略为 JSON 文件，或从文件导入恢复。
              </p>
              <div className="flex gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleExportCurves}
                >
                  <Download className="mr-1.5 h-3.5 w-3.5" />
                  导出配置
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleImportCurves}
                >
                  <Upload className="mr-1.5 h-3.5 w-3.5" />
                  导入配置
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>
      </motion.div>

      {/* About */}
      <motion.div variants={staggerItem} transition={spring.soft}>
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Info className="h-4 w-4" />
              关于
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">应用名称</span>
                <span className="font-medium">NoCrate</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">版本</span>
                <span className="font-mono text-xs">0.1.0</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">描述</span>
                <span>轻量级 ASUS 主板硬件控制工具</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">权限状态</span>
                {isAdmin === null ? (
                  <span className="text-xs text-muted-foreground">检测中...</span>
                ) : isAdmin ? (
                  <span className="inline-flex items-center gap-1 text-xs font-medium text-foreground">
                    <ShieldCheck className="h-3.5 w-3.5" />
                    管理员
                  </span>
                ) : (
                  <span className="inline-flex items-center gap-1 text-xs font-medium text-destructive">
                    <ShieldAlert className="h-3.5 w-3.5" />
                    普通用户
                  </span>
                )}
              </div>
              <p className="pt-2 text-xs text-muted-foreground">
                替代 Armoury Crate，仅保留风扇调节与 ARGB 灯效控制。
              </p>
            </div>
          </CardContent>
        </Card>
      </motion.div>
    </motion.div>
  );
}
