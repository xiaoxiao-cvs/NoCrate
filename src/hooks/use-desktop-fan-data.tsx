/**
 * Hook for polling desktop fan policies, curves and live sensor data.
 *
 * Desktop motherboards use `GetFanPolicy` / `SetFanPolicy` for control,
 * `GetManualFanCurvePro` / `SetManualFanCurvePro` for 8-point curves,
 * plus Super I/O 直读获取实时风扇转速与温度。
 */
import { useCallback, useEffect, useRef, useState } from "react";

import {
  getDesktopFanCurve,
  getDesktopFanPolicies,
  getSioSensors,
  setDesktopFanCurve as invokeSetCurve,
  setDesktopFanPolicy as invokeSetPolicy,
} from "@/lib/tauri-commands";
import type {
  DesktopFanCurve,
  DesktopFanMode,
  DesktopFanPolicy,
  FanCurvePoint,
  SioSnapshot,
} from "@/lib/types";

/** Polling interval in milliseconds. */
const POLL_INTERVAL_MS = 2_000;

export interface UseDesktopFanDataReturn {
  /** Current fan policies. Empty array until first fetch. */
  policies: DesktopFanPolicy[];
  /** 当前加载的风扇曲线，按 "fanType-mode" 索引。 */
  curves: Map<string, DesktopFanCurve>;
  /** Super I/O 传感器快照（含风扇转速与温度） */
  sioData: SioSnapshot | null;
  /** True until the first successful fetch. */
  loading: boolean;
  /** Human-readable error string, or `null`. */
  error: string | null;
  /** Force an immediate refresh. */
  refresh: () => Promise<void>;
  /** Update a single fan header's policy. */
  updatePolicy: (policy: DesktopFanPolicy) => Promise<void>;
  /** 加载指定风扇头+模式的曲线。 */
  loadCurve: (fanType: number, mode: DesktopFanMode) => Promise<void>;
  /** 保存风扇曲线到硬件。 */
  saveCurve: (
    fanType: number,
    mode: DesktopFanMode,
    points: FanCurvePoint[],
  ) => Promise<void>;
}

/** 生成曲线缓存键。 */
function curveKey(fanType: number, mode: DesktopFanMode): string {
  return `${fanType}-${mode}`;
}

export function useDesktopFanData(): UseDesktopFanDataReturn {
  const [policies, setPolicies] = useState<DesktopFanPolicy[]>([]);
  const [curves, setCurves] = useState<Map<string, DesktopFanCurve>>(new Map());
  const [sioData, setSioData] = useState<SioSnapshot | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const mountedRef = useRef(true);

  const refresh = useCallback(async () => {
    try {
      const [policyData, sensorData] = await Promise.all([
        getDesktopFanPolicies(),
        getSioSensors().catch(() => null),
      ]);
      if (!mountedRef.current) return;
      setPolicies(policyData);
      setSioData(sensorData);
      setError(null);

      // 自动加载每个风扇头当前模式的曲线
      const curvePromises = policyData.map(async (p) => {
        const key = curveKey(p.fan_type, p.mode);
        try {
          const curve = await getDesktopFanCurve(p.fan_type, p.mode);
          return curve ? ([key, curve] as const) : null;
        } catch {
          return null;
        }
      });
      const curveResults = await Promise.all(curvePromises);
      if (!mountedRef.current) return;

      setCurves((prev) => {
        const next = new Map(prev);
        for (const result of curveResults) {
          if (result) next.set(result[0], result[1]);
        }
        return next;
      });
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

  const loadCurve = useCallback(
    async (fanType: number, mode: DesktopFanMode) => {
      try {
        const curve = await getDesktopFanCurve(fanType, mode);
        if (!mountedRef.current) return;
        if (curve) {
          setCurves((prev) => {
            const next = new Map(prev);
            next.set(curveKey(fanType, mode), curve);
            return next;
          });
        }
      } catch (e) {
        setError(String(e));
      }
    },
    [],
  );

  const saveCurve = useCallback(
    async (fanType: number, mode: DesktopFanMode, points: FanCurvePoint[]) => {
      const curve: DesktopFanCurve = { fan_type: fanType, mode, points };
      // Optimistic update
      setCurves((prev) => {
        const next = new Map(prev);
        next.set(curveKey(fanType, mode), curve);
        return next;
      });
      try {
        await invokeSetCurve(curve);
      } catch (e) {
        setError(String(e));
        // 回滚：重新从硬件加载
        loadCurve(fanType, mode);
      }
    },
    [loadCurve],
  );

  return {
    policies,
    curves,
    sioData,
    loading,
    error,
    refresh,
    updatePolicy,
    loadCurve,
    saveCurve,
  };
}
