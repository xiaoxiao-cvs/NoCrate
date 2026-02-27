import { useCallback, useEffect, useState } from "react";

import { getAsusHWSensors } from "@/lib/tauri-commands";
import type { AsusHWSensor } from "@/lib/types";

/** Polling interval for ASUSHW sensor data (ms). */
const POLL_INTERVAL = 2000;

/** Hook that periodically polls ASUSHW sensor readings. */
export function useAsusHWData() {
  const [sensors, setSensors] = useState<AsusHWSensor[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchSensors = useCallback(async () => {
    try {
      const data = await getAsusHWSensors();
      setSensors(data);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchSensors();
    const timer = setInterval(fetchSensors, POLL_INTERVAL);
    return () => clearInterval(timer);
  }, [fetchSensors]);

  const temps = sensors.filter((s) => s.sensor_type === "temperature");
  const fans = sensors.filter((s) => s.sensor_type === "fan");

  return { temps, fans, sensors, loading, error };
}
