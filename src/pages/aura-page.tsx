import { motion } from "motion/react";
import { Sparkles } from "lucide-react";
import { staggerContainer, staggerItem, spring } from "@/lib/motion";

export default function AuraPage() {
  return (
    <motion.div
      variants={staggerContainer}
      initial="initial"
      animate="animate"
      className="flex flex-col gap-6"
    >
      <motion.div variants={staggerItem} transition={spring.soft}>
        <h1 className="flex items-center gap-2 text-xl font-semibold text-foreground">
          <Sparkles className="h-5 w-5" />
          灯效控制
        </h1>
        <p className="mt-1 text-sm text-muted-foreground">
          管理主板 ARGB 接头灯效模式、颜色和亮度
        </p>
      </motion.div>

      <motion.div
        variants={staggerItem}
        transition={spring.soft}
        className="flex h-48 items-center justify-center rounded-xl border border-border bg-card text-muted-foreground"
      >
        ARGB 控制面板（即将实现）
      </motion.div>
    </motion.div>
  );
}
