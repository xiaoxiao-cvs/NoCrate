/**
 * Semi-circular gauge that visualizes fan RPM.
 *
 * Uses Motion springs for smooth arc fill and number transitions.
 */
import { motion, useMotionValue, useSpring, useTransform } from "motion/react";
import { useEffect, useState } from "react";

import { cn } from "@/lib/utils";

export interface FanGaugeProps {
  /** Display label (e.g. "CPU", "GPU"). */
  label: string;
  /** Current RPM reading. */
  rpm: number;
  /** Expected maximum RPM – sets the 100 % mark. */
  maxRpm?: number;
  className?: string;
}

// SVG layout
const VIEW_W = 180;
const VIEW_H = 108;
const CX = VIEW_W / 2;
const CY = 96;
const RADIUS = 72;
const STROKE = 7;

// Semi-circle arc (180°) — left to right
const ARC_PATH = `M ${CX - RADIUS} ${CY} A ${RADIUS} ${RADIUS} 0 0 1 ${CX + RADIUS} ${CY}`;
const ARC_LENGTH = Math.PI * RADIUS; // ≈ 226.2

/** Spring config for the arc & number. */
const SPRING = { stiffness: 90, damping: 22 };

export function FanGauge({
  label,
  rpm,
  maxRpm = 2500,
  className,
}: FanGaugeProps) {
  // ── Animated RPM value ──────────────────────────────────────
  const motionRpm = useMotionValue(rpm);
  const springRpm = useSpring(motionRpm, SPRING);
  const [displayRpm, setDisplayRpm] = useState(rpm);

  useEffect(() => {
    motionRpm.set(rpm);
  }, [rpm, motionRpm]);

  useEffect(() => {
    const unsub = springRpm.on("change", (v) =>
      setDisplayRpm(Math.round(v)),
    );
    return unsub;
  }, [springRpm]);

  // ── Arc fill ────────────────────────────────────────────────
  const normalised = useTransform(springRpm, [0, maxRpm], [0, 1]);
  const dashOffset = useTransform(
    normalised,
    (n) => ARC_LENGTH * (1 - Math.min(n, 1)),
  );

  return (
    <div className={cn("flex flex-col items-center gap-1", className)}>
      <svg
        viewBox={`0 0 ${VIEW_W} ${VIEW_H}`}
        className="w-full max-w-45"
      >
        {/* Background track */}
        <path
          d={ARC_PATH}
          fill="none"
          className="stroke-border"
          strokeWidth={STROKE}
          strokeLinecap="round"
        />

        {/* Animated fill */}
        <motion.path
          d={ARC_PATH}
          fill="none"
          className="stroke-foreground"
          strokeWidth={STROKE}
          strokeLinecap="round"
          style={{
            strokeDasharray: ARC_LENGTH,
            strokeDashoffset: dashOffset,
          }}
        />

        {/* RPM number */}
        <text
          x={CX}
          y={CY - 20}
          textAnchor="middle"
          className="fill-foreground text-[28px] font-bold tabular-nums"
        >
          {displayRpm}
        </text>

        {/* Unit */}
        <text
          x={CX}
          y={CY - 4}
          textAnchor="middle"
          className="fill-muted-foreground text-[11px]"
        >
          RPM
        </text>
      </svg>

      <span className="text-xs font-medium tracking-wider text-muted-foreground">
        {label}
      </span>
    </div>
  );
}
