import { invoke } from "@tauri-apps/api/core";
import type { StatusTone } from "../types";
import { hasLocalDatabase } from "./applications";
import { demoActivity, demoAnalytics, demoDashboard } from "../data/demo";

export interface DashboardSummary {
  total: number;
  active: number;
  assessments: number;
  interviews: number;
  waiting: number;
  offers: number;
  rejected: number;
}

export interface DashboardTask {
  id: string;
  applicationId: string;
  title: string;
  company: string;
  role: string;
  dueAt: string;
  priority: number;
  status: "todo" | "doing" | "done";
  overdue: boolean;
  tone: StatusTone;
}

export interface DashboardEvent {
  id: string;
  applicationId: string;
  title: string;
  company: string;
  role: string;
  scheduledAt: string;
  kind: "task" | "next_action" | "milestone";
  tone: StatusTone;
}

export interface DashboardData {
  summary: DashboardSummary;
  tasks: DashboardTask[];
  events: DashboardEvent[];
}

export interface ActivitySummary {
  streakDays: number;
  thisWeekApplications: number;
  previousWeekApplications: number;
  dailyActivity: number[];
}

export interface AnalyticsPeriod {
  label: string;
  applications: number;
  interviews: number;
}

export interface AnalyticsData {
  total: number;
  thisMonth: number;
  previousMonth: number;
  assessments: number;
  interviews: number;
  offers: number;
  averageFeedbackDays: number | null;
  daily: AnalyticsPeriod[];
  weekly: AnalyticsPeriod[];
  directions: Array<{ name: string; count: number }>;
}

export const getActivitySummary = () => hasLocalDatabase ? invoke<ActivitySummary>("get_activity_summary") : Promise.resolve(demoActivity());
export const getAnalytics = () => hasLocalDatabase ? invoke<AnalyticsData>("get_analytics") : Promise.resolve(demoAnalytics());

export function getDashboard(monthStart: string, monthEnd: string, todayStart: string, todayEnd: string) {
  return hasLocalDatabase ? invoke<DashboardData>("get_dashboard", { monthStart, monthEnd, todayStart, todayEnd }) : Promise.resolve(demoDashboard());
}
