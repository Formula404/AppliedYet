import { invoke } from "@tauri-apps/api/core";
import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";
import { hasLocalDatabase } from "./applications";

interface DueTaskReminder {
  taskId: string;
  applicationId: string;
  title: string;
  company: string;
  role: string;
  dueAt?: string;
}

const dueText = (value?: string) => value
  ? new Date(value).toLocaleString("zh-CN", { month: "numeric", day: "numeric", hour: "2-digit", minute: "2-digit" })
  : "未设置截止时间";

let schedulerUsers = 0;
let schedulerTimer: number | undefined;
let activePoll: Promise<void> | undefined;
let permissionResult: boolean | undefined;
let permissionRequest: Promise<boolean> | undefined;

const ensurePermission = () => {
  if (permissionResult !== undefined) return Promise.resolve(permissionResult);
  if (!permissionRequest) {
    permissionRequest = (async () => {
      let allowed = await isPermissionGranted();
      if (!allowed) allowed = await requestPermission() === "granted";
      permissionResult = allowed;
      return allowed;
    })().finally(() => { permissionRequest = undefined; });
  }
  return permissionRequest;
};

const pollDueTaskReminders = async () => {
  if (schedulerUsers === 0 || !await ensurePermission() || schedulerUsers === 0) return;
  const now = new Date().toISOString();
  const reminders = await invoke<DueTaskReminder[]>("list_due_task_reminders", { now });
  if (schedulerUsers === 0) return;
  for (const reminder of reminders) {
    try {
      // Claim first so a failed database write cannot happen after the OS notification
      // has already been shown. The conditional update also deduplicates other windows.
      await invoke("mark_task_reminder_delivered", { taskId: reminder.taskId, notifiedAt: now });
      try {
        sendNotification({
          title: `任务提醒 · ${reminder.company}`,
          body: `${reminder.title}\n${reminder.role} · 截止 ${dueText(reminder.dueAt)}`,
        });
      } catch (error) {
        await invoke("release_task_reminder_delivery", { taskId: reminder.taskId, notifiedAt: now })
          .catch((releaseError) => console.error("任务通知发送失败且无法释放发送状态", releaseError));
        throw error;
      }
    } catch (error) {
      // One locked/deleted task must not prevent the rest of this due batch from firing.
      console.error(`任务通知发送失败: ${reminder.taskId}`, error);
    }
  }
};

const runPoll = () => {
  if (activePoll) return activePoll;
  activePoll = pollDueTaskReminders()
    .catch((error) => console.error("任务通知调度失败", error))
    .finally(() => { activePoll = undefined; });
  return activePoll;
};

export function startTaskNotificationScheduler() {
  if (!hasLocalDatabase) return () => undefined;
  schedulerUsers += 1;
  if (schedulerUsers === 1) {
    void runPoll();
    schedulerTimer = window.setInterval(() => { void runPoll(); }, 30_000);
  }
  let released = false;
  return () => {
    if (released) return;
    released = true;
    schedulerUsers = Math.max(0, schedulerUsers - 1);
    if (schedulerUsers === 0 && schedulerTimer !== undefined) {
      window.clearInterval(schedulerTimer);
      schedulerTimer = undefined;
    }
  };
}
