import { getVersion } from "@tauri-apps/api/app";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type DownloadEvent, type Update } from "@tauri-apps/plugin-updater";

export interface AvailableUpdate {
  version: string;
  currentVersion: string;
  notes?: string;
  date?: string;
}

export const UPDATE_AVAILABLE_EVENT = "applied-yet:update-available";

export async function currentAppVersion() {
  if (import.meta.env.DEV) return "0.1.0-dev";
  return isTauri() ? getVersion() : "0.1.0-preview";
}

export async function checkForUpdate(): Promise<AvailableUpdate | null> {
  if (import.meta.env.DEV || !isTauri()) return null;
  const update = await check(await updaterOptions());
  if (!update) return null;
  const result = updateDetails(update);
  await update.close();
  return result;
}

export async function downloadAndInstallUpdate(
  onEvent?: (event: DownloadEvent) => void,
) {
  if (import.meta.env.DEV) throw new Error("开发模式已禁用软件更新");
  if (!isTauri()) throw new Error("软件更新仅可在桌面应用中使用");
  const update = await check(await updaterOptions());
  if (!update) throw new Error("当前已是最新版本");
  await update.downloadAndInstall(onEvent, { timeout: 10 * 60_000 });
  await relaunch();
}

async function updaterOptions() {
  const proxy = await invoke<string | null>("get_system_proxy").catch(() => null);
  return proxy ? { timeout: 20_000, proxy } : { timeout: 20_000 };
}

function updateDetails(update: Update): AvailableUpdate {
  return {
    version: update.version,
    currentVersion: update.currentVersion,
    notes: update.body,
    date: update.date,
  };
}
