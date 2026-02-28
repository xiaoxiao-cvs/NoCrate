import { useEffect, useRef } from "react";

import type { LhmSensorSnapshot } from "@/lib/types";
import { useToast } from "@/hooks/use-toast";
import { useConfig } from "@/hooks/use-config";

/**
 * Monitors LHM sensor data and fires toast alerts when any
 * temperature sensor exceeds the configured threshold.
 *
 * Uses a cooldown to avoid spamming — only fires once per sensor
 * per 60 seconds.
 */
export function useTempAlerts(snapshot: LhmSensorSnapshot | null) {
  const { config } = useConfig();
  const toast = useToast();

  // Track last alert time per sensor to avoid spam
  const cooldownRef = useRef<Map<string, number>>(new Map());

  const COOLDOWN_MS = 60_000; // 1 minute cooldown per sensor

  useEffect(() => {
    if (!snapshot || !config?.temp_alert_enabled) return;

    const threshold = config.temp_alert_threshold;
    const now = Date.now();

    for (const sensor of snapshot.temperatures) {
      if (sensor.value >= threshold) {
        const lastAlert = cooldownRef.current.get(sensor.identifier) ?? 0;
        if (now - lastAlert > COOLDOWN_MS) {
          cooldownRef.current.set(sensor.identifier, now);
          toast.warning(
            `${sensor.name} 温度过高`,
            `当前 ${sensor.value.toFixed(0)}°C，已超过阈值 ${threshold}°C`,
          );
        }
      }
    }
  }, [snapshot, config?.temp_alert_enabled, config?.temp_alert_threshold, toast]);
}
