import { invoke } from "@tauri-apps/api/core";
import { hasLocalDatabase } from "./applications";
import { createDemoResume, deleteDemoResume, listDemoResumes, primaryDemoResume, replaceDemoResume } from "../data/demo";

export interface ResumeProfile {
  id: string;
  name: string;
  filePath?: string;
  fileFormat?: string;
  parsedText: string;
  personalInfo: string;
  educationBackground: string;
  internshipExperience: string;
  projectExperience: string;
  professionalSkills: string;
  academicAchievements: string;
  skillCertificates: string;
  targetDirection: string;
  notes: string;
  parentProfileId?: string;
  linkedApplicationCount: number;
  assessmentCount: number;
  interviewCount: number;
  offerCount: number;
  isPrimary: boolean;
  archivedAt?: string;
  createdAt: string;
  updatedAt: string;
}

export interface UpdateResumeProfileInput {
  name: string;
  personalInfo: string;
  educationBackground: string;
  internshipExperience: string;
  projectExperience: string;
  professionalSkills: string;
  academicAchievements: string;
  skillCertificates: string;
  targetDirection: string;
  notes: string;
}

export interface ResumeImportResult {
  profile: ResumeProfile;
  aiStatus: "succeeded" | "skipped" | "failed";
  warning?: string;
}

export const listResumeProfiles = () => hasLocalDatabase ? invoke<ResumeProfile[]>("list_resume_profiles") : listDemoResumes();
export const importResumeProfile = (path: string, confirmAiSend = false) => invoke<ResumeImportResult>("import_resume_profile", { input: { path, confirmAiSend: Boolean(confirmAiSend) } });
export const updateResumeProfile = (id: string, input: UpdateResumeProfileInput) => hasLocalDatabase ? invoke<ResumeProfile>("update_resume_profile", { id, input }) : replaceDemoResume(id, input);
export const setPrimaryResumeProfile = (id: string) => hasLocalDatabase ? invoke<void>("set_primary_resume_profile", { id }) : primaryDemoResume(id);
export const deleteResumeProfile = (id: string) => hasLocalDatabase ? invoke<void>("delete_resume_profile", { id }) : deleteDemoResume(id);
export const duplicateResumeProfile = async (id: string) => hasLocalDatabase ? invoke<ResumeProfile>("duplicate_resume_profile", { id }) : createDemoResume(`${(await listDemoResumes()).find((item) => item.id === id)?.name || "简历"} · 副本`, (await listDemoResumes()).find((item) => item.id === id));
export const setResumeProfileArchived = (id: string, archived: boolean) => hasLocalDatabase ? invoke<void>("set_resume_profile_archived", { id, archived }) : replaceDemoResume(id, { archivedAt: archived ? new Date().toISOString() : undefined }).then(() => undefined);
export const createBlankResumeProfile = (name: string) => hasLocalDatabase ? invoke<ResumeProfile>("create_blank_resume_profile", { name }) : createDemoResume(name);
