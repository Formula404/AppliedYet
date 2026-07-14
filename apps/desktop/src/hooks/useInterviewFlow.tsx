import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import { applications as initialApplications } from "../data/mock";
import type { Application } from "../types";
import {
  createApplication as persistApplication,
  hasLocalDatabase,
  listApplications,
  setApplicationArchived as persistApplicationArchived,
  updateApplicationStage as persistApplicationStage,
  type CreateApplicationInput,
} from "../services/applications";

export interface ExperienceLink {
  id: string;
  applicationId: string;
  source: "link" | "manual";
  url?: string;
  title: string;
  importedAt: string;
  status: "待分析" | "已提取" | "分析失败";
  questions: string[];
}

export interface InterviewQuestion {
  id: string;
  prompt: string;
  source: "面经" | "AI 简历题" | "真实面试";
  answer: string;
  score?: number;
  evaluation?: string;
}

export interface InterviewSession {
  id: string;
  applicationId: string;
  type: "模拟面试" | "真实面试";
  round: string;
  createdAt: string;
  duration: string;
  status: "进行中" | "待复盘" | "复盘完成";
  questions: InterviewQuestion[];
}

interface CreateMockOptions {
  applicationId: string;
  questionCount: number;
  useExperience: boolean;
  useAi: boolean;
  resumeQuestions?: string[];
}

interface InterviewFlowValue {
  applications: Application[];
  eligibleApplications: Application[];
  experienceLinks: ExperienceLink[];
  sessions: InterviewSession[];
  selectedApplicationId: string;
  applicationsLoading: boolean;
  applicationsError: string | null;
  setSelectedApplicationId: (id: string) => void;
  updateApplicationStage: (id: string, stage: string, stageTone: Application["stageTone"]) => void;
  createApplication: (input: CreateApplicationInput) => Promise<Application>;
  refreshApplications: () => Promise<void>;
  archiveApplication: (id: string, archived: boolean) => Promise<void>;
  importExperienceLink: (applicationId: string, url: string) => string;
  addManualExperience: (applicationId: string, title: string, questions: string[]) => string;
  analyzeExperienceLink: (id: string) => void;
  createMockSession: (options: CreateMockOptions) => string;
  updateSessionAnswer: (sessionId: string, questionId: string, answer: string) => void;
  completeSession: (id: string) => void;
}

const InterviewFlowContext = createContext<InterviewFlowValue | null>(null);
const isInterviewEligible = (application: Application) =>
  !application.archived && !application.stage.includes("拒绝") && !application.stage.toLowerCase().includes("offer");
const makeId = (prefix: string) => `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`;

const experienceQuestions = [
  "介绍一下你负责过的高并发项目，以及你在其中承担的工作。",
  "如何保证分布式事务最终一致性？",
  "线上服务延迟突然升高时，你会按照什么顺序排查？",
  "为什么选择消息队列，而不是同步调用？",
  "说说你对 JVM 内存模型和常见 GC 问题的理解。",
];

const resumeQuestions = [
  "简历中提到订单系统重构，请说明重构前的核心问题。",
  "这个项目的关键指标提升了多少，你是如何验证的？",
  "项目中最困难的一次技术决策是什么？",
  "如果重新做一次，你会改变哪个方案？",
  "请具体说明你个人的贡献，而不是团队整体成果。",
  "讲一次你推动跨团队协作的经历。",
];

const reviewedQuestion = (id: string, prompt: string, answer: string, score: number, evaluation: string): InterviewQuestion => ({
  id, prompt, answer, score, evaluation, source: "真实面试",
});

