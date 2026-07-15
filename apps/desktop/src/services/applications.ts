import { invoke, isTauri } from "@tauri-apps/api/core";
import type { Application, ApplicationDetail, ApplicationEvent, ApplicationTask } from "../types";

export interface CreateApplicationInput {
  companyName: string;
  companyShortName?: string;
  industry?: string;
  companyType?: string;
  website?: string;
  companyNotes?: string;
  positionTitle: string;
  department?: string;
  location?: string;
  recruitmentType?: string;
  jobCode?: string;
  sourceUrl?: string;
  channel?: string;
  appliedAt?: string;
  priority?: number;
  jdRaw?: string;
  resumeProfileId?: string;
}

export const hasLocalDatabase = isTauri();

export function listApplications() {
  return invoke<Application[]>("list_applications");
}

export function exportApplicationsExcel(path: string) {
  return invoke<number>("export_applications_excel", { path });
}

export function createApplication(input: CreateApplicationInput) {
  return invoke<Application>("create_application", { input });
}

export function updateApplicationStage(id: string, stage: string) {
  return invoke<void>("update_application_stage", { id, stage });
}

export type UpdateApplicationDetailInput = Omit<ApplicationDetail, "id" | "createdAt" | "updatedAt" | "archivedAt" | "tasks" | "events">;

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

export function updateApplicationTask(taskId: string, input: CreateTaskInput) {
  return invoke<ApplicationTask>("update_application_task", { taskId, input });
}

export function deleteApplicationTask(taskId: string) {
  return invoke<void>("delete_application_task", { taskId });
}

export function setApplicationArchived(id: string, archived: boolean) {
  return invoke<void>("set_application_archived", { id, archived });
}

export function deleteArchivedApplication(id: string) {
  return invoke<void>("delete_archived_application", { id });
}

export function revertApplicationEvent(eventId: string) {
  return invoke<ApplicationDetail>("revert_application_event", { eventId });
}

export function updateApplicationEventTime(eventId: string, happenedAt: string) {
  return invoke<ApplicationDetail>("update_application_event_time", { eventId, happenedAt });
}

export function createApplicationEvent(applicationId: string, input: CreateEventInput) {
  return invoke<ApplicationEvent>("create_application_event", { applicationId, input });
}
