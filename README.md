# 投了吗（Applied Yet?）

“投了吗”是一款面向 Windows 的本地优先求职流程工作台。投递、任务、流程事件、简历、招聘邮件索引、面试记录和设置保存在本机 SQLite；AI、ASR 与邮箱连接均为可选能力。

## 已实现能力

- 投递看板、列表、筛选、归档、软删除、阶段历史和可撤销事件；
- 日历、待办提醒、流程 KPI、趋势分析和真实 Offer/谈薪视图；
- 多版本结构化简历、投递绑定、文档解析和 Excel 导出；
- IMAP 增量同步、招聘邮件识别、投递匹配和人工确认后更新阶段；
- 面经链接/人工题目、模拟面试、断点续答、真实材料导入、AI 复盘和个人题库；
- OpenAI Responses、OpenAI 兼容 Chat Completions 与 Anthropic 协议；
- 可配置 ASR、Windows 系统通知和 Windows 凭据管理器；
- 数据目录迁移、SQLite 完整性检查、手动备份与安全恢复；
- 浏览器隔离演示模式：不连接真实 SQLite、系统凭据或邮箱。

## 架构

```text
apps/desktop/
├── src/
│   ├── app/          # 路由
│   ├── components/   # 桌面外壳、错误边界、通用 UI
│   ├── hooks/        # 跨页面业务状态与主题
│   ├── pages/        # 投递、邮件、面试、简历、设置等页面
│   ├── services/     # Tauri IPC / 浏览器演示适配层
│   ├── data/         # 与真实数据隔离的演示数据
│   └── types/        # 前端共享类型
└── src-tauri/
    ├── migrations/   # 顺序化 SQLite 迁移
    └── src/
        ├── commands/ # 窄 Tauri 命令层
        ├── database/ # 数据模型、事务和领域规则
        ├── ai.rs / asr.rs / http.rs
        └── document.rs / experience.rs / resume*.rs
```

前端不直接读写数据库或凭据；所有桌面能力经过 `services` → Tauri command → 领域模块。SQLite 写操作使用事务，数据库迁移、移动、备份和恢复均执行完整性检查。API Key、邮箱密码和 OAuth refresh token 只保存在 Windows 凭据管理器，不进入 SQLite 备份。

## 开发环境

- Node.js 20+ 与 npm 10+；
- Rust stable（MSVC toolchain）；
- Tauri 2 的 Windows 构建依赖，包括 Microsoft C++ Build Tools 与 WebView2。

安装依赖：

```powershell
npm install
```

浏览器演示模式：

```powershell
npm run dev
```

Windows 桌面开发模式：

```powershell
npm run tauri -- dev
```

## 质量检查

一次运行严格 TypeScript、前端生产构建、Rust 格式、Clippy（警告视为错误）和全部测试：

```powershell
npm run check
```

也可以分别运行：

```powershell
npm run typecheck
npm run build
npm run check:rust
```

TypeScript 已启用未使用代码、隐式返回、switch fallthrough 和未检查索引等严格检查。

## Windows 构建

项目使用 NSIS 作为当前 Windows 安装包格式：

```powershell
npm run tauri:build
```

产物位于：

```text
apps/desktop/src-tauri/target/release/tou-le-ma.exe
apps/desktop/src-tauri/target/release/bundle/nsis/投了吗_<版本>_x64-setup.exe
```

发布前应根据发行主体配置 Windows 代码签名；未配置签名不影响本地构建，但会影响 SmartScreen 信任提示。

## 隐私与安全

- 外部 Provider 默认不获准接收简历、岗位或面试转写；
- 可要求每次发送敏感数据前再次确认；后端也会重复校验授权，不能只绕过前端确认；
- 远程 IMAP 必须使用 TLS，OAuth 使用 PKCE 与本机回调；
- AI、ASR、OAuth 和文档输入均有限流或大小上限；
- 面经抓取拒绝本机、内网和非 HTTP(S) 地址；
- Tauri 使用最小窗口权限、生产 CSP 和冻结原型配置。

当前 `imap 2.4.x` 的上游 `imap-proto 0.10.x` 会产生 Rust future-incompatibility 提示，但当前 stable 构建和测试正常。升级到新的 IMAP 实现前需完整验证网易 RFC 2971 ID、OAuth2 和各邮箱服务商兼容性。

## 文档

- [产品文档](docs/投了吗_产品文档.md)
- [技术文档](docs/投了吗_技术文档.md)
- [UI 设计规范](docs/投了吗_UI设计规范.md)
