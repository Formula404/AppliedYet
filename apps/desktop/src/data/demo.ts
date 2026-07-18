import type { StoredInterviewPreparation, AiCallSummary, PredictedQuestion } from "../services/ai";
import type { ActivitySummary, AnalyticsData, DashboardData } from "../services/dashboard";
import type { EmailStats, RecruitmentEmail } from "../services/emails";
import type { ResumeProfile } from "../services/resumes";
import type { QuestionBankItem } from "../services/interviews";

const iso = (dayOffset: number, hour = 9, minute = 0) => {
  const value = new Date();
  value.setHours(hour, minute, 0, 0);
  value.setDate(value.getDate() + dayOffset);
  return value.toISOString();
};

const monthLabel = (offset: number) => {
  const value = new Date();
  value.setMonth(value.getMonth() + offset);
  return `${value.getMonth() + 1}月`;
};

export const demoDashboard = (): DashboardData => ({
  summary: { total: 26, active: 14, assessments: 4, interviews: 6, waiting: 2, offers: 5, rejected: 9 },
  tasks: [
    { id: "demo-task-1", applicationId: "ant", title: "整理支付链路项目数据", company: "蚂蚁集团", role: "后端开发工程师", dueAt: iso(0, 10, 30), priority: 3, status: "todo", overdue: false, tone: "red" },
    { id: "demo-task-2", applicationId: "meituan", title: "准备 HR 薪资沟通", company: "美团", role: "Java 开发工程师", dueAt: iso(0, 15, 30), priority: 2, status: "doing", overdue: false, tone: "orange" },
    { id: "demo-task-3", applicationId: "shopee", title: "完成在线测评", company: "Shopee", role: "平台开发工程师", dueAt: iso(0, 23, 59), priority: 3, status: "todo", overdue: false, tone: "green" },
    { id: "demo-task-4", applicationId: "tencent", title: "复盘缓存一致性方案", company: "腾讯科技", role: "后台开发工程师", dueAt: iso(0, 9, 0), priority: 1, status: "done", overdue: false, tone: "purple" },
    { id: "demo-task-5", applicationId: "ali", title: "确认薪资方案与入职时间", company: "阿里巴巴", role: "后端开发工程师", dueAt: iso(1, 12, 0), priority: 3, status: "todo", overdue: false, tone: "orange" },
    { id: "demo-task-6", applicationId: "huawei", title: "回复 Offer 意向确认邮件", company: "华为", role: "分布式存储工程师", dueAt: iso(3, 17, 0), priority: 2, status: "todo", overdue: false, tone: "teal" },
  ],
  events: [
    { id: "demo-event-1", applicationId: "ant", title: "技术一面", company: "蚂蚁集团", role: "后端开发工程师", scheduledAt: iso(0, 14, 0), kind: "milestone", tone: "purple" },
    { id: "demo-event-2", applicationId: "meituan", title: "HR 电话沟通", company: "美团", role: "Java 开发工程师", scheduledAt: iso(0, 15, 30), kind: "next_action", tone: "orange" },
    { id: "demo-event-3", applicationId: "shopee", title: "在线测评截止", company: "Shopee", role: "平台开发工程师", scheduledAt: iso(0, 23, 59), kind: "task", tone: "green" },
    { id: "demo-event-4", applicationId: "tencent", title: "技术二面", company: "腾讯科技", role: "后台开发工程师", scheduledAt: iso(1, 14, 0), kind: "milestone", tone: "purple" },
    { id: "demo-event-5", applicationId: "bytedance", title: "二面安排", company: "字节跳动", role: "研发工程师", scheduledAt: iso(2, 10, 0), kind: "next_action", tone: "blue" },
    { id: "demo-event-6", applicationId: "jd", title: "跟进面试结果", company: "京东", role: "数据开发工程师", scheduledAt: iso(-1, 17, 0), kind: "task", tone: "red" },
    { id: "demo-event-7", applicationId: "ant", title: "新增投递", company: "蚂蚁集团", role: "后端开发工程师", scheduledAt: iso(-5, 11, 20), kind: "milestone", tone: "blue" },
    { id: "demo-event-8", applicationId: "meituan", title: "薪资方案确认", company: "美团", role: "Java 开发工程师", scheduledAt: iso(5, 16, 0), kind: "next_action", tone: "teal" },
    { id: "demo-event-9", applicationId: "ali", title: "谈薪中", company: "阿里巴巴", role: "后端开发工程师", scheduledAt: iso(0, 18, 0), kind: "milestone", tone: "orange" },
    { id: "demo-event-10", applicationId: "huawei", title: "Offer 截止", company: "华为", role: "分布式存储工程师", scheduledAt: iso(3, 23, 59), kind: "task", tone: "teal" },
  ],
});

