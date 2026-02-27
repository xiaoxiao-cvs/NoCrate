import { motion } from "motion/react";
import { Settings } from "lucide-react";
import { staggerContainer, staggerItem, spring } from "@/lib/motion";

export default function SettingsPage() {
  return (
    <motion.div
      variants={staggerContainer}
      initial="initial"
      animate="animate"
      className="flex flex-col gap-6"
    >
      <motion.div variants={staggerItem} transition={spring.soft}>
        <h1 className="flex items-center gap-2 text-xl font-semibold text-foreground">
          <Settings className="h-5 w-5" />
          设置
        </h1>
        <p className="mt-1 text-sm text-muted-foreground">
          应用配置与系统偏好
        </p>
      </motion.div>

      <motion.div
        variants={staggerItem}
        transition={spring.soft}
        className="flex h-48 items-center justify-center rounded-xl border border-border bg-card text-muted-foreground"
      >
        设置选项（即将实现）
      </motion.div>
    </motion.div>
  );
}
