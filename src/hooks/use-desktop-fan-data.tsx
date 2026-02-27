/**
 * Hook for polling desktop fan policies from the backend.
 *
 * Desktop motherboards use `GetFanPolicy` / `SetFanPolicy` instead of
 * the laptop-style DSTS/DEVS RPM readings + three-mode thermal profiles.
 */
import { useCallback, useEffect, useRef, useState } from "react";

import {
  getDesktopFanPolicies,
  setDesktopFanPolicy as invokeSetPolicy,
} from "@/lib/tauri-commands";
import type { DesktopFanPolicy } from "@/lib/types";

/** Polling interval in milliseconds. */
const POLL_INTERVAL_MS = 3_000;

export interface UseDesktopFanDataReturn {
  /** Current fan policies. Empty array until first fetch. */
  policies: DesktopFanPolicy[];
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
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const mountedRef = useRef(true);

  const refresh = useCallback(async () => {
    try {
      const data = await getDesktopFanPolicies();
      if (!mountedRef.current) return;
      setPolicies(data);
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

  return { policies, loading, error, refresh, updatePolicy };
}
