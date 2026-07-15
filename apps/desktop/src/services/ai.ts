import { invoke } from "@tauri-apps/api/core";
import { hasLocalDatabase } from "./applications";
import { demoAiCalls, demoPreparation, demoResumeQuestions } from "../data/demo";

export interface FocusArea { title: string; reason: string; priority: "high" | "medium" | "low" }
export interface PredictedQuestion { question: string; rationale: string; sourceBasis: string[] }
export interface ActionItem { action: string; estimatedMinutes: number }
export interface InterviewPreparationContent {
  summary: string;
  resumeMatch?: { summary: string; strengths: string[]; risks: string[]; evidenceToPrepare: string[] };
  focusAreas: FocusArea[];
  predictedQuestions: PredictedQuestion[];
  actionPlan: ActionItem[];
  sourceNotes: string[];
}
export interface StoredInterviewPreparation {
  id: string;
  applicationId: string;
  aiCallId: string;
  content: InterviewPreparationContent;
  sources: Array<Record<string, unknown>>;
  model: string;
  createdAt: string;
}
export interface AiCallSummary {
  id: string;
  feature: string;
  model: string;
  status: "running" | "succeeded" | "failed";
  attempts: number;
  durationMs?: number;
  inputSources: Array<Record<string, unknown>>;
  errorMessage?: string;
  createdAt: string;
}
export interface ProviderConnectionResult { ok: boolean; model: string; durationMs: number; message: string }

export const testAiProvider = () => invoke<ProviderConnectionResult>("test_ai_provider");
export const generateInterviewPreparation = (applicationId: string) => hasLocalDatabase ? invoke<StoredInterviewPreparation>("generate_interview_preparation", { applicationId }) : Promise.resolve(demoPreparation(applicationId));
export const getLatestInterviewPreparation = (applicationId: string) => hasLocalDatabase ? invoke<StoredInterviewPreparation | null>("get_latest_interview_preparation", { applicationId }) : Promise.resolve(demoPreparation(applicationId));
export const listApplicationAiCalls = (applicationId: string) => hasLocalDatabase ? invoke<AiCallSummary[]>("list_application_ai_calls", { applicationId }) : Promise.resolve(demoAiCalls(applicationId));
export const generateResumeQuestions = (applicationId: string, count: number) => hasLocalDatabase ? invoke<PredictedQuestion[]>("generate_resume_questions", { applicationId, count }) : Promise.resolve(demoResumeQuestions(count));

export interface ProcessingJobResult { id: string; kind: string; status: string; result?: Record<string, unknown>; durationMs?: number }
export const parseDocument = (path: string, applicationId?: string) => invoke<ProcessingJobResult>("parse_document", { path, applicationId });
export const transcribeAudio = (path: string, applicationId?: string) => invoke<ProcessingJobResult>("transcribe_audio", { path, applicationId });