export const demoActivity = (): ActivitySummary => ({
  streakDays: 12, thisWeekApplications: 6, previousWeekApplications: 4,
  dailyActivity: [1, 2, 1, 3, 2, 4, 2, 3, 5, 2, 4, 6, 3, 5],
});

export const demoAnalytics = (): AnalyticsData => ({
  total: 26, thisMonth: 10, previousMonth: 7, assessments: 13, interviews: 9, offers: 5, averageFeedbackDays: 2.6,
  daily: ["一", "二", "三", "四", "五", "六", "日"].map((label, index) => ({ label: `周${label}`, applications: [1, 3, 2, 4, 2, 1, 3][index] ?? 0, interviews: [0, 1, 1, 2, 1, 0, 2][index] ?? 0 })),
  weekly: Array.from({ length: 8 }, (_, index) => ({ label: `${index + 1}周前`, applications: [3, 5, 4, 7, 6, 4, 8, 6][index] ?? 0, interviews: [1, 2, 1, 3, 2, 2, 4, 3][index] ?? 0 })),
  directions: [{ name: "Java / 后端开发", count: 11 }, { name: "平台研发", count: 6 }, { name: "数据开发", count: 4 }, { name: "前端开发", count: 3 }],
});

const initialEmails: RecruitmentEmail[] = [
  { id: "demo-mail-1", sender: "campus@antgroup.com", subject: "蚂蚁集团技术一面安排", receivedAt: iso(0, 8, 42), snippet: "感谢投递后端开发工程师岗位，诚邀你参加技术一面。", bodyText: "你好！你的技术一面安排在今天 14:00，请提前 10 分钟进入会议。面试将重点沟通项目经历与系统设计。", links: [{ label: "进入面试会议", url: "https://meeting.example.com/ant-demo" }], category: "面试邀请", suggestedStage: "面试中", status: "confirmed", matchedApplicationId: "ant", company: "蚂蚁集团", role: "后端开发工程师", currentStage: "面试中", confidence: 98, reasons: ["公司域名一致", "岗位名称一致", "邮件含面试时间"] },
  { id: "demo-mail-2", sender: "recruiting@bytedance.com", subject: "面试通过及后续安排", receivedAt: iso(-1, 18, 16), snippet: "恭喜通过本轮技术面试，后续面试时间将另行通知。", bodyText: "恭喜你通过研发工程师岗位第一轮技术面试。我们将在两个工作日内与你确认下一轮时间。", links: [], category: "结果通知 · 流程进展", suggestedStage: "等待结果", status: "confirmed", matchedApplicationId: "bytedance", company: "字节跳动", role: "研发工程师", currentStage: "等待结果", confidence: 96, reasons: ["公司名称命中", "岗位一致", "通过语义明确"] },
  { id: "demo-mail-3", sender: "talent@meituan.com", subject: "关于 Java 开发岗位的进一步沟通", receivedAt: iso(-2, 11, 8), snippet: "希望与你电话沟通当前求职进展和岗位意向。", bodyText: "你好，我们希望进一步了解你的岗位意向、期望城市与薪资范围，请回复方便沟通的时间。", links: [], category: "HR 沟通", suggestedStage: "等待结果", status: "pending", matchedApplicationId: "meituan", company: "美团", role: "Java 开发工程师", currentStage: "HR 沟通", confidence: 82, reasons: ["公司与岗位匹配", "阶段语义可能为 HR 面或 Offer 沟通，需人工确认"] },
  { id: "demo-mail-4", sender: "no-reply@hackerrank.com", subject: "Online Assessment Reminder - Backend", receivedAt: iso(-2, 9, 35), snippet: "Your assessment invitation will expire tonight.", bodyText: "This is a reminder that your backend online assessment expires at 23:59 today.", links: [{ label: "Start assessment", url: "https://assessment.example.com/demo" }], category: "测评邀请", suggestedStage: "在线测评", status: "confirmed", matchedApplicationId: "shopee", company: "Shopee", role: "平台开发工程师", currentStage: "在线测评", confidence: 91, reasons: ["岗位关键词匹配", "测评链接与截止时间明确"] },
  { id: "demo-mail-5", sender: "星云科技招聘 <hr@startup-example.com>", subject: "后端工程师面试邀请", receivedAt: iso(-3, 16, 20), snippet: "我们在招聘网站看到了你的资料，希望约一次线上沟通。", bodyText: "你好，我们是一家企业服务创业公司，希望邀请你参加后端工程师线上面试。", links: [], category: "面试邀请", suggestedStage: "面试中", status: "unmatched", company: "星云科技", role: "后端工程师", confidence: 43, reasons: ["没有找到对应投递", "公司名称未出现在现有记录"] },
];

