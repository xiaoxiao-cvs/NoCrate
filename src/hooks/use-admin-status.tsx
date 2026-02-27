import { useEffect, useState } from "react";
import { isAdmin } from "@/lib/system-commands";

/**
 * Check if the app is running with administrator privileges.
 * Returns `null` while loading, then `true` or `false`.
 */
export function useAdminStatus(): boolean | null {
  const [admin, setAdmin] = useState<boolean | null>(null);

  useEffect(() => {
    isAdmin()
      .then(setAdmin)
      .catch(() => setAdmin(false));
  }, []);

  return admin;
}
