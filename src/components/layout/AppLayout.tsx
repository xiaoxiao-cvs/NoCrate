import { Titlebar } from "./Titlebar";
import { Sidebar } from "./Sidebar";

export function AppLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex h-screen flex-col overflow-hidden bg-bg-primary">
      {/* Titlebar — 32px fixed top */}
      <Titlebar />

      {/* Main area — Sidebar + Content */}
      <div className="flex flex-1 overflow-hidden">
        <Sidebar />

        {/* Content area */}
        <main className="flex-1 overflow-y-auto bg-bg-secondary p-4">
          {children}
        </main>
      </div>
    </div>
  );
}
