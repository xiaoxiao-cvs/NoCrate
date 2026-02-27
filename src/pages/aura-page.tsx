import { motion } from "motion/react";
import { Loader2, Sparkles } from "lucide-react";
import { useCallback, useEffect, useState } from "react";

import { AuraEffectSelector } from "@/components/aura-effect-selector";
import { ColorPicker } from "@/components/color-picker";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  AURA_EFFECTS,
  auraIsAvailable,
  auraSetEffect,
  auraTurnOff,
  hexToRgb,
  rgbToHex,
  type AuraEffect,
  type AuraSpeed,
  type RgbColor,
} from "@/lib/aura-commands";
import { spring, staggerContainer, staggerItem } from "@/lib/motion";

export default function AuraPage() {
  const [available, setAvailable] = useState<boolean | null>(null);
  const [effect, setEffect] = useState<AuraEffect>("static");
  const [color, setColor] = useState<RgbColor>({ r: 255, g: 60, b: 0 });
  const [speed, setSpeed] = useState<AuraSpeed>("medium");
  const [error, setError] = useState<string | null>(null);

  // ── Check availability on mount ──────────────────────────────
  useEffect(() => {
    auraIsAvailable()
      .then(setAvailable)
      .catch(() => setAvailable(false));
  }, []);

  // ── Send effect to hardware ──────────────────────────────────
  const applyEffect = useCallback(
    async (e: AuraEffect, c: RgbColor, s: AuraSpeed) => {
      try {
        if (e === "off") {
          await auraTurnOff();
        } else {
          await auraSetEffect(e, c, s);
        }
        setError(null);
      } catch (err) {
        setError(String(err));
      }
    },
    [],
  );

  const handleEffectChange = useCallback(
    (e: AuraEffect) => {
      setEffect(e);
      applyEffect(e, color, speed);
    },
    [applyEffect, color, speed],
  );

  const handleSpeedChange = useCallback(
    (s: AuraSpeed) => {
      setSpeed(s);
      applyEffect(effect, color, s);
    },
    [applyEffect, effect, color],
  );

  const handleColorChange = useCallback(
    (hex: string) => {
      const rgb = hexToRgb(hex);
      setColor(rgb);
      // Only apply if current effect uses colour
      const meta = AURA_EFFECTS.find((e) => e.id === effect);
      if (meta?.hasColor) {
        applyEffect(effect, rgb, speed);
      }
    },
    [applyEffect, effect, speed],
  );

  // ── Loading ─────────────────────────────────────────────────
  if (available === null) {
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

  const currentMeta = AURA_EFFECTS.find((e) => e.id === effect);

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
          <Sparkles className="h-5 w-5" />
          灯效控制
        </h1>
        <p className="mt-1 text-sm text-muted-foreground">
          管理主板 ARGB 接头灯效模式与颜色
        </p>
      </motion.div>

      {/* ── Error ───────────────────────────────────────────── */}
      {error && (
        <motion.div
          variants={staggerItem}
          transition={spring.soft}
          className="rounded-lg border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive"
        >
          {error}
        </motion.div>
      )}

      {/* ── Not available ───────────────────────────────────── */}
      {!available && (
        <motion.div
          variants={staggerItem}
          transition={spring.soft}
          className="rounded-lg border border-border bg-card px-4 py-8 text-center text-sm text-muted-foreground"
        >
          未检测到 AURA 控制器。请确认主板型号支持且对应驱动已安装。
        </motion.div>
      )}

      {/* ── Controls ────────────────────────────────────────── */}
      {available && (
        <>
          {/* Effect selector */}
          <motion.div variants={staggerItem} transition={spring.soft}>
            <Card>
              <CardHeader>
                <CardTitle>灯效模式</CardTitle>
              </CardHeader>
              <CardContent>
                <AuraEffectSelector
                  effect={effect}
                  speed={speed}
                  onEffectChange={handleEffectChange}
                  onSpeedChange={handleSpeedChange}
                />
              </CardContent>
            </Card>
          </motion.div>

          {/* Colour picker — only for effects that use colour */}
          {currentMeta?.hasColor && (
            <motion.div variants={staggerItem} transition={spring.soft}>
              <Card>
                <CardHeader>
                  <CardTitle>颜色</CardTitle>
                </CardHeader>
                <CardContent>
                  <ColorPicker
                    value={rgbToHex(color)}
                    onChange={handleColorChange}
                  />
                </CardContent>
              </Card>
            </motion.div>
          )}
        </>
      )}
    </motion.div>
  );
}

