import { getCurrentWindow } from "@tauri-apps/api/window";

type WindowTheme = "light" | "dark";

export async function setWindowTheme(theme: WindowTheme): Promise<void> {
  if (!("__TAURI_INTERNALS__" in window)) return;
  await getCurrentWindow().setTheme(theme);
}
