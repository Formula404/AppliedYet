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
