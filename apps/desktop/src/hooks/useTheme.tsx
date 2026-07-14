import { createContext, useContext, useEffect, useState, type ReactNode } from "react";

export type ThemeMode = "light" | "dark" | "system";
type ResolvedTheme = "light" | "dark";

interface ThemeContextValue {
  mode: ThemeMode;
  resolved: ResolvedTheme;
  setMode: (mode: ThemeMode) => void;
}

const ThemeContext = createContext<ThemeContextValue>({
  mode: "system",
  resolved: "light",
  setMode: () => {},
});

const STORAGE_KEY = "theme-mode";

const MEDIA = window.matchMedia("(prefers-color-scheme: dark)");

function resolveMode(mode: ThemeMode): ResolvedTheme {
  if (mode === "system") return MEDIA.matches ? "dark" : "light";
  return mode;
}

function applyTheme(theme: ResolvedTheme) {
  document.documentElement.setAttribute("data-theme", theme);
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [mode, setModeState] = useState<ThemeMode>(() => {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === "light" || stored === "dark" || stored === "system") return stored;
    return "system";
  });

  const [resolved, setResolved] = useState<ResolvedTheme>(() => resolveMode(mode));

  useEffect(() => {
    const handler = () => {
      if (mode === "system") setResolved(resolveMode("system"));
    };
    MEDIA.addEventListener("change", handler);
    return () => MEDIA.removeEventListener("change", handler);
  }, [mode]);

  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, mode);
    const next = resolveMode(mode);
    setResolved(next);
  }, [mode]);

  useEffect(() => {
    applyTheme(resolved);
  }, [resolved]);

  const setMode = (m: ThemeMode) => setModeState(m);

  return (
    <ThemeContext.Provider value={{ mode, resolved, setMode }}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  return useContext(ThemeContext);
}
