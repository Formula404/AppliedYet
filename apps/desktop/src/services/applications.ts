import { invoke, isTauri } from "@tauri-apps/api/core";
import type { Application, ApplicationDetail, ApplicationEvent, ApplicationTask } from "../types";

export interface CreateApplicationInput {
  companyName: string;
  positionTitle: string;
  location?: string;
  channel?: string;
  appliedAt?: string;
  jdRaw?: string;
}

export const hasLocalDatabase = isTauri();

export function listApplications() {
  return invoke<Application[]>("list_applications");
}

export function createApplication(input: CreateApplicationInput) {
  return invoke<Application>("create_application", { input });
}

export function updateApplicationStage(id: string, stage: string) {
  return invoke<void>("update_application_stage", { id, stage });
}

export type UpdateApplicationDetailInput = Omit<ApplicationDetail, "id" | "createdAt" | "updatedAt" | "tasks" | "events">;

export interface CreateTaskInput {
  title: string;
  description?: string;
  priority: number;
  dueAt?: string;
  remindAt?: string;
  applicationStage?: string;
}

export interface CreateEventInput {
  title: string;
  content?: string;
  happenedAt?: string;
}

export function getApplicationDetail(id: string) {
  return invoke<ApplicationDetail>("get_application_detail", { id });
}

export function updateApplicationDetail(id: string, input: UpdateApplicationDetailInput) {
  return invoke<ApplicationDetail>("update_application_detail", { id, input });
}

export function createApplicationTask(applicationId: string, input: CreateTaskInput) {
  return invoke<ApplicationTask>("create_application_task", { applicationId, input });
}

export function setApplicationTaskStatus(taskId: string, status: ApplicationTask["status"]) {
  return invoke<ApplicationTask>("set_application_task_status", { taskId, status });
}

export function createApplicationEvent(applicationId: string, input: CreateEventInput) {
  return invoke<ApplicationEvent>("create_application_event", { applicationId, input });
}
