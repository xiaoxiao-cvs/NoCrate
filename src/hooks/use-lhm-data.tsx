import { useCallback, useEffect, useRef, useState } from "react";

import { getLhmSensors, getLhmStatus } from "@/lib/tauri-commands";
import type { LhmSensorSnapshot, LhmStatus } from "@/lib/types";

/** Polling interval for LHM sensor data (ms). */
const POLL_INTERVAL = 2000;

/** Empty snapshot used as initial state. */
const EMPTY_SNAPSHOT: LhmSensorSnapshot = {
  temperatures: [],
  fans: [],
  controls: [],
  voltages: [],
  clocks: [],
  loads: [],
  powers: [],
};

/** Hook that periodically polls LibreHardwareMonitor sensor data. */
export function useLhmData() {
  const [status, setStatus] = useState<LhmStatus>("unavailable");
  const [snapshot, setSnapshot] = useState<LhmSensorSnapshot>(EMPTY_SNAPSHOT);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const pollingRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // Check LHM availability + fetch first snapshot
  const init = useCallback(async () => {
    try {
      const s = await getLhmStatus();
      setStatus(s);

      // If available, fetch sensors immediately
      const isAvailable = typeof s === "object" && "available" in s;
      if (isAvailable) {
        const data = await getLhmSensors();
        setSnapshot(data);
      }

      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  // Fetch sensors (for polling)
  const fetchSensors = useCallback(async () => {
    try {
      const data = await getLhmSensors();
      setSnapshot(data);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    init();
  }, [init]);

  // Start/stop polling based on status
  useEffect(() => {
    const isAvailable = typeof status === "object" && "available" in status;
    if (!isAvailable) {
      if (pollingRef.current) {
        clearInterval(pollingRef.current);
        pollingRef.current = null;
      }
      return;
    }

    pollingRef.current = setInterval(fetchSensors, POLL_INTERVAL);
    return () => {
      if (pollingRef.current) {
        clearInterval(pollingRef.current);
        pollingRef.current = null;
      }
    };
  }, [status, fetchSensors]);

  /** Refresh: re-check status and reload sensors. */
  const refresh = useCallback(async () => {
    setLoading(true);
    await init();
  }, [init]);

  const isAvailable = typeof status === "object" && "available" in status;

  return {
    status,
    snapshot,
    isAvailable,
    loading,
    error,
    refresh,
  };
}
