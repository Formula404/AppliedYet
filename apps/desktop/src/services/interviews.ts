import { invoke } from "@tauri-apps/api/core";

export interface InterviewQuestion {
  id: string;
  prompt: string;
  source: "面经" | "AI 简历题" | "真实面试" | "个人题库";
  answer: string;
  score?: number;
  evaluation?: string;
}

interface StoredInterviewSession {
  id: string;
  applicationId: string;
  sessionType: "模拟面试" | "真实面试";
  round: string;
  createdAt: string;
  duration: string;
  status: "进行中" | "待复盘" | "复盘完成";
  currentQuestionIndex: number;
  reviewSummary?: string;
  questions: InterviewQuestion[];
}

export interface InterviewSession extends Omit<StoredInterviewSession, "sessionType"> {
  type: StoredInterviewSession["sessionType"];
}

export interface CreateInterviewQuestion {
  prompt: string;
  source: InterviewQuestion["source"];
  answer?: string;
}

export interface QuestionBankItem {
  id: string;
  prompt: string;
  category: string;
  bestAnswer: string;
  mastery: "待加强" | "练习中" | "熟悉" | "掌握";
  source: string;
  occurrenceCount: number;
  lastSeenAt: string;
}

export type SaveQuestionBankInput = Pick<QuestionBankItem, "prompt" | "category" | "bestAnswer" | "mastery">;

const mapSession = ({ sessionType, ...session }: StoredInterviewSession): InterviewSession => ({ ...session, type: sessionType });

export const listInterviewSessions = async () =>
  (await invoke<StoredInterviewSession[]>("list_interview_sessions")).map(mapSession);

export const createMockInterviewSession = async (applicationId: string, questions: CreateInterviewQuestion[]) =>
  mapSession(await invoke<StoredInterviewSession>("create_mock_interview_session", { applicationId, questions }));

export const updateInterviewSessionAnswer = (sessionId: string, questionId: string, answer: string) =>
  invoke<void>("update_interview_session_answer", { sessionId, questionId, answer });

export const updateInterviewSessionProgress = (id: string, questionIndex: number) =>
  invoke<void>("update_interview_session_progress", { id, questionIndex });

export const completeInterviewSession = async (id: string) =>
  mapSession(await invoke<StoredInterviewSession>("complete_interview_session", { id }));

export const generateInterviewReview = async (id: string) =>
  mapSession(await invoke<StoredInterviewSession>("generate_interview_review", { id }));

export const importInterviewTranscript = async (applicationId: string, transcript: string) =>
  mapSession(await invoke<StoredInterviewSession>("import_interview_transcript", { applicationId, transcript }));

export const deleteInterviewSession = (id: string) =>
  invoke<void>("delete_interview_session", { id });

export const listQuestionBankItems = () => invoke<QuestionBankItem[]>("list_question_bank_items");

export const saveQuestionBankItem = (id: string | undefined, input: SaveQuestionBankInput) =>
  invoke<QuestionBankItem>("save_question_bank_item", { id, input });

export const deleteQuestionBankItem = (id: string) => invoke<void>("delete_question_bank_item", { id });