export function InterviewFlowProvider({ children }: { children: ReactNode }) {
  const [applications, setApplications] = useState<Application[]>(() => hasLocalDatabase ? [] : initialApplications.map((item) => ({ ...item })));
  const [applicationsLoading, setApplicationsLoading] = useState(hasLocalDatabase);
  const [applicationsError, setApplicationsError] = useState<string | null>(null);
  const [selectedApplicationId, setSelectedApplicationId] = useState("ant");
  const [experienceLinks, setExperienceLinks] = useState<ExperienceLink[]>([
    { id: "link-ant-1", applicationId: "ant", source: "link", url: "https://www.nowcoder.com/discuss/ant-backend", title: "蚂蚁后端一面经验整理", importedAt: "今天 10:24", status: "已提取", questions: experienceQuestions },
    { id: "link-tencent-1", applicationId: "tencent", source: "link", url: "https://www.nowcoder.com/discuss/tencent-server", title: "腾讯后台开发二面面经", importedAt: "昨天", status: "已提取", questions: experienceQuestions.slice(1) },
  ]);
  const [sessions, setSessions] = useState<InterviewSession[]>([
    {
      id: "real-ant-1", applicationId: "ant", type: "真实面试", round: "技术一面", createdAt: "今天 09:30", duration: "48 分钟", status: "复盘完成",
      questions: [
        reviewedQuestion("real-ant-q1", "介绍一下订单系统重构项目。", "我负责拆分订单服务，并通过异步化降低主链路耗时。", 82, "结构清晰，也说明了个人职责；如果补充重构前后的延迟和吞吐量，结论会更有说服力。"),
        reviewedQuestion("real-ant-q2", "如何处理消息重复消费？", "通过业务唯一键和状态机实现幂等，同时记录消费日志。", 88, "方案完整，覆盖了业务幂等和可追溯性。可以进一步说明数据库唯一约束与并发冲突处理。"),
        reviewedQuestion("real-ant-q3", "项目中遇到的最大分歧是什么？", "团队对同步还是异步方案有争议，我组织压测后用数据推进决策。", 76, "有行动和结果，但缺少分歧双方诉求以及最终量化效果，STAR 结构还可以更完整。"),
      ],
    },
    {
      id: "mock-tencent-1", applicationId: "tencent", type: "模拟面试", round: "技术综合模拟", createdAt: "昨天 20:16", duration: "26 分钟", status: "待复盘",
      questions: [
        { id: "mock-t-q1", prompt: experienceQuestions[1], source: "面经", answer: "我会优先选择本地事务消息或事务发件箱模式。", score: 72, evaluation: "方向正确，但需要补充失败重试、幂等和对账闭环。" },
        { id: "mock-t-q2", prompt: resumeQuestions[0], source: "AI 简历题", answer: "原系统模块耦合较高，发布和扩容都比较困难。", score: 68, evaluation: "识别了架构问题，但回答偏抽象，应结合具体故障或指标说明为什么必须重构。" },
        { id: "mock-t-q3", prompt: resumeQuestions[4], source: "AI 简历题", answer: "我负责核心方案设计、压测以及迁移过程中的问题处理。", score: 79, evaluation: "个人边界表达清楚，可以继续补充每项工作的成果与验证方式。" },
      ],
    },
  ]);

  const refreshApplications = useCallback(async () => {
    if (!hasLocalDatabase) return;
    setApplicationsLoading(true);
    try {
      const items = await listApplications();
      setApplications(items);
      setSelectedApplicationId((current) => items.some((item) => item.id === current) ? current : (items[0]?.id || ""));
      setApplicationsError(null);
    } catch (error) {
      setApplicationsError(String(error));
      throw error;
    } finally {
      setApplicationsLoading(false);
    }
  }, []);

  useEffect(() => {
    if (!hasLocalDatabase) return;
    refreshApplications().catch(() => undefined);
  }, [refreshApplications]);

  const eligibleApplications = useMemo(() => applications.filter(isInterviewEligible), [applications]);

  const value = useMemo<InterviewFlowValue>(() => ({
    applications,
    eligibleApplications,
    experienceLinks,
    sessions,
    selectedApplicationId,
    applicationsLoading,
    applicationsError,
    refreshApplications,
    archiveApplication: async (id, archived) => {
      if (hasLocalDatabase) {
        await persistApplicationArchived(id, archived);
        await refreshApplications();
      } else {
        setApplications((current) => current.map((item) => item.id === id ? { ...item, archived } : item));
      }
    },
    setSelectedApplicationId,
    updateApplicationStage: (id, stage, stageTone) => {
      const previous = applications.find((item) => item.id === id);
      setApplications((current) => current.map((item) => item.id === id ? { ...item, stage, stageTone, updated: "刚刚" } : item));
      if (hasLocalDatabase) {
        persistApplicationStage(id, stage).catch((error) => {
          if (previous) setApplications((current) => current.map((item) => item.id === id ? previous : item));
          setApplicationsError(String(error));
        });
      }
      if ((stage.includes("拒绝") || stage.toLowerCase().includes("offer")) && selectedApplicationId === id) {
        const next = applications.find((item) => item.id !== id && isInterviewEligible(item));
        if (next) setSelectedApplicationId(next.id);
      }
    },
    createApplication: async (input) => {
      if (!hasLocalDatabase) {
        const created: Application = {
          id: makeId("application"), company: input.companyName, companyMark: input.companyName[0] || "?",
          role: input.positionTitle, city: input.location || "未填写", stage: "已投递", stageTone: "blue",
          priority: "中", nextStep: "待安排", nextTime: "待安排", progress: 1, updated: "刚刚",
        };
        setApplications((current) => [created, ...current]);
        return created;
      }
      try {
        const created = await persistApplication(input);
        setApplications((current) => [created, ...current]);
        setApplicationsError(null);
        return created;
      } catch (error) {
        setApplicationsError(String(error));
        throw error;
      }
    },
    importExperienceLink: (applicationId, url) => {
      const id = makeId("link");
      let title = "待分析的网页面经";
      try { title = `${new URL(url).hostname} · 面经帖子`; } catch { /* UI 已校验 URL */ }
      setExperienceLinks((current) => [{ id, applicationId, source: "link", url, title, importedAt: "刚刚", status: "待分析", questions: [] }, ...current]);
      return id;
    },
    addManualExperience: (applicationId, title, questions) => {
      const id = makeId("manual");
      setExperienceLinks((current) => [{ id, applicationId, source: "manual", title: title || "人工整理面经", importedAt: "刚刚", status: "已提取", questions }, ...current]);
      return id;
    },
    analyzeExperienceLink: (id) => {
      setExperienceLinks((current) => current.map((item) => item.id === id ? { ...item, status: "已提取", questions: experienceQuestions } : item));
    },
    createMockSession: ({ applicationId, questionCount, useExperience, useAi, resumeQuestions: generatedResumeQuestions }) => {
      const id = makeId("mock");
      const imported = experienceLinks.filter((link) => link.applicationId === applicationId && link.status === "已提取").flatMap((link) => link.questions);
      const pool = [
        ...(useExperience ? imported.map((prompt) => ({ prompt, source: "面经" as const })) : []),
        ...(useAi ? (generatedResumeQuestions?.length ? generatedResumeQuestions : resumeQuestions).map((prompt) => ({ prompt, source: "AI 简历题" as const })) : []),
      ];
      const questions = Array.from({ length: questionCount }, (_, index): InterviewQuestion => ({
        id: `${id}-q${index + 1}`,
        prompt: pool[index % pool.length].prompt,
        source: pool[index % pool.length].source,
        answer: "",
      }));
      setSessions((current) => [{ id, applicationId, type: "模拟面试", round: "技术综合模拟", createdAt: "刚刚", duration: "进行中", status: "进行中", questions }, ...current]);
      return id;
    },
    updateSessionAnswer: (sessionId, questionId, answer) => {
      setSessions((current) => current.map((session) => session.id !== sessionId ? session : {
        ...session,
        questions: session.questions.map((question) => question.id === questionId ? { ...question, answer } : question),
      }));
    },
    completeSession: (id) => {
      setSessions((current) => current.map((session) => session.id !== id ? session : {
        ...session,
        status: "待复盘",
        duration: `${Math.max(10, session.questions.length * 3)} 分钟`,
        questions: session.questions.map((question, index) => ({
          ...question,
          score: question.answer.trim() ? 72 + (index % 4) * 4 : 45,
          evaluation: question.answer.trim()
            ? "回答覆盖了主要思路，但还需要补充具体数据、方案取舍和验证结果。"
            : "本题未作答，建议先整理核心概念，再用项目案例形成完整回答。",
        })),
      }));
    },
  }), [applications, applicationsError, applicationsLoading, eligibleApplications, experienceLinks, refreshApplications, selectedApplicationId, sessions]);

  return <InterviewFlowContext.Provider value={value}>{children}</InterviewFlowContext.Provider>;
}

export function useInterviewFlow() {
  const value = useContext(InterviewFlowContext);
  if (!value) throw new Error("useInterviewFlow 必须在 InterviewFlowProvider 中使用");
  return value;
}