let demoEmails = initialEmails.map((item) => ({ ...item }));
export const listDemoEmails = async () => demoEmails.map((item) => ({ ...item, links: [...item.links], reasons: [...item.reasons] }));
export const demoEmailStats = async (): Promise<EmailStats> => ({ thisWeek: demoEmails.length, pending: demoEmails.filter((item) => item.status === "pending").length, confirmed: demoEmails.filter((item) => item.status === "confirmed").length, unmatched: demoEmails.filter((item) => item.status === "unmatched").length });
export const setDemoEmailStatus = async (id: string, status: RecruitmentEmail["status"]) => { demoEmails = demoEmails.map((item) => item.id === id ? { ...item, status } : item); };
export const setDemoEmailApplication = async (id: string, applicationId: string) => {
  demoEmails = demoEmails.map((item) => item.id === id ? {
    ...item,
    matchedApplicationId: applicationId,
    status: "pending",
    confidence: 100,
    reasons: ["用户从该邮件创建并确认关联"],
  } : item);
};

const initialResumes: ResumeProfile[] = [
  { id: "resume-backend", name: "Java 后端 · 2026 秋招", filePath: "演示数据/张同学_后端简历.pdf", fileFormat: "pdf", parsedText: "张同学｜Java 后端开发\n某 985 高校 软件工程 硕士\n两段后端研发实习，负责订单与营销系统。", personalInfo: JSON.stringify({ name: "张同学", birthday: "2002-06-18", contact: "138****2026 · demo@example.com", links: "github.com/demo-candidate" }), educationBackground: JSON.stringify([{ startDate: "2024-09", endDate: "2027-06", school: "浙江大学", degree: "硕士", major: "软件工程" }, { startDate: "2020-09", endDate: "2024-06", school: "南京大学", degree: "本科", major: "计算机科学与技术" }]), internshipExperience: JSON.stringify([{ company: "某头部互联网公司", role: "后端研发实习生", startDate: "2025-06", endDate: "2025-10", description: "参与订单履约系统重构，将核心接口 P99 延迟降低 38%，峰值吞吐提升 2.1 倍。" }]), projectExperience: JSON.stringify([{ name: "高并发秒杀与订单平台", role: "核心开发", startDate: "2025-02", endDate: "2025-05", technologies: "Java / Spring Boot / Redis / Kafka / MySQL", description: "设计缓存、限流与异步下单链路，完成 10 万 QPS 压测与一致性对账。" }]), professionalSkills: "Java、Spring Boot、MySQL、Redis、Kafka、Docker、Kubernetes", academicAchievements: JSON.stringify([{ title: "全国大学生软件创新大赛一等奖", kind: "竞赛", date: "2024-11", link: "", description: "负责服务端架构与性能优化。" }]), skillCertificates: JSON.stringify(["CET-6", "阿里云 ACP"]), targetDirection: "Java 后端 / 平台研发", notes: "突出高并发、稳定性与工程实践", linkedApplicationCount: 7, assessmentCount: 5, interviewCount: 4, offerCount: 1, isPrimary: true, createdAt: iso(-100), updatedAt: iso(-2) },
  { id: "resume-data", name: "数据开发 · 数据平台方向", fileFormat: "docx", parsedText: "面向数据开发岗位的定制版本。", personalInfo: JSON.stringify({ name: "张同学", birthday: "2002-06-18", contact: "138****2026 · demo@example.com", links: "github.com/demo-candidate" }), educationBackground: JSON.stringify([{ startDate: "2024-09", endDate: "2027-06", school: "浙江大学", degree: "硕士", major: "软件工程" }]), internshipExperience: JSON.stringify([{ company: "某云计算公司", role: "数据研发实习生", startDate: "2025-11", endDate: "2026-03", description: "建设实时数据质量监控，告警发现时间由小时级缩短至分钟级。" }]), projectExperience: JSON.stringify([{ name: "实时湖仓分析平台", role: "项目负责人", startDate: "2025-09", endDate: "2026-01", technologies: "Flink / Kafka / ClickHouse", description: "实现实时指标链路与数据血缘追踪。" }]), professionalSkills: "Java、Python、Flink、Spark、Kafka、ClickHouse、Airflow", academicAchievements: "[]", skillCertificates: JSON.stringify(["CET-6"]), targetDirection: "数据开发 / 实时计算", notes: "强调数仓建模与实时计算", linkedApplicationCount: 3, assessmentCount: 2, interviewCount: 1, offerCount: 0, isPrimary: false, createdAt: iso(-60), updatedAt: iso(-6) },
  { id: "resume-archive", name: "通用版 · 历史存档", fileFormat: "pdf", parsedText: "早期通用简历版本。", personalInfo: JSON.stringify({ name: "张同学", birthday: "", contact: "demo@example.com", links: "" }), educationBackground: "[]", internshipExperience: "[]", projectExperience: "[]", professionalSkills: "Java、Python", academicAchievements: "[]", skillCertificates: "[]", targetDirection: "通用研发", notes: "历史版本", linkedApplicationCount: 0, assessmentCount: 0, interviewCount: 0, offerCount: 0, isPrimary: false, archivedAt: iso(-30), createdAt: iso(-180), updatedAt: iso(-30) },
];

