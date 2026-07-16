import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { applications as initialApplications } from "../data/mock";
import type { Application } from "../types";
import {
  createApplication as persistApplication,
  deleteArchivedApplication as persistArchivedApplicationDeletion,
  hasLocalDatabase,
  listApplications,
  setApplicationArchived as persistApplicationArchived,
  updateApplicationStage as persistApplicationStage,
  type CreateApplicationInput,
} from "../services/applications";
import {
  analyzeInterviewExperienceLink,
  createManualInterviewExperience,
  deleteInterviewExperienceSource,
  importInterviewExperienceLink,
  listInterviewExperienceSources,
  updateInterviewExperienceQuestions,
  type ExperienceLink,
} from "../services/experience";
import {
  completeInterviewSession,
  createMockInterviewSession,
  deleteInterviewSession,
  generateInterviewReview,
  importInterviewTranscript,
  listInterviewSessions,
  updateInterviewSessionAnswer,
  updateInterviewSessionProgress,
  type CreateInterviewQuestion,
  type InterviewQuestion,
  type InterviewSession,
} from "../services/interviews";
export type { ExperienceLink } from "../services/experience";
export type { InterviewQuestion, InterviewSession } from "../services/interviews";

interface CreateMockOptions {
  applicationId: string;
  questionCount: number;
  useExperience: boolean;
  useAi: boolean;
  useBank?: boolean;
  bankQuestions?: string[];
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
  updateApplicationStage: (id: string, stage: string, stageTone: Application["stageTone"]) => Promise<void>;
  createApplication: (input: CreateApplicationInput) => Promise<Application>;
  refreshApplications: () => Promise<void>;
  archiveApplication: (id: string, archived: boolean) => Promise<void>;
  deleteApplication: (id: string) => Promise<void>;
  importExperienceLink: (applicationId: string, url: string) => Promise<ExperienceLink>;
  addManualExperience: (applicationId: string, title: string, questions: string[]) => Promise<ExperienceLink>;
  analyzeExperienceLink: (id: string) => Promise<ExperienceLink>;
  deleteExperienceSource: (id: string) => Promise<void>;
  updateExperienceQuestions: (id: string, questions: string[]) => Promise<ExperienceLink>;
  createMockSession: (options: CreateMockOptions) => Promise<InterviewSession>;
  updateSessionAnswer: (sessionId: string, questionId: string, answer: string) => Promise<void>;
  updateSessionProgress: (id: string, questionIndex: number) => Promise<void>;
  completeSession: (id: string) => Promise<InterviewSession>;
  reviewSession: (id: string, confirmAiSend: boolean) => Promise<InterviewSession>;
  importTranscript: (applicationId: string, transcript: string, confirmAiSend: boolean) => Promise<InterviewSession>;
  deleteSession: (id: string) => Promise<void>;
}

const InterviewFlowContext = createContext<InterviewFlowValue | null>(null);
const isTerminalStage = (stage: string) =>
  stage.includes("拒绝") || stage.includes("人才库") || stage.toLowerCase().includes("offer")
  || ["流程暂停", "流程结束", "主动放弃"].includes(stage);
const isInterviewEligible = (application: Application) =>
  !application.archived && !isTerminalStage(application.stage);
