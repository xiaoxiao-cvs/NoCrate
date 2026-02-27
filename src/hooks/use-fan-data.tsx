/**
 * Hook for polling fan speeds and thermal profile from the backend.
 *
 * Refreshes at a fixed interval and exposes loading / error state
 * so the UI can show skeletons or error messages.
 */
import { useCallback, useEffect, useRef, useState } from "react";

import {
  getAllFanSpeeds,
  getThermalProfile,
  setThermalProfile as invokeSetThermalProfile,
} from "@/lib/tauri-commands";
import type { FanInfo, ThermalProfile } from "@/lib/types";

/** Polling interval in milliseconds. */
const POLL_INTERVAL_MS = 2_000;

export interface UseFanDataReturn {
  /** Current fan readings. Empty array until first fetch. */
  fans: FanInfo[];
  /** Active thermal profile. */
  profile: ThermalProfile;
  /** True until the first successful fetch. */
  loading: boolean;
  /** Human-readable error string, or `null`. */
  error: string | null;
  /** Force an immediate refresh. */
  refresh: () => Promise<void>;
  /** Switch the thermal profile and optimistically update local state. */
  changeProfile: (profile: ThermalProfile) => Promise<void>;
}

export function useFanData(): UseFanDataReturn {
  const [fans, setFans] = useState<FanInfo[]>([]);
  const [profile, setProfile] = useState<ThermalProfile>("standard");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const mountedRef = useRef(true);

  const refresh = useCallback(async () => {
    try {
      const [fanData, profileData] = await Promise.all([
        getAllFanSpeeds(),
        getThermalProfile(),
      ]);
      if (!mountedRef.current) return;
      setFans(fanData);
      setProfile(profileData);
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

  const changeProfile = useCallback(
    async (next: ThermalProfile) => {
      // Optimistic update
      setProfile(next);
      try {
        await invokeSetThermalProfile(next);
      } catch (e) {
        setError(String(e));
        // Roll back — next poll will correct the state anyway
        refresh();
      }
    },
    [refresh],
  );

  return { fans, profile, loading, error, refresh, changeProfile };
}
