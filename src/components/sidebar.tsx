import { useLocation, useNavigate } from "react-router";
import { Fan, Sparkles, Settings, PanelLeftClose, PanelLeft, Sun, Moon, Activity } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useCallback, useState } from "react";
import { cn } from "@/lib/utils";
import { spring, sidebarVariants, interactive } from "@/lib/motion";
import { useTheme } from "@/hooks/use-theme";

interface NavItem {
  path: string;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
}

const navItems: NavItem[] = [
  { path: "/fan", label: "风扇控制", icon: Fan },
  { path: "/sensor", label: "传感器", icon: Activity },
  { path: "/aura", label: "灯效控制", icon: Sparkles },
  { path: "/settings", label: "设置", icon: Settings },
];

export function Sidebar() {
  const location = useLocation();
  const navigate = useNavigate();
  const { resolvedTheme, setTheme } = useTheme();
  const [collapsed, setCollapsed] = useState(false);

  const toggleCollapse = useCallback(() => {
    setCollapsed((prev) => !prev);
  }, []);

  const toggleTheme = useCallback(() => {
    setTheme(resolvedTheme === "dark" ? "light" : "dark");
  }, [resolvedTheme, setTheme]);

  return (
    <motion.aside
      className="flex h-full shrink-0 flex-col border-r border-sidebar-border bg-sidebar"
      variants={sidebarVariants}
      animate={collapsed ? "collapsed" : "expanded"}
      transition={spring.soft}
    >
      {/* Navigation items */}
      <nav className="flex flex-1 flex-col gap-1 p-2">
        {navItems.map((item) => {
          const isActive = location.pathname === item.path;
          const Icon = item.icon;

          return (
            <motion.button
              key={item.path}
              type="button"
              onClick={() => void navigate(item.path)}
              whileHover={interactive.whileHover}
              whileTap={interactive.whileTap}
              transition={spring.default}
              className={cn(
                "relative flex items-center gap-3 rounded-lg px-3 py-2",
                "text-sm font-medium text-sidebar-foreground",
                "transition-colors hover:bg-sidebar-accent",
                isActive && "text-sidebar-accent-foreground",
              )}
            >
              {/* Active indicator — shared layout animation */}
              {isActive && (
                <motion.div
                  layoutId="sidebar-active"
                  className="absolute inset-0 rounded-lg bg-sidebar-accent"
                  transition={spring.snappy}
                />
              )}

              <Icon className="relative z-10 h-4 w-4 shrink-0" />

              <AnimatePresence mode="wait">
                {!collapsed && (
                  <motion.span
                    key="label"
                    initial={{ opacity: 0, width: 0 }}
                    animate={{ opacity: 1, width: "auto" }}
                    exit={{ opacity: 0, width: 0 }}
                    transition={spring.soft}
                    className="relative z-10 overflow-hidden whitespace-nowrap"
                  >
                    {item.label}
                  </motion.span>
                )}
              </AnimatePresence>
            </motion.button>
          );
        })}
      </nav>

      {/* Bottom controls */}
      <div className="flex flex-col gap-1 border-t border-sidebar-border p-2">
        {/* Theme toggle */}
        <motion.button
          type="button"
          onClick={toggleTheme}
          whileHover={interactive.whileHover}
          whileTap={interactive.whileTap}
          transition={spring.default}
          className="flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium text-sidebar-foreground transition-colors hover:bg-sidebar-accent"
          aria-label="切换主题"
        >
          <AnimatePresence mode="wait">
            {resolvedTheme === "dark" ? (
              <motion.div
                key="sun"
                initial={{ rotate: -90, opacity: 0 }}
                animate={{ rotate: 0, opacity: 1 }}
                exit={{ rotate: 90, opacity: 0 }}
                transition={spring.default}
              >
                <Sun className="h-4 w-4 shrink-0" />
              </motion.div>
            ) : (
              <motion.div
                key="moon"
                initial={{ rotate: 90, opacity: 0 }}
                animate={{ rotate: 0, opacity: 1 }}
                exit={{ rotate: -90, opacity: 0 }}
                transition={spring.default}
              >
                <Moon className="h-4 w-4 shrink-0" />
              </motion.div>
            )}
          </AnimatePresence>

          <AnimatePresence mode="wait">
            {!collapsed && (
              <motion.span
                key="theme-label"
                initial={{ opacity: 0, width: 0 }}
                animate={{ opacity: 1, width: "auto" }}
                exit={{ opacity: 0, width: 0 }}
                transition={spring.soft}
                className="overflow-hidden whitespace-nowrap"
              >
                {resolvedTheme === "dark" ? "浅色模式" : "深色模式"}
              </motion.span>
            )}
          </AnimatePresence>
        </motion.button>

        {/* Collapse toggle */}
        <motion.button
          type="button"
          onClick={toggleCollapse}
          whileHover={interactive.whileHover}
          whileTap={interactive.whileTap}
          transition={spring.default}
          className="flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium text-muted-foreground transition-colors hover:bg-sidebar-accent"
          aria-label={collapsed ? "展开侧边栏" : "收起侧边栏"}
        >
          <AnimatePresence mode="wait">
            {collapsed ? (
              <motion.div
                key="expand"
                initial={{ rotate: 180, opacity: 0 }}
                animate={{ rotate: 0, opacity: 1 }}
                exit={{ rotate: -180, opacity: 0 }}
                transition={spring.default}
              >
                <PanelLeft className="h-4 w-4 shrink-0" />
              </motion.div>
            ) : (
              <motion.div
                key="collapse"
                initial={{ rotate: -180, opacity: 0 }}
                animate={{ rotate: 0, opacity: 1 }}
                exit={{ rotate: 180, opacity: 0 }}
                transition={spring.default}
              >
                <PanelLeftClose className="h-4 w-4 shrink-0" />
              </motion.div>
            )}
          </AnimatePresence>

          <AnimatePresence mode="wait">
            {!collapsed && (
              <motion.span
                key="collapse-label"
                initial={{ opacity: 0, width: 0 }}
                animate={{ opacity: 1, width: "auto" }}
                exit={{ opacity: 0, width: 0 }}
                transition={spring.soft}
                className="overflow-hidden whitespace-nowrap"
              >
                收起
              </motion.span>
            )}
          </AnimatePresence>
        </motion.button>
      </div>
    </motion.aside>
  );
}
