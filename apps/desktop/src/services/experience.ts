import { invoke } from "@tauri-apps/api/core";

export interface ExperienceLink {
  id: string;
  applicationId: string;
  source: "link" | "manual";
  url?: string;
  title: string;
  importedAt: string;
  status: "待分析" | "已提取" | "分析失败";
  questions: string[];
  errorMessage?: string;
}

export const listInterviewExperienceSources = (applicationId?: string) =>
  invoke<ExperienceLink[]>("list_interview_experience_sources", { applicationId });

export const importInterviewExperienceLink = (applicationId: string, url: string) =>
  invoke<ExperienceLink>("import_interview_experience_link", { applicationId, url });

export const createManualInterviewExperience = (applicationId: string, title: string, questions: string[]) =>
  invoke<ExperienceLink>("create_manual_interview_experience", { applicationId, title, questions });

export const analyzeInterviewExperienceLink = (id: string) =>
  invoke<ExperienceLink>("analyze_interview_experience_link", { id });

export const deleteInterviewExperienceSource = (id: string) =>
  invoke<void>("delete_interview_experience_source", { id });

export const updateInterviewExperienceQuestions = (id: string, questions: string[]) =>
  invoke<ExperienceLink>("update_interview_experience_questions", { id, questions });
