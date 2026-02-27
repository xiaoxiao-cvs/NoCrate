/**
 * Simple HSV colour picker for AURA static-colour selection.
 *
 * Uses a saturation-lightness canvas + a hue slider, all rendered
 * with CSS gradients (no <canvas> needed). This is the ONLY place
 * in the app where chromatic colours appear.
 */
import { useCallback, useRef, useState, type PointerEvent } from "react";

import { cn } from "@/lib/utils";

export interface ColorPickerProps {
  /** Current colour as hex string (e.g. "#ff0000"). */
  value: string;
  onChange: (hex: string) => void;
  className?: string;
}

// ─── HSV ↔ RGB helpers ───────────────────────────────────────

interface Hsv {
  h: number; // 0–360
  s: number; // 0–1
  v: number; // 0–1
}

function hsvToHex({ h, s, v }: Hsv): string {
  const f = (n: number) => {
    const k = (n + h / 60) % 6;
    const c = v - v * s * Math.max(0, Math.min(k, 4 - k, 1));
    return Math.round(c * 255)
      .toString(16)
      .padStart(2, "0");
  };
  return `#${f(5)}${f(3)}${f(1)}`;
}

function hexToHsv(hex: string): Hsv {
  const h = hex.replace("#", "");
  const r = parseInt(h.slice(0, 2), 16) / 255;
  const g = parseInt(h.slice(2, 4), 16) / 255;
  const b = parseInt(h.slice(4, 6), 16) / 255;

  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const d = max - min;

  let hue = 0;
  if (d !== 0) {
    if (max === r) hue = ((g - b) / d + 6) % 6;
    else if (max === g) hue = (b - r) / d + 2;
    else hue = (r - g) / d + 4;
    hue *= 60;
  }

  return {
    h: hue,
    s: max === 0 ? 0 : d / max,
    v: max,
  };
}

// ─── Component ───────────────────────────────────────────────

export function ColorPicker({ value, onChange, className }: ColorPickerProps) {
  const [hsv, setHsv] = useState<Hsv>(() => hexToHsv(value));
  const svRef = useRef<HTMLDivElement>(null);
  const [dragging, setDragging] = useState<"sv" | "hue" | null>(null);

  const emit = useCallback(
    (next: Hsv) => {
      setHsv(next);
      onChange(hsvToHex(next));
    },
    [onChange],
  );

  // ── SV panel pointer handling ───────────────────────────────
  const handleSv = useCallback(
    (e: PointerEvent) => {
      const rect = svRef.current?.getBoundingClientRect();
      if (!rect) return;
      const s = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
      const v = Math.max(
        0,
        Math.min(1, 1 - (e.clientY - rect.top) / rect.height),
      );
      emit({ ...hsv, s, v });
    },
    [emit, hsv],
  );

  // ── Hue slider pointer handling ─────────────────────────────
  const handleHue = useCallback(
    (e: PointerEvent) => {
      const target = e.currentTarget as HTMLElement;
      const rect = target.getBoundingClientRect();
      const h = Math.max(
        0,
        Math.min(360, ((e.clientX - rect.left) / rect.width) * 360),
      );
      emit({ ...hsv, h });
    },
    [emit, hsv],
  );

  const onPointerDown = useCallback(
    (kind: "sv" | "hue", e: PointerEvent) => {
      e.preventDefault();
      (e.target as Element).setPointerCapture(e.pointerId);
      setDragging(kind);
      if (kind === "sv") handleSv(e);
      else handleHue(e);
    },
    [handleSv, handleHue],
  );

  const onPointerMove = useCallback(
    (kind: "sv" | "hue", e: PointerEvent) => {
      if (dragging !== kind) return;
      if (kind === "sv") handleSv(e);
      else handleHue(e);
    },
    [dragging, handleSv, handleHue],
  );

  const onPointerUp = useCallback(() => setDragging(null), []);

  const hueColor = `hsl(${hsv.h} 100% 50%)`;

  return (
    <div className={cn("flex flex-col gap-3", className)}>
      {/* ── SV panel ─────────────────────────────────────────── */}
      <div
        ref={svRef}
        className="relative h-40 cursor-crosshair rounded-lg border border-border"
        style={{
          background: `
            linear-gradient(to top, #000, transparent),
            linear-gradient(to right, #fff, ${hueColor})
          `,
        }}
        onPointerDown={(e) => onPointerDown("sv", e)}
        onPointerMove={(e) => onPointerMove("sv", e)}
        onPointerUp={onPointerUp}
        onPointerLeave={onPointerUp}
      >
        {/* Thumb */}
        <div
          className="pointer-events-none absolute h-4 w-4 -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-white shadow-md"
          style={{
            left: `${hsv.s * 100}%`,
            top: `${(1 - hsv.v) * 100}%`,
            backgroundColor: value,
          }}
        />
      </div>

      {/* ── Hue slider ───────────────────────────────────────── */}
      <div
        className="relative h-4 cursor-pointer rounded-full border border-border"
        style={{
          background:
            "linear-gradient(to right, #f00, #ff0, #0f0, #0ff, #00f, #f0f, #f00)",
        }}
        onPointerDown={(e) => onPointerDown("hue", e)}
        onPointerMove={(e) => onPointerMove("hue", e)}
        onPointerUp={onPointerUp}
        onPointerLeave={onPointerUp}
      >
        <div
          className="pointer-events-none absolute top-1/2 h-5 w-5 -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-white shadow-md"
          style={{
            left: `${(hsv.h / 360) * 100}%`,
            backgroundColor: hueColor,
          }}
        />
      </div>

      {/* ── Hex input ────────────────────────────────────────── */}
      <div className="flex items-center gap-2">
        <div
          className="h-8 w-8 rounded-md border border-border"
          style={{ backgroundColor: value }}
        />
        <input
          type="text"
          value={value}
          onChange={(e) => {
            const hex = e.target.value;
            if (/^#[0-9a-fA-F]{6}$/.test(hex)) {
              setHsv(hexToHsv(hex));
              onChange(hex);
            }
          }}
          className="h-8 w-24 rounded-md border border-border bg-background px-2 text-sm font-mono text-foreground focus:outline-none focus:ring-2 focus:ring-ring"
          maxLength={7}
        />
      </div>
    </div>
  );
}
