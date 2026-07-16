import { getVersion } from "@tauri-apps/api/app";
import { isTauri } from "@tauri-apps/api/core";
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
  return isTauri() ? getVersion() : "0.1.0-preview";
}

export async function checkForUpdate(): Promise<AvailableUpdate | null> {
  if (!isTauri()) return null;
  const update = await check({ timeout: 20_000 });
  if (!update) return null;
  const result = updateDetails(update);
  await update.close();
  return result;
}

export async function downloadAndInstallUpdate(
  onEvent?: (event: DownloadEvent) => void,
) {
  if (!isTauri()) throw new Error("软件更新仅可在桌面应用中使用");
  const update = await check({ timeout: 20_000 });
  if (!update) throw new Error("当前已是最新版本");
  await update.downloadAndInstall(onEvent, { timeout: 10 * 60_000 });
  await relaunch();
}

function updateDetails(update: Update): AvailableUpdate {
  return {
    version: update.version,
    currentVersion: update.currentVersion,
    notes: update.body,
    date: update.date,
  };
}