let demoResumes = initialResumes.map((item) => ({ ...item }));
const normalizeDemoResumePrimary = () => {
  const primaryId = demoResumes.find((item) => item.isPrimary && !item.archivedAt)?.id
    ?? demoResumes.find((item) => !item.archivedAt)?.id;
  demoResumes = demoResumes.map((item) => ({ ...item, isPrimary: item.id === primaryId }));
};
export const listDemoResumes = async () => demoResumes.map((item) => ({ ...item }));
export const replaceDemoResume = async (id: string, value: Partial<ResumeProfile>) => {
  if (!demoResumes.some((item) => item.id === id)) throw new Error("简历不存在");
  const now = new Date().toISOString();
  demoResumes = demoResumes.map((item) => item.id === id ? { ...item, ...value, updatedAt: now } : item);
  normalizeDemoResumePrimary();
  const updated = demoResumes.find((item) => item.id === id);
  if (!updated) throw new Error("简历不存在");
  return { ...updated };
};
export const createDemoResume = async (name: string, source?: ResumeProfile) => {
  const now = new Date().toISOString();
  const isPrimary = !demoResumes.some((item) => !item.archivedAt);
  const created: ResumeProfile = source
    ? { ...source, id: `resume-${Date.now()}`, name, isPrimary, archivedAt: undefined, linkedApplicationCount: 0, assessmentCount: 0, interviewCount: 0, offerCount: 0, createdAt: now, updatedAt: now }
    : { id: `resume-${Date.now()}`, name, parsedText: "", personalInfo: "{}", educationBackground: "[]", internshipExperience: "[]", projectExperience: "[]", professionalSkills: "", academicAchievements: "[]", skillCertificates: "[]", targetDirection: "", notes: "", linkedApplicationCount: 0, assessmentCount: 0, interviewCount: 0, offerCount: 0, isPrimary, createdAt: now, updatedAt: now };
  demoResumes = [created, ...demoResumes];
  normalizeDemoResumePrimary();
  return { ...created, isPrimary };
};
export const deleteDemoResume = async (id: string) => {
  if (!demoResumes.some((item) => item.id === id)) throw new Error("简历不存在");
  demoResumes = demoResumes.filter((item) => item.id !== id);
  normalizeDemoResumePrimary();
};
export const primaryDemoResume = async (id: string) => {
  const target = demoResumes.find((item) => item.id === id);
  if (!target) throw new Error("简历不存在");
  if (target.archivedAt) throw new Error("归档简历不能设为默认简历");
  demoResumes = demoResumes.map((item) => ({ ...item, isPrimary: item.id === id }));
};

