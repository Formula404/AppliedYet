export type StatusTone = "blue" | "green" | "orange" | "purple" | "teal" | "red" | "gray";

export interface Application {
  id: string;
  company: string;
  companyMark: string;
  role: string;
  city: string;
  stage: string;
  stageTone: StatusTone;
  priority: "高" | "中" | "普通";
  nextStep: string;
  nextTime: string;
  progress: number;
  updated: string;
  risk?: string;
}

export interface MailItem {
  id: string;
  company: string;
  role: string;
  subject: string;
  summary: string;
  type: string;
  time: string;
  confidence: number;
  status: "已更新流程" | "待确认" | "已自动匹配";
}

export interface TaskItemData {
  id: string;
  title: string;
  relation: string;
  time: string;
  tone: StatusTone;
  overdue?: boolean;
}

export interface ApplicationTask {
  id: string;
  title: string;
  description?: string;
  priority: number;
  status: "todo" | "doing" | "done" | "canceled";
  dueAt?: string;
  remindAt?: string;
  applicationStage?: string;
  sourceType: string;
  completedAt?: string;
  createdAt: string;
}

export interface ApplicationEvent {
  id: string;
  eventType: string;
  title: string;
  content?: string;
  sourceType: "manual" | "email" | "ai" | "system";
  stageBefore?: string;
  stageAfter?: string;
  happenedAt: string;
  reversible: boolean;
  revertedAt?: string;
}

export interface ApplicationDetail {
  id: string;
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
  jdRaw?: string;
  appliedAt?: string;
  channel?: string;
  priority: number;
  currentStage: string;
  nextAction?: string;
  nextActionDueAt?: string;
  createdAt: string;
  updatedAt: string;
  tasks: ApplicationTask[];
  events: ApplicationEvent[];
}
