import { invoke } from "@tauri-apps/api/core";
import { hasLocalDatabase } from "./applications";
import { listDemoQuestionBankItems } from "../data/demo";

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
  systemMastery: QuestionBankItem["mastery"];
  manualMastery?: QuestionBankItem["mastery"];
  membershipStatus: "active" | "archived";
  realInterviewCount: number;
  askedCount: number;
  practiceCount: number;
  referenceCount: number;
  companyCount: number;
  legacyCount: number;
  lastRealAskedAt?: string;
  lastPracticedAt?: string;
  nextReviewAt?: string;
  createdAt: string;
  sources: string[];
  needsReview: boolean;
}

export type SaveQuestionBankInput = Pick<QuestionBankItem, "prompt" | "category" | "bestAnswer" | "mastery"> & { forceNew?: boolean };
export type QuestionBankSort = "review_priority" | "real_frequency" | "reference_frequency" | "last_real_asked" | "last_practiced" | "created_at";
export interface ListQuestionBankInput {
  query?: string;
  status?: "active" | "archived";
  reviewState?: "due";
  mastery?: QuestionBankItem["mastery"][];
  sort?: QuestionBankSort;
  direction?: "asc" | "desc";
  pageSize?: 30 | 50 | 100;
  cursor?: string;
}
export interface QuestionBankFacets { active: number; due: number; pendingMatches: number; archived: number }
export interface QuestionBankPage { items: QuestionBankItem[]; total: number; nextCursor?: string; facets: QuestionBankFacets }
export interface QuestionEvidence {
  id: string; eventType: string; sourceType: string; sourceId: string; sourceItemId: string;
  prompt: string; company?: string; position?: string; round?: string; occurredAt: string; verificationState: string;
}
export interface QuestionBankDetail extends QuestionBankItem { variants: string[]; evidence: QuestionEvidence[] }
export interface QuestionMatchCandidate { question: QuestionBankItem; score: number; reason: string }
export interface MergeQuestionInput { targetId: string; sourceId: string; displayPrompt: string; reason?: string }
export interface SplitQuestionInput { questionId: string; observationIds: string[]; displayPrompt: string }

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

export const generateInterviewReview = async (id: string, confirmAiSend: boolean) =>
  mapSession(await invoke<StoredInterviewSession>("generate_interview_review", { id, confirmAiSend }));

export const importInterviewTranscript = async (applicationId: string, transcript: string, confirmAiSend: boolean) =>
  mapSession(await invoke<StoredInterviewSession>("import_interview_transcript", { applicationId, transcript, confirmAiSend }));

export const importProcessingJob = async (jobId: string, confirmAiSend: boolean) =>
  mapSession(await invoke<StoredInterviewSession>("import_processing_job", { jobId, confirmAiSend }));

export const deleteInterviewSession = (id: string) =>
  invoke<void>("delete_interview_session", { id });

const defaultListInput: Required<Pick<ListQuestionBankInput, "status" | "sort" | "direction" | "pageSize">> =
  { status: "active", sort: "review_priority", direction: "desc", pageSize: 30 };

export const listQuestionBankItems = async (input: ListQuestionBankInput = {}): Promise<QuestionBankPage> => {
  const resolved = { ...defaultListInput, ...input };
  if (hasLocalDatabase) return invoke<QuestionBankPage>("list_question_bank_items", { input: resolved });
  const items = (await listDemoQuestionBankItems())
    .filter((item) => (input.status ?? "active") === item.membershipStatus)
    .filter((item) => !input.query || `${item.prompt} ${item.category} ${item.sources.join(" ")}`.toLowerCase().includes(input.query.toLowerCase()));
  return { items: items.slice(0, resolved.pageSize), total: items.length, facets: { active: items.length, due: items.filter((item) => item.needsReview).length, pendingMatches: 0, archived: 0 } };
};

export const getQuestionBankItem = (id: string) => invoke<QuestionBankDetail>("get_question_bank_item", { id });

export const saveQuestionBankItem = (id: string | undefined, input: SaveQuestionBankInput) =>
  invoke<QuestionBankItem>("save_question_bank_item", { id, input });

export const deleteQuestionBankItem = (id: string) => invoke<void>("delete_question_bank_item", { id });
export const archiveQuestionBankItem = (id: string) => invoke<void>("archive_question_bank_item", { id });
export const restoreQuestionBankItem = (id: string) => invoke<void>("restore_question_bank_item", { id });
export const listQuestionMatchCandidates = (prompt: string, excludeId?: string) =>
  invoke<QuestionMatchCandidate[]>("list_question_match_candidates", { prompt, excludeId });
export const resolveQuestionMatch = (leftId: string, rightId: string, action: "merge" | "same_topic" | "keep_separate", reason = "") =>
  invoke<string | undefined>("resolve_question_match", { leftId, rightId, action, reason });
export const mergeQuestionBankItems = (input: MergeQuestionInput) =>
  invoke<string>("merge_question_bank_items", { input });
export const splitQuestionBankItem = (input: SplitQuestionInput) =>
  invoke<QuestionBankItem>("split_question_bank_item", { input });
export const undoQuestionMerge = (auditId: string) => invoke<void>("undo_question_merge", { auditId });
