/**
 * Hook for polling desktop fan policies and live RPM from the backend.
 *
 * Desktop motherboards use `GetFanPolicy` / `SetFanPolicy` for control,
 * plus `device_status` (via get_all_fan_speeds) for live RPM readings.
 */
import { useCallback, useEffect, useRef, useState } from "react";

import {
  getAllFanSpeeds,
  getDesktopFanPolicies,
  setDesktopFanPolicy as invokeSetPolicy,
} from "@/lib/tauri-commands";
import type { DesktopFanPolicy, FanInfo } from "@/lib/types";

/** Polling interval in milliseconds. */
const POLL_INTERVAL_MS = 2_000;

export interface UseDesktopFanDataReturn {
  /** Current fan policies. Empty array until first fetch. */
  policies: DesktopFanPolicy[];
  /** Live RPM readings from device_status. */
  fanSpeeds: FanInfo[];
  /** True until the first successful fetch. */
  loading: boolean;
  /** Human-readable error string, or `null`. */
  error: string | null;
  /** Force an immediate refresh. */
  refresh: () => Promise<void>;
  /** Update a single fan header's policy. */
  updatePolicy: (policy: DesktopFanPolicy) => Promise<void>;
}

export function useDesktopFanData(): UseDesktopFanDataReturn {
  const [policies, setPolicies] = useState<DesktopFanPolicy[]>([]);
  const [fanSpeeds, setFanSpeeds] = useState<FanInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const mountedRef = useRef(true);

  const refresh = useCallback(async () => {
    try {
      const [policyData, speedData] = await Promise.all([
        getDesktopFanPolicies(),
        getAllFanSpeeds().catch(() => [] as FanInfo[]),
      ]);
      if (!mountedRef.current) return;
      setPolicies(policyData);
      setFanSpeeds(speedData);
      setError(null);
    } catch (e) {
      if (!mountedRef.current) return;
      setError(String(e));
    } finally {
      if (mountedRef.current) setLoading(false);
    }
  }, []);

  useEffect(() => {
    mountedRef.current = true;
    refresh();
    const id = setInterval(refresh, POLL_INTERVAL_MS);
    return () => {
      mountedRef.current = false;
      clearInterval(id);
    };
  }, [refresh]);

  const updatePolicy = useCallback(
    async (policy: DesktopFanPolicy) => {
      // Optimistic update
      setPolicies((prev) =>
        prev.map((p) => (p.fan_type === policy.fan_type ? policy : p)),
      );
      try {
        await invokeSetPolicy(policy);
      } catch (e) {
        setError(String(e));
        refresh();
      }
    },
    [refresh],
  );

  return { policies, fanSpeeds, loading, error, refresh, updatePolicy };
}
