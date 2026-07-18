import { invoke } from "@tauri-apps/api/core";
import type { EmailAccountSettings } from "./settings";
import { hasLocalDatabase, type CreateApplicationInput } from "./applications";
import type { Application } from "../types";
import { demoEmailStats, listDemoEmails, setDemoEmailApplication, setDemoEmailStatus } from "../data/demo";

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

export const listEmailMessages = () => hasLocalDatabase ? invoke<RecruitmentEmail[]>("list_email_messages") : listDemoEmails();
export const getEmailStats = () => hasLocalDatabase ? invoke<EmailStats>("get_email_stats") : demoEmailStats();
export const syncEmails = () => hasLocalDatabase ? invoke<EmailSyncResult>("sync_emails") : Promise.resolve({ fetched: 5, recognized: 5, matched: 4 });
export const confirmEmailMatch = (id: string) => hasLocalDatabase ? invoke<void>("confirm_email_match", { id }) : setDemoEmailStatus(id, "confirmed");
export const ignoreEmail = (id: string) => hasLocalDatabase ? invoke<void>("ignore_email", { id }) : setDemoEmailStatus(id, "ignored");
export const rematchEmail = (id: string) => hasLocalDatabase ? invoke<void>("rematch_email", { id }) : setDemoEmailStatus(id, "pending");
export const attachEmailToApplication = (emailId: string, applicationId: string) => hasLocalDatabase
  ? invoke<void>("attach_email_to_application", { emailId, applicationId })
  : setDemoEmailApplication(emailId, applicationId);
export const createApplicationFromEmail = (emailId: string, input: CreateApplicationInput) =>
  invoke<Application>("create_application_from_email", { emailId, input });
export const authorizeEmailOAuth = (account: EmailAccountSettings) => invoke<void>("authorize_email_oauth", { account });
