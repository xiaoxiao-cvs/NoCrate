import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";

import {
  getConfig,
  updateConfig,
  type AppConfig,
  type ConfigUpdate,
} from "@/lib/config-commands";

interface ConfigContextValue {
  config: AppConfig | null;
  loading: boolean;
  error: string | null;
  update: (changes: ConfigUpdate) => Promise<void>;
  refresh: () => void;
}

const ConfigContext = createContext<ConfigContextValue | undefined>(undefined);

const DEFAULT_CONFIG: AppConfig = {
  theme: "system",
  close_to_tray: false,
  auto_start: false,
  fan_poll_interval_ms: 2000,
  last_thermal_profile: 0,
  last_aura_effect: "static",
  last_aura_color: "#ff0000",
  last_aura_speed: "medium",
};

export function ConfigProvider({ children }: { children: ReactNode }) {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const mountedRef = useRef(true);

  const load = useCallback(() => {
    setLoading(true);
    getConfig()
      .then((cfg) => {
        if (mountedRef.current) {
          setConfig(cfg);
          setError(null);
        }
      })
      .catch((e) => {
        if (mountedRef.current) {
          console.warn("Config load failed, using defaults:", e);
          setConfig(DEFAULT_CONFIG);
          setError(null);
        }
      })
      .finally(() => {
        if (mountedRef.current) setLoading(false);
      });
  }, []);

  useEffect(() => {
    mountedRef.current = true;
    load();
    return () => {
      mountedRef.current = false;
    };
  }, [load]);

  const update = useCallback(
    async (changes: ConfigUpdate) => {
      // Optimistic update
      setConfig((prev) => (prev ? { ...prev, ...changes } : prev));
      try {
        const updated = await updateConfig(changes);
        if (mountedRef.current) setConfig(updated);
      } catch (e) {
        // Rollback on failure
        if (mountedRef.current) load();
        throw e;
      }
    },
    [load],
  );

  return (
    <ConfigContext.Provider
      value={{ config, loading, error, update, refresh: load }}
    >
      {children}
    </ConfigContext.Provider>
  );
}

export function useConfig(): ConfigContextValue {
  const ctx = useContext(ConfigContext);
  if (!ctx) {
    throw new Error("useConfig must be used within a ConfigProvider");
  }
  return ctx;
}