export const demoPreparation = (applicationId: string): StoredInterviewPreparation => ({
  id: `demo-prep-${applicationId}`, applicationId, aiCallId: `demo-call-${applicationId}`, model: "演示模型 · 求职教练", createdAt: iso(-1, 20, 18), sources: [{ type: "job_description" }, { type: "resume_profile" }, { type: "interview_experience" }],
  content: { summary: "岗位重点集中在高并发服务设计、数据一致性和项目深挖。建议用可量化指标串联方案取舍、个人贡献与最终结果。", resumeMatch: { summary: "核心技术栈与岗位匹配度较高，项目成果有量化基础。", strengths: ["Java 与主流中间件经验完整", "有高并发与性能优化结果", "项目中个人职责较清晰"], risks: ["稳定性治理案例还不够具体", "跨团队推进过程缺少冲突与取舍"], evidenceToPrepare: ["压测报告与基线数据", "一次线上故障复盘", "个人负责模块边界"] }, focusAreas: [{ title: "分布式一致性", reason: "JD 多次强调交易链路与可靠消息，需要准备失败重试、幂等和对账闭环。", priority: "high" }, { title: "项目指标深挖", reason: "简历写有 38% 延迟优化，需要解释测量口径和归因。", priority: "high" }, { title: "团队协作", reason: "准备一个通过数据推动技术决策的 STAR 案例。", priority: "medium" }], predictedQuestions: [{ question: "订单链路中如何保证消息不丢、不重并最终一致？", rationale: "岗位职责与项目经历均涉及交易和异步化。", sourceBasis: ["JD：交易链路", "项目：Kafka 异步下单"] }, { question: "P99 延迟降低 38% 的基线、方案和验证方法分别是什么？", rationale: "面试官通常会核验量化成果。", sourceBasis: ["简历量化指标"] }, { question: "讲一次你与其他团队对技术方案意见不一致的经历。", rationale: "验证推动力与协作方式。", sourceBasis: ["岗位能力模型"] }], actionPlan: [{ action: "画出订单系统核心链路与异常分支", estimatedMinutes: 25 }, { action: "整理三项优化前后指标与测量口径", estimatedMinutes: 20 }, { action: "用 STAR 结构录制一次 3 分钟项目介绍", estimatedMinutes: 15 }], sourceNotes: ["演示数据：岗位 JD", "演示数据：关联简历", "演示数据：面经题库"] },
});

