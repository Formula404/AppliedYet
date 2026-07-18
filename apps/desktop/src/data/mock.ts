import type { Application, MailItem, TaskItemData } from "../types";

export const columnStages: Record<string, string> = {
  "已投递": "已投递",
  "测评": "在线测评",
  "笔试": "笔试",
  "面试": "面试中",
  "等待结果": "等待结果",
  "Offer": "已获Offer",
  "进入人才库": "进入人才库",
  "已拒绝": "已拒绝",
};

export const stageToneMap: Record<string, string> = {
  "已投递": "blue",
  "在线测评": "orange",
  "笔试": "orange",
  "面试中": "purple",
  "等待结果": "gray",
  "已获Offer": "teal",
  "进入人才库": "gray",
  "已拒绝": "red",
};

export const applications: Application[] = [
  { id: "ant", company: "蚂蚁集团", companyMark: "蚁", role: "后端开发工程师", city: "杭州", stage: "一面安排", stageTone: "blue", priority: "高", nextStep: "技术一面", nextTime: "今天 09:30", progress: 3, updated: "12 分钟前", risk: "面试准备 70%", resumeProfileId: "resume-backend", resumeName: "Java 后端 · 2026 秋招" },
  { id: "tencent", company: "腾讯科技", companyMark: "腾", role: "后台开发工程师", city: "深圳", stage: "二面准备中", stageTone: "purple", priority: "高", nextStep: "完成项目复盘", nextTime: "明天 14:00", progress: 4, updated: "昨天", resumeProfileId: "resume-backend", resumeName: "Java 后端 · 2026 秋招" },
  { id: "meituan", company: "美团", companyMark: "美", role: "Java 开发工程师", city: "北京", stage: "HR 沟通", stageTone: "orange", priority: "中", nextStep: "薪资沟通", nextTime: "今天 15:30", progress: 4, updated: "2 小时前", resumeProfileId: "resume-backend", resumeName: "Java 后端 · 2026 秋招" },
  { id: "bytedance", company: "字节跳动", companyMark: "字", role: "研发工程师", city: "北京", stage: "等待结果", stageTone: "gray", priority: "普通", nextStep: "等待二面安排", nextTime: "待安排", progress: 4, updated: "5 月 19 日", resumeProfileId: "resume-backend", resumeName: "Java 后端 · 2026 秋招" },
  { id: "shopee", company: "Shopee", companyMark: "S", role: "平台开发工程师", city: "新加坡", stage: "在线测评", stageTone: "green", priority: "高", nextStep: "完成在线测评", nextTime: "今天 23:59", progress: 2, updated: "今天 08:40", risk: "即将截止", resumeProfileId: "resume-backend", resumeName: "Java 后端 · 2026 秋招" },
  { id: "jd", company: "京东", companyMark: "京", role: "数据开发工程师", city: "北京", stage: "等待结果", stageTone: "gray", priority: "普通", nextStep: "等待面试结果", nextTime: "已等待 3 天", progress: 4, updated: "3 天前", risk: "长时间未更新", resumeProfileId: "resume-data", resumeName: "数据开发 · 数据平台方向" },
  { id: "rej1", company: "快手", companyMark: "快", role: "后端开发工程师", city: "北京", stage: "已拒绝", stageTone: "red", priority: "高", nextStep: "已结束", nextTime: "5 月 15 日", progress: 5, updated: "5 月 15 日", risk: "简历未通过筛选" },
  { id: "rej2", company: "网易", companyMark: "网", role: "Java 开发工程师", city: "杭州", stage: "已拒绝", stageTone: "red", priority: "高", nextStep: "已结束", nextTime: "5 月 12 日", progress: 5, updated: "5 月 12 日", risk: "二面未通过" },
  { id: "rej3", company: "拼多多", companyMark: "拼", role: "研发工程师", city: "上海", stage: "已拒绝", stageTone: "red", priority: "中", nextStep: "已结束", nextTime: "5 月 8 日", progress: 5, updated: "5 月 8 日", risk: "测评未达标" },
  { id: "rej4", company: "小红书", companyMark: "红", role: "前端开发工程师", city: "上海", stage: "已拒绝", stageTone: "red", priority: "中", nextStep: "已结束", nextTime: "4 月 28 日", progress: 5, updated: "4 月 28 日", risk: "岗位已招满" },
  { id: "rej5", company: "携程", companyMark: "携", role: "平台开发工程师", city: "上海", stage: "已拒绝", stageTone: "red", priority: "普通", nextStep: "已结束", nextTime: "4 月 20 日", progress: 5, updated: "4 月 20 日" },
  { id: "rej6", company: "哔哩哔哩", companyMark: "B", role: "后端开发工程师", city: "上海", stage: "已拒绝", stageTone: "red", priority: "普通", nextStep: "已结束", nextTime: "4 月 15 日", progress: 5, updated: "4 月 15 日" },
  { id: "ali", company: "阿里巴巴", companyMark: "阿", role: "后端开发工程师", city: "杭州", stage: "谈薪中", stageTone: "orange", priority: "高", nextStep: "等待薪资方案", nextTime: "今天 18:00", progress: 5, updated: "1 小时前", resumeProfileId: "resume-backend", resumeName: "Java 后端 · 2026 秋招" },
  { id: "huawei", company: "华为", companyMark: "华", role: "分布式存储工程师", city: "深圳", stage: "已获Offer", stageTone: "teal", priority: "中", nextStep: "确认入职意向", nextTime: "3 天后截止", progress: 5, updated: "昨天", resumeProfileId: "resume-backend", resumeName: "Java 后端 · 2026 秋招" },
];

