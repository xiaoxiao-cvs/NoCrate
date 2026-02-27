import { Routes, Route, useLocation } from "react-router";
import { AppLayout } from "@/components/layout/AppLayout";
import { PageTransition } from "@/components/layout/PageTransition";
import { Dashboard } from "@/pages/Dashboard";
import { FanControl } from "@/pages/FanControl";
import { SettingsPage } from "@/pages/Settings";

function App() {
  const location = useLocation();

  return (
    <AppLayout>
      <PageTransition key={location.pathname}>
        <Routes location={location}>
          <Route path="/" element={<Dashboard />} />
          <Route path="/fan" element={<FanControl />} />
          <Route path="/settings" element={<SettingsPage />} />
        </Routes>
      </PageTransition>
    </AppLayout>
  );
}

export default App;
