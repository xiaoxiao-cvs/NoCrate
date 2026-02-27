import { BrowserRouter, Routes, Route, Navigate } from "react-router";
import { lazy, Suspense } from "react";
import { ThemeProvider } from "./hooks/use-theme";
import { AppLayout } from "./layouts/app-layout";

const FanPage = lazy(() => import("./pages/fan-page"));
const AuraPage = lazy(() => import("./pages/aura-page"));
const SettingsPage = lazy(() => import("./pages/settings-page"));

export function App() {
  return (
    <ThemeProvider>
      <BrowserRouter>
        <Routes>
          <Route element={<AppLayout />}>
            <Route
              path="/fan"
              element={
                <Suspense>
                  <FanPage />
                </Suspense>
              }
            />
            <Route
              path="/aura"
              element={
                <Suspense>
                  <AuraPage />
                </Suspense>
              }
            />
            <Route
              path="/settings"
              element={
                <Suspense>
                  <SettingsPage />
                </Suspense>
              }
            />
            <Route path="*" element={<Navigate to="/fan" replace />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </ThemeProvider>
  );
}