export const mails: MailItem[] = [
  { id: "m1", company: "蚂蚁集团", role: "后端开发工程师", subject: "面试安排通知", summary: "您好，感谢您投递蚂蚁集团，邀请您参加技术一面…", type: "面试安排", time: "今天 09:15", confidence: 98, status: "已处理" },
  { id: "m2", company: "字节跳动", role: "研发工程师", subject: "面试结果及后续安排", summary: "您的简历已通过本轮面试，后续面试时间将另行通知…", type: "面试通过", time: "昨天", confidence: 94, status: "已自动匹配" },
  { id: "m3", company: "美团", role: "Java 开发工程师", subject: "沟通邀请", summary: "想和您沟通一下目前的求职进展和岗位匹配情况…", type: "HR 沟通", time: "5 月 18 日", confidence: 76, status: "待确认" },
  { id: "m4", company: "Shopee", role: "平台开发工程师", subject: "Online Assessment Reminder", summary: "This is a reminder that your online assessment will expire soon…", type: "测评提醒", time: "5 月 18 日", confidence: 91, status: "已处理" },
];

export const todayTasks: TaskItemData[] = [
  { id: "t1", title: "技术一面", relation: "蚂蚁集团 · 后端开发", time: "09:30", tone: "blue" },
  { id: "t2", title: "完成 LeetCode 每日一题", relation: "个人训练", time: "10:30", tone: "orange" },
  { id: "t3", title: "复盘：快手二面", relation: "快手 · 后端开发", time: "14:00", tone: "purple" },
  { id: "t4", title: "HR 电话沟通", relation: "美团 · Java 开发", time: "15:30", tone: "orange" },
  { id: "t5", title: "在线测评截止", relation: "Shopee · 平台开发", time: "23:59", tone: "green" },
];

export const calendarEvents: Record<number, { label: string; tone: string }[]> = {
  1: [{ label: "测评截止 · 网易", tone: "green" }],
  2: [{ label: "一面 · 字节跳动", tone: "purple" }],
  4: [{ label: "复盘任务 3", tone: "purple" }],
  6: [{ label: "二面 · 腾讯", tone: "blue" }],
  7: [{ label: "HR沟通 · 美团", tone: "orange" }],
  9: [{ label: "一面 · 阿里", tone: "blue" }],
  13: [{ label: "测评截止 · Shopee", tone: "green" }],
  15: [{ label: "二面 · 拼多多", tone: "blue" }],
  16: [{ label: "复盘任务 2", tone: "purple" }],
  18: [{ label: "HR沟通 · 京东", tone: "orange" }],
  20: [{ label: "一面 · 蚂蚁集团", tone: "blue" }, { label: "复盘任务 1", tone: "purple" }],
  22: [{ label: "Offer截止 · 网易", tone: "green" }],
  27: [{ label: "二面 · 快手", tone: "blue" }],
  29: [{ label: "HR沟通 · 携程", tone: "orange" }],
};
