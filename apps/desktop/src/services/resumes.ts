import { invoke } from "@tauri-apps/api/core";

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

export const listResumeProfiles = () => invoke<ResumeProfile[]>("list_resume_profiles");
export const importResumeProfile = (path: string, confirmAiSend = false) => invoke<ResumeImportResult>("import_resume_profile", { input: { path, confirmAiSend: Boolean(confirmAiSend) } });
export const updateResumeProfile = (id: string, input: UpdateResumeProfileInput) => invoke<ResumeProfile>("update_resume_profile", { id, input });
export const setPrimaryResumeProfile = (id: string) => invoke<void>("set_primary_resume_profile", { id });
export const deleteResumeProfile = (id: string) => invoke<void>("delete_resume_profile", { id });
export const duplicateResumeProfile = (id: string) => invoke<ResumeProfile>("duplicate_resume_profile", { id });
export const setResumeProfileArchived = (id: string, archived: boolean) => invoke<void>("set_resume_profile_archived", { id, archived });
export const createBlankResumeProfile = (name: string) => invoke<ResumeProfile>("create_blank_resume_profile", { name });
