import { useLocation, useNavigate } from "react-router";
import { Fan, LayoutDashboard, Settings, Sun, Moon } from "lucide-react";
import { motion } from "motion/react";
import { useTheme } from "@/hooks/useTheme";

interface NavItem {
  icon: React.ElementType;
  label: string;
  path: string;
}

const navItems: NavItem[] = [
  { icon: LayoutDashboard, label: "仪表盘", path: "/" },
  { icon: Fan, label: "风扇控制", path: "/fan" },
  { icon: Settings, label: "设置", path: "/settings" },
];

export function Sidebar() {
  const location = useLocation();
  const navigate = useNavigate();
  const { resolvedTheme, toggleTheme } = useTheme();

  return (
    <aside className="flex h-full w-14 shrink-0 flex-col items-center border-r border-border bg-bg-primary py-3">
      {/* Navigation items */}
      <nav className="flex flex-1 flex-col items-center gap-1">
        {navItems.map((item) => {
          const isActive = location.pathname === item.path;
          const Icon = item.icon;

          return (
            <button
              key={item.path}
              aria-label={item.label}
              className="relative flex h-10 w-10 items-center justify-center rounded-lg text-text-secondary transition-colors hover:bg-bg-secondary hover:text-text-primary"
              onClick={() => navigate(item.path)}
              title={item.label}
            >
              {/* Active indicator — animated sliding bar */}
              {isActive && (
                <motion.div
                  className="absolute left-0 top-1/2 h-5 w-0.5 -translate-y-1/2 rounded-r-full bg-accent"
                  layoutId="sidebar-indicator"
                  transition={{
                    type: "spring",
                    stiffness: 350,
                    damping: 30,
                  }}
                />
              )}
              <Icon
                className={isActive ? "text-text-primary" : ""}
                size={18}
              />
            </button>
          );
        })}
      </nav>

      {/* Theme toggle at bottom */}
      <motion.button
        aria-label="切换主题"
        className="flex h-10 w-10 items-center justify-center rounded-lg text-text-secondary transition-colors hover:bg-bg-secondary hover:text-text-primary"
        onClick={toggleTheme}
        title={resolvedTheme === "dark" ? "切换到亮色" : "切换到暗色"}
        whileHover={{ scale: 1.05 }}
        whileTap={{ scale: 0.9 }}
      >
        <motion.div
          animate={{ rotate: resolvedTheme === "dark" ? 0 : 180 }}
          transition={{ type: "spring", stiffness: 200, damping: 15 }}
        >
          {resolvedTheme === "dark" ? <Moon size={18} /> : <Sun size={18} />}
        </motion.div>
      </motion.button>
    </aside>
  );
}