export const demoAiCalls = (applicationId: string): AiCallSummary[] => [{ id: `demo-call-${applicationId}`, feature: "interview_preparation", model: "演示模型 · 求职教练", status: "succeeded", attempts: 1, durationMs: 1380, inputSources: [{ type: "job_description" }, { type: "resume_profile" }], createdAt: iso(-1, 20, 18) }];
export const demoResumeQuestions = (count: number): PredictedQuestion[] => {
  const questions = ["请介绍订单系统重构中你个人负责的部分。", "为什么选择 Kafka 异步化，如何处理失败？", "你如何证明 P99 延迟降低 38% 来自这次优化？", "高峰流量下如何保护数据库？", "说一次你推动跨团队方案落地的经历。"];
  return Array.from({ length: count }, (_, index) => ({ question: questions[index % questions.length] ?? "请介绍一个最有代表性的项目。", rationale: "根据关联简历与岗位职责生成", sourceBasis: ["关联简历", "岗位 JD"] }));
};

export const demoMonthLabels = [monthLabel(-2), monthLabel(-1), monthLabel(0)];

const demoBankItems: QuestionBankItem[] = [
  { id: "bank-1", prompt: "介绍一下订单系统重构项目及你个人负责的部分。", category: "项目深挖", bestAnswer: "从原系统耦合问题出发，说明拆分思路、异步化方案以及个人在架构设计与压测中的具体工作。", mastery: "熟悉", source: "AI 简历题", occurrenceCount: 3, lastSeenAt: iso(-1) },
  { id: "bank-2", prompt: "如何处理消息重复消费与最终一致性？", category: "专业知识", bestAnswer: "通过业务唯一键和状态机实现幂等，辅以消费日志和定时对账兜底。", mastery: "掌握", source: "真实面试", occurrenceCount: 5, lastSeenAt: iso(0) },
  { id: "bank-3", prompt: "线上服务延迟突然升高，你会按什么顺序排查？", category: "专业知识", bestAnswer: "先确认变更与发布，再查监控定位资源瓶颈（CPU/内存/IO/网络），然后分析慢查询与 GC 日志。", mastery: "练习中", source: "面经", occurrenceCount: 2, lastSeenAt: iso(-3) },
  { id: "bank-4", prompt: "讲一次你与团队意见不一致的经历。", category: "行为面试", bestAnswer: "使用 STAR 结构，说明分歧双方的诉求、你用数据推动决策的过程以及最终的量化成果。", mastery: "待加强", source: "AI 简历题", occurrenceCount: 1, lastSeenAt: iso(-5) },
  { id: "bank-5", prompt: "如何证明 P99 延迟降低 38% 来自你的优化？", category: "项目深挖", bestAnswer: "明确基线测量口径，用 A/B 对比控制变量，展示优化前后的火焰图或耗时分布。", mastery: "熟悉", source: "AI 简历题", occurrenceCount: 2, lastSeenAt: iso(-2) },
  { id: "bank-6", prompt: "对 JVM 内存模型和常见 GC 问题的理解。", category: "专业知识", bestAnswer: "解释堆分区、GC 算法选择依据，结合一次实际 GC 调优案例说明停顿分析与参数调整。", mastery: "练习中", source: "面经", occurrenceCount: 2, lastSeenAt: iso(-4) },
  { id: "bank-7", prompt: "为什么选择这个岗位和公司？", category: "岗位动机", bestAnswer: "从技术方向、业务前景和个人成长三个角度组织，体现对公司和团队的真实了解。", mastery: "掌握", source: "真实面试", occurrenceCount: 4, lastSeenAt: iso(-1) },
];

export const listDemoQuestionBankItems = async (): Promise<QuestionBankItem[]> => demoBankItems.map((item) => ({ ...item }));
