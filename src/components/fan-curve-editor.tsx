/**
 * Interactive fan-curve editor.
 *
 * Renders an SVG graph with 8 draggable control points.
 * Temperature on the X axis (°C), duty cycle on Y (%).
 */
import { motion } from "motion/react";
import { useCallback, useRef, useState, type PointerEvent } from "react";

import { spring } from "@/lib/motion";
import { cn } from "@/lib/utils";
import type { FanCurvePoint } from "@/lib/types";

export interface FanCurveEditorProps {
  points: FanCurvePoint[];
  onChange?: (points: FanCurvePoint[]) => void;
  className?: string;
}

// ── Layout constants ──────────────────────────────────────────
const SVG_W = 480;
const SVG_H = 240;
const PAD = { top: 16, right: 16, bottom: 28, left: 36 };
const PLOT_W = SVG_W - PAD.left - PAD.right;
const PLOT_H = SVG_H - PAD.top - PAD.bottom;

// Axis ranges
const TEMP_MIN = 20;
const TEMP_MAX = 100;
const DUTY_MIN = 0;
const DUTY_MAX = 100;

// Grid lines
const TEMP_TICKS = [20, 30, 40, 50, 60, 70, 80, 90, 100];
const DUTY_TICKS = [0, 25, 50, 75, 100];

// ── Helpers ───────────────────────────────────────────────────
function tempToX(temp: number): number {
  return PAD.left + ((temp - TEMP_MIN) / (TEMP_MAX - TEMP_MIN)) * PLOT_W;
}

function dutyToY(duty: number): number {
  return PAD.top + (1 - (duty - DUTY_MIN) / (DUTY_MAX - DUTY_MIN)) * PLOT_H;
}

function xToTemp(x: number): number {
  const raw = TEMP_MIN + ((x - PAD.left) / PLOT_W) * (TEMP_MAX - TEMP_MIN);
  return Math.round(Math.max(TEMP_MIN, Math.min(TEMP_MAX, raw)));
}

function yToDuty(y: number): number {
  const raw =
    DUTY_MAX - ((y - PAD.top) / PLOT_H) * (DUTY_MAX - DUTY_MIN);
  return Math.round(Math.max(DUTY_MIN, Math.min(DUTY_MAX, raw)));
}

function pointsToPath(pts: FanCurvePoint[]): string {
  return pts
    .map((p, i) => {
      const x = tempToX(p.temp_c);
      const y = dutyToY(p.duty_pct);
      return `${i === 0 ? "M" : "L"} ${x} ${y}`;
    })
    .join(" ");
}

// ── Component ─────────────────────────────────────────────────
export function FanCurveEditor({
  points,
  onChange,
  className,
}: FanCurveEditorProps) {
  const svgRef = useRef<SVGSVGElement>(null);
  const [dragIdx, setDragIdx] = useState<number | null>(null);

  /** Convert pointer coords to SVG space. */
  const pointerToSvg = useCallback(
    (e: PointerEvent) => {
      const svg = svgRef.current;
      if (!svg) return { x: 0, y: 0 };
      const rect = svg.getBoundingClientRect();
      const scaleX = SVG_W / rect.width;
      const scaleY = SVG_H / rect.height;
      return {
        x: (e.clientX - rect.left) * scaleX,
        y: (e.clientY - rect.top) * scaleY,
      };
    },
    [],
  );

  const onPointerDown = useCallback(
    (idx: number, e: PointerEvent) => {
      e.preventDefault();
      (e.target as Element).setPointerCapture(e.pointerId);
      setDragIdx(idx);
    },
    [],
  );

  const onPointerMove = useCallback(
    (e: PointerEvent) => {
      if (dragIdx === null || !onChange) return;
      const { x, y } = pointerToSvg(e);
      const temp = xToTemp(x);
      const duty = yToDuty(y);

      const next = [...points];
      next[dragIdx] = { temp_c: temp, duty_pct: duty };

      // Keep points sorted by temperature
      next.sort((a, b) => a.temp_c - b.temp_c);
      onChange(next);
    },
    [dragIdx, onChange, pointerToSvg, points],
  );

  const onPointerUp = useCallback(() => {
    setDragIdx(null);
  }, []);

  return (
    <svg
      ref={svgRef}
      viewBox={`0 0 ${SVG_W} ${SVG_H}`}
      className={cn("w-full select-none", className)}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerLeave={onPointerUp}
    >
      {/* ── Grid lines ─────────────────────────────────────── */}
      {TEMP_TICKS.map((t) => (
        <line
          key={`vg-${t}`}
          x1={tempToX(t)}
          y1={PAD.top}
          x2={tempToX(t)}
          y2={PAD.top + PLOT_H}
          className="stroke-border"
          strokeWidth={0.5}
        />
      ))}
      {DUTY_TICKS.map((d) => (
        <line
          key={`hg-${d}`}
          x1={PAD.left}
          y1={dutyToY(d)}
          x2={PAD.left + PLOT_W}
          y2={dutyToY(d)}
          className="stroke-border"
          strokeWidth={0.5}
        />
      ))}

      {/* ── Axis labels ────────────────────────────────────── */}
      {TEMP_TICKS.map((t) => (
        <text
          key={`tl-${t}`}
          x={tempToX(t)}
          y={SVG_H - 4}
          textAnchor="middle"
          className="fill-muted-foreground text-[9px]"
        >
          {t}°
        </text>
      ))}
      {DUTY_TICKS.map((d) => (
        <text
          key={`dl-${d}`}
          x={PAD.left - 6}
          y={dutyToY(d) + 3}
          textAnchor="end"
          className="fill-muted-foreground text-[9px]"
        >
          {d}%
        </text>
      ))}

      {/* ── Filled area under curve ────────────────────────── */}
      <path
        d={`${pointsToPath(points)} L ${tempToX(points[points.length - 1].temp_c)} ${dutyToY(0)} L ${tempToX(points[0].temp_c)} ${dutyToY(0)} Z`}
        className="fill-foreground/5"
      />

      {/* ── Curve line ─────────────────────────────────────── */}
      <motion.path
        d={pointsToPath(points)}
        fill="none"
        className="stroke-foreground"
        strokeWidth={2}
        strokeLinecap="round"
        strokeLinejoin="round"
        initial={false}
        transition={spring.gauge}
      />

      {/* ── Control points ─────────────────────────────────── */}
      {points.map((p, i) => (
        <motion.circle
          key={i}
          cx={tempToX(p.temp_c)}
          cy={dutyToY(p.duty_pct)}
          r={dragIdx === i ? 7 : 5}
          className={cn(
            "cursor-grab fill-background stroke-foreground",
            dragIdx === i && "cursor-grabbing",
          )}
          strokeWidth={2}
          onPointerDown={(e) => onPointerDown(i, e)}
          whileHover={{ scale: 1.3 }}
          transition={spring.default}
        />
      ))}

      {/* ── Tooltip for dragged point ──────────────────────── */}
      {dragIdx !== null && points[dragIdx] && (
        <g>
          <rect
            x={tempToX(points[dragIdx].temp_c) - 28}
            y={dutyToY(points[dragIdx].duty_pct) - 26}
            width={56}
            height={18}
            rx={4}
            className="fill-foreground"
          />
          <text
            x={tempToX(points[dragIdx].temp_c)}
            y={dutyToY(points[dragIdx].duty_pct) - 14}
            textAnchor="middle"
            className="fill-background text-[10px] font-medium"
          >
            {points[dragIdx].temp_c}° → {points[dragIdx].duty_pct}%
          </text>
        </g>
      )}
    </svg>
  );
}
