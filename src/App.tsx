import { BrowserRouter, Routes, Route, Navigate } from "react-router";
import { ThemeProvider } from "./hooks/use-theme";

export function App() {
  return (
    <ThemeProvider>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<Navigate to="/fan" replace />} />
        </Routes>
      </BrowserRouter>
    </ThemeProvider>
  );
}
