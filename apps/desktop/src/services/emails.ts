import { invoke } from "@tauri-apps/api/core";
import type { EmailSettings } from "./settings";

export type EmailStatus = "unmatched" | "pending" | "confirmed" | "ignored";

export interface RecruitmentEmail {
  id: string;
  sender: string;
  subject: string;
  receivedAt: string;
  snippet: string;
  bodyText: string;
  links: Array<{ label: string; url: string }>;
  category: string;
  suggestedStage?: string;
  status: EmailStatus;
  matchedApplicationId?: string;
  company?: string;
  role?: string;
  currentStage?: string;
  confidence: number;
  reasons: string[];
}

export interface EmailStats { thisWeek: number; pending: number; confirmed: number; unmatched: number }
export interface EmailSyncResult { fetched: number; recognized: number; matched: number }

export const listEmailMessages = () => invoke<RecruitmentEmail[]>("list_email_messages");
export const getEmailStats = () => invoke<EmailStats>("get_email_stats");
export const syncEmails = () => invoke<EmailSyncResult>("sync_emails");
export const confirmEmailMatch = (id: string) => invoke<void>("confirm_email_match", { id });
export const ignoreEmail = (id: string) => invoke<void>("ignore_email", { id });
export const rematchEmail = (id: string) => invoke<void>("rematch_email", { id });
export const authorizeEmailOAuth = (settings: EmailSettings) => invoke<void>("authorize_email_oauth", { settings });
