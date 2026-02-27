import { motion } from "motion/react";
import { Settings, Monitor, Moon, Sun, Info, RefreshCw } from "lucide-react";

import { staggerContainer, staggerItem, spring } from "@/lib/motion";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { Select, type SelectOption } from "@/components/ui/select";
import { useConfig } from "@/hooks/use-config";
import { useTheme } from "@/hooks/use-theme";
import { useToast } from "@/hooks/use-toast";

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

  const handleThemeChange = async (value: string) => {
    try {
      setTheme(value as "light" | "dark" | "system");
      await update({ theme: value });
    } catch {
      toast.error("主题切换失败");
    }
  };

  const handleToggle = async (
    key: "close_to_tray" | "auto_start",
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
