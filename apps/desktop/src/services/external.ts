import { invoke, isTauri } from "@tauri-apps/api/core";

export function openExternalUrl(value: string) {
  let url: URL;
  try {
    url = new URL(value);
  } catch {
    return Promise.reject(new Error("外部链接格式无效"));
  }
  if (!["http:", "https:", "mailto:"].includes(url.protocol)) {
    return Promise.reject(new Error("只允许打开 HTTP、HTTPS 或邮件链接"));
  }
  if (isTauri()) return invoke<void>("open_external_url", { url: url.href });
  const opened = window.open(url.href, "_blank", "noopener,noreferrer");
  return opened ? Promise.resolve() : Promise.reject(new Error("浏览器阻止了新窗口，请允许弹出窗口后重试"));
}
