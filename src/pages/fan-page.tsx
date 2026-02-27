import { motion } from "motion/react";
import { Fan } from "lucide-react";
import { staggerContainer, staggerItem, spring } from "@/lib/motion";

export default function FanPage() {
  return (
    <motion.div
      variants={staggerContainer}
      initial="initial"
      animate="animate"
      className="flex flex-col gap-6"
    >
      <motion.div variants={staggerItem} transition={spring.soft}>
        <h1 className="flex items-center gap-2 text-xl font-semibold text-foreground">
          <Fan className="h-5 w-5" />
          风扇控制
        </h1>
        <p className="mt-1 text-sm text-muted-foreground">
          监控风扇转速与温度，调整风扇策略和自定义曲线
        </p>
      </motion.div>

      <motion.div
        variants={staggerItem}
        transition={spring.soft}
        className="flex h-48 items-center justify-center rounded-xl border border-border bg-card text-muted-foreground"
      >
        风扇数据面板（即将实现）
      </motion.div>
    </motion.div>
  );
}