const makeId = (prefix: string) => `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`;
const requireItem = <T,>(items: readonly T[], index: number, message = "数据项不存在"): T => {
  const item = items[index];
  if (item === undefined) throw new Error(message);
  return item;
};

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
  const stageUpdateQueues = useRef(new Map<string, Promise<void>>());
  const sessionAnswerQueues = useRef(new Map<string, Promise<void>>());
  const pendingSessionAnswers = useRef(new Map<string, { sessionId: string; questionId: string; answer: string }>());
  const sessionAnswerErrors = useRef(new Map<string, unknown>());
  const applicationRefreshId = useRef(0);
  const [selectedApplicationId, setSelectedApplicationId] = useState("ant");
  const [experienceLinks, setExperienceLinks] = useState<ExperienceLink[]>(() => hasLocalDatabase ? [] : [
    { id: "link-ant-1", applicationId: "ant", source: "link", url: "https://www.nowcoder.com/discuss/ant-backend", title: "蚂蚁后端一面经验整理", importedAt: "今天 10:24", status: "已提取", questions: experienceQuestions },
    { id: "link-tencent-1", applicationId: "tencent", source: "link", url: "https://www.nowcoder.com/discuss/tencent-server", title: "腾讯后台开发二面面经", importedAt: "昨天", status: "已提取", questions: experienceQuestions.slice(1) },
  ]);
  const [sessions, setSessions] = useState<InterviewSession[]>(() => hasLocalDatabase ? [] : [
    {
      id: "real-ant-1", applicationId: "ant", type: "真实面试", round: "技术一面", createdAt: "今天 09:30", duration: "48 分钟", status: "复盘完成", currentQuestionIndex: 0,
      questions: [
        reviewedQuestion("real-ant-q1", "介绍一下订单系统重构项目。", "我负责拆分订单服务，并通过异步化降低主链路耗时。", 82, "结构清晰，也说明了个人职责；如果补充重构前后的延迟和吞吐量，结论会更有说服力。"),
        reviewedQuestion("real-ant-q2", "如何处理消息重复消费？", "通过业务唯一键和状态机实现幂等，同时记录消费日志。", 88, "方案完整，覆盖了业务幂等和可追溯性。可以进一步说明数据库唯一约束与并发冲突处理。"),
        reviewedQuestion("real-ant-q3", "项目中遇到的最大分歧是什么？", "团队对同步还是异步方案有争议，我组织压测后用数据推进决策。", 76, "有行动和结果，但缺少分歧双方诉求以及最终量化效果，STAR 结构还可以更完整。"),
      ],
    },
    {
      id: "mock-tencent-1", applicationId: "tencent", type: "模拟面试", round: "技术综合模拟", createdAt: "昨天 20:16", duration: "26 分钟", status: "待复盘", currentQuestionIndex: 0,
      questions: [
        { id: "mock-t-q1", prompt: requireItem(experienceQuestions, 1), source: "面经", answer: "我会优先选择本地事务消息或事务发件箱模式。", score: 72, evaluation: "方向正确，但需要补充失败重试、幂等和对账闭环。" },
        { id: "mock-t-q2", prompt: requireItem(resumeQuestions, 0), source: "AI 简历题", answer: "原系统模块耦合较高，发布和扩容都比较困难。", score: 68, evaluation: "识别了架构问题，但回答偏抽象，应结合具体故障或指标说明为什么必须重构。" },
        { id: "mock-t-q3", prompt: requireItem(resumeQuestions, 4), source: "AI 简历题", answer: "我负责核心方案设计、压测以及迁移过程中的问题处理。", score: 79, evaluation: "个人边界表达清楚，可以继续补充每项工作的成果与验证方式。" },
      ],
    },
  ]);

  const refreshApplications = useCallback(async () => {
    if (!hasLocalDatabase) return;
    const refreshId = ++applicationRefreshId.current;
    setApplicationsLoading(true);
    try {
      const items = await listApplications();
      if (refreshId !== applicationRefreshId.current) return;
      setApplications(items);
      const eligible = items.filter(isInterviewEligible);
      setSelectedApplicationId((current) => eligible.some((item) => item.id === current) ? current : (eligible[0]?.id || ""));
      setApplicationsError(null);
    } catch (error) {
      if (refreshId !== applicationRefreshId.current) return;
      setApplicationsError(String(error));
      throw error;
    } finally {
      if (refreshId === applicationRefreshId.current) setApplicationsLoading(false);
    }
  }, []);

  useEffect(() => {
    if (!hasLocalDatabase) return;
    refreshApplications().catch(() => undefined);
    listInterviewExperienceSources()
      .then(setExperienceLinks)
      .catch((error) => setApplicationsError(String(error)));
    listInterviewSessions()
      .then(setSessions)
      .catch((error) => setApplicationsError(String(error)));
  }, [refreshApplications]);

  useEffect(() => {
    if (!hasLocalDatabase) return;
    const changed = () => { void refreshApplications().catch(() => undefined); };
    window.addEventListener("application-index-changed", changed);
    return () => window.removeEventListener("application-index-changed", changed);
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
        if (archived && selectedApplicationId === id) {
          const next = applications.find((item) => item.id !== id && isInterviewEligible(item));
          setSelectedApplicationId(next?.id ?? "");
        }
      }
    },
    deleteApplication: async (id) => {
      if (hasLocalDatabase) {
        await persistArchivedApplicationDeletion(id);
        await refreshApplications();
      } else {
        const target = applications.find((item) => item.id === id);
        if (!target) throw new Error("投递记录不存在");
        if (!target.archived) throw new Error("只能删除已归档的投递");
        setApplications((current) => current.filter((item) => item.id !== id));
        if (selectedApplicationId === id) {
          const next = applications.find((item) => item.id !== id && isInterviewEligible(item));
          setSelectedApplicationId(next?.id ?? "");
        }
      }
    },
    setSelectedApplicationId,
    updateApplicationStage: async (id, stage, stageTone) => {
      setApplications((current) => current.map((item) => item.id === id ? { ...item, stage, stageTone, updated: "刚刚" } : item));
      if (hasLocalDatabase) {
        // 同一投递的连续拖拽必须按用户操作顺序落库，否则后发请求可能先完成，
        // 造成 current_stage 与事件时间线脱节，继而无法正确撤销。
        const previousQueue = stageUpdateQueues.current.get(id) ?? Promise.resolve();
        const queuedUpdate = previousQueue.catch(() => undefined).then(() => persistApplicationStage(id, stage));
        stageUpdateQueues.current.set(id, queuedUpdate);
        try {
          await queuedUpdate;
          setApplicationsError(null);
        } catch (error) {
          setApplicationsError(String(error));
          if (stageUpdateQueues.current.get(id) === queuedUpdate) {
            // No newer drag is waiting: reload the persisted state instead of rolling
            // back to a closure snapshot that may predate an earlier successful drag.
            await refreshApplications().catch(() => undefined);
          }
          throw error;
        } finally {
          if (stageUpdateQueues.current.get(id) === queuedUpdate) stageUpdateQueues.current.delete(id);
        }
      }
      if (isTerminalStage(stage) && selectedApplicationId === id) {
        const next = applications.find((item) => item.id !== id && isInterviewEligible(item));
        setSelectedApplicationId(next?.id ?? "");
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
    importExperienceLink: async (applicationId, url) => {
      if (hasLocalDatabase) {
        const created = await importInterviewExperienceLink(applicationId, url);
        setExperienceLinks((current) => [created, ...current]);
        return created;
      }
      const id = makeId("link");
      let title = "待分析的网页面经";
      try { title = `${new URL(url).hostname} · 面经帖子`; } catch { /* UI 已校验 URL */ }
      const created: ExperienceLink = { id, applicationId, source: "link", url, title, importedAt: "刚刚", status: "待分析", questions: [] };
      setExperienceLinks((current) => [created, ...current]);
      return created;
    },
    addManualExperience: async (applicationId, title, questions) => {
      if (hasLocalDatabase) {
        const created = await createManualInterviewExperience(applicationId, title, questions);
        setExperienceLinks((current) => [created, ...current]);
        return created;
      }
      const id = makeId("manual");
      const created: ExperienceLink = { id, applicationId, source: "manual", title: title || "人工整理面经", importedAt: "刚刚", status: "已提取", questions };
      setExperienceLinks((current) => [created, ...current]);
      return created;
    },
    analyzeExperienceLink: async (id) => {
      let analyzed: ExperienceLink;
      if (hasLocalDatabase) {
        analyzed = await analyzeInterviewExperienceLink(id);
      } else {
        const existing = experienceLinks.find((item) => item.id === id);
        if (!existing) throw new Error("面经来源不存在");
        analyzed = { ...existing, status: "已提取", questions: experienceQuestions };
      }
      setExperienceLinks((current) => current.map((item) => item.id === id ? analyzed : item));
      return analyzed;
    },
    deleteExperienceSource: async (id) => {
      if (hasLocalDatabase) await deleteInterviewExperienceSource(id);
      setExperienceLinks((current) => current.filter((item) => item.id !== id));
    },
    updateExperienceQuestions: async (id, questions) => {
      const updated = hasLocalDatabase
        ? await updateInterviewExperienceQuestions(id, questions)
        : (() => {
            const existing = experienceLinks.find((item) => item.id === id);
            if (!existing) throw new Error("面经来源不存在");
            return { ...existing, questions };
          })();
      setExperienceLinks((current) => current.map((item) => item.id === id ? updated : item));
      return updated;
    },
    createMockSession: async ({ applicationId, questionCount, useExperience, useAi, useBank, bankQuestions, resumeQuestions: generatedResumeQuestions }) => {
      const id = makeId("mock");
      const imported = experienceLinks.filter((link) => link.applicationId === applicationId && link.status === "已提取").flatMap((link) => link.questions);
      const pool = [
        ...(useExperience ? imported.map((prompt) => ({ prompt, source: "面经" as const })) : []),
        ...(useAi ? (generatedResumeQuestions?.length ? generatedResumeQuestions : resumeQuestions).map((prompt) => ({ prompt, source: "AI 简历题" as const })) : []),
        ...(useBank ? (bankQuestions ?? []).map((prompt) => ({ prompt, source: "个人题库" as const })) : []),
      ];
      if (!pool.length) throw new Error("没有可用于本场模拟的问题");
      const questionInputs = Array.from({ length: questionCount }, (_, index): CreateInterviewQuestion => {
        const selected = requireItem(pool, index % pool.length, "面试题池为空");
        return { prompt: selected.prompt, source: selected.source, answer: "" };
      });
      const created = hasLocalDatabase
        ? await createMockInterviewSession(applicationId, questionInputs)
        : { id, applicationId, type: "模拟面试" as const, round: "技术综合模拟", createdAt: "刚刚", duration: "进行中", status: "进行中" as const, currentQuestionIndex: 0, questions: questionInputs.map((question, index) => ({ ...question, id: `${id}-q${index + 1}`, answer: question.answer || "" })) };
      setSessions((current) => [created, ...current]);
      return created;
    },
    updateSessionAnswer: async (sessionId, questionId, answer) => {
      setSessions((current) => current.map((session) => session.id !== sessionId ? session : {
        ...session,
        questions: session.questions.map((question) => question.id === questionId ? { ...question, answer } : question),
      }));
      if (hasLocalDatabase) {
        const key = `${sessionId}:${questionId}`;
        pendingSessionAnswers.current.set(key, { sessionId, questionId, answer });
        let queued = sessionAnswerQueues.current.get(key);
        if (!queued) {
          queued = (async () => {
            while (true) {
              const pending = pendingSessionAnswers.current.get(key);
              if (!pending) return;
              pendingSessionAnswers.current.delete(key);
              try {
                await updateInterviewSessionAnswer(pending.sessionId, pending.questionId, pending.answer);
                sessionAnswerErrors.current.delete(key);
              } catch (error) {
                sessionAnswerErrors.current.set(key, error);
                // If the user typed again while the failed write was in flight, persist
                // the newest full answer; a successful latest write supersedes the failure.
                if (!pendingSessionAnswers.current.has(key)) throw error;
              }
            }
          })();
          sessionAnswerQueues.current.set(key, queued);
        }
        try { await queued; }
        finally { if (sessionAnswerQueues.current.get(key) === queued) sessionAnswerQueues.current.delete(key); }
      }
    },
    updateSessionProgress: async (id, currentQuestionIndex) => {
      setSessions((current) => current.map((session) => session.id === id ? { ...session, currentQuestionIndex } : session));
      if (hasLocalDatabase) await updateInterviewSessionProgress(id, currentQuestionIndex);
    },
    completeSession: async (id) => {
      const pending = [...sessionAnswerQueues.current.entries()]
        .filter(([key]) => key.startsWith(`${id}:`))
        .map(([, promise]) => promise);
      await Promise.all(pending);
      const failed = [...sessionAnswerErrors.current.entries()]
        .find(([key]) => key.startsWith(`${id}:`));
      if (failed) throw new Error(`仍有答案未保存，请检查数据库后重试：${String(failed[1])}`);
      if (hasLocalDatabase) {
        const completed = await completeInterviewSession(id);
        const lateFailed = [...sessionAnswerErrors.current.entries()]
          .find(([key]) => key.startsWith(`${id}:`));
        for (const key of sessionAnswerErrors.current.keys()) if (key.startsWith(`${id}:`)) sessionAnswerErrors.current.delete(key);
        if (lateFailed) throw new Error(`仍有答案未保存，请检查数据库后重试：${String(lateFailed[1])}`);
        setSessions((current) => current.map((session) => session.id === id ? completed : session));
        return completed;
      }
      const existing = sessions.find((session) => session.id === id);
      if (!existing) throw new Error("面试会话不存在");
      const completed: InterviewSession = {
        ...existing,
        status: "待复盘",
        duration: `${Math.max(10, existing.questions.length * 3)} 分钟`,
        questions: existing.questions.map((question, index) => ({
          ...question,
          score: question.answer.trim() ? 72 + (index % 4) * 4 : 45,
          evaluation: question.answer.trim()
            ? "回答覆盖了主要思路，但还需要补充具体数据、方案取舍和验证结果。"
            : "本题未作答，建议先整理核心概念，再用项目案例形成完整回答。",
        })),
      };
      setSessions((current) => current.map((session) => session.id === id ? completed : session));
      for (const key of sessionAnswerErrors.current.keys()) if (key.startsWith(`${id}:`)) sessionAnswerErrors.current.delete(key);
      return completed;
    },
    reviewSession: async (id, confirmAiSend) => {
      if (!hasLocalDatabase) {
        const existing = sessions.find((item) => item.id === id);
        if (!existing) throw new Error("面试会话不存在");
        const reviewed = { ...existing, status: "复盘完成" as const, reviewSummary: "浏览器演示复盘结果。" };
        setSessions((current) => current.map((item) => item.id === id ? reviewed : item));
        return reviewed;
      }
      const reviewed = await generateInterviewReview(id, confirmAiSend);
      setSessions((current) => current.map((session) => session.id === id ? reviewed : session));
      return reviewed;
    },
    importTranscript: async (applicationId, transcript, confirmAiSend) => {
      if (!hasLocalDatabase) throw new Error("浏览器演示模式不支持导入真实面试材料");
      const imported = await importInterviewTranscript(applicationId, transcript, confirmAiSend);
      setSessions((current) => [imported, ...current]);
      return imported;
    },
    deleteSession: async (id) => {
      if (hasLocalDatabase) await deleteInterviewSession(id);
      for (const key of pendingSessionAnswers.current.keys()) if (key.startsWith(`${id}:`)) pendingSessionAnswers.current.delete(key);
      for (const key of sessionAnswerErrors.current.keys()) if (key.startsWith(`${id}:`)) sessionAnswerErrors.current.delete(key);
      setSessions((current) => current.filter((session) => session.id !== id));
    },
  }), [applications, applicationsError, applicationsLoading, eligibleApplications, experienceLinks, refreshApplications, selectedApplicationId, sessions]);

  return <InterviewFlowContext.Provider value={value}>{children}</InterviewFlowContext.Provider>;
}

export function useInterviewFlow() {
  const value = useContext(InterviewFlowContext);
  if (!value) throw new Error("useInterviewFlow 必须在 InterviewFlowProvider 中使用");
  return value;
}
