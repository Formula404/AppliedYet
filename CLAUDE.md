# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Install dependencies (cd apps/desktop first)
cd apps/desktop && npm install

# Dev server (browser at localhost:1421)
npm run dev              # from root — proxies to apps/desktop

# Tauri desktop mode
npm run tauri dev        # from root

# Type check
npm run typecheck        # from root

# Production build
npm run build            # from root

# Tauri-specific
npm run tauri build      # from root — produces installer
```

## Project Architecture

Monorepo at `apps/desktop/`. Single app — no other apps yet.

### Stack
- **Desktop shell**: Tauri 2 / Rust (`apps/desktop/src-tauri/`)
- **Frontend**: React 18, TypeScript, Vite, React Router
- **Icons**: Lucide React
- **Styling**: Native CSS Design Tokens (no Tailwind, no UI framework)
- **State**: Currently mock data only; no backend/persistence yet

### Directory Layout (`apps/desktop/src/`)

```
app/App.tsx              # Router configuration (10 routes)
components/
  AppShell.tsx           # Main layout: sidebar, topbar, search, title bar
  TitleBar.tsx           # Custom window title bar (decorations: false)
  ui.tsx                 # Primitives: Badge, Card, CardHeader, PageHeader
pages/
  HomePage.tsx           # Dashboard: calendar, KPI, pipeline, tasks
  ApplicationsPage.tsx   # Kanban/list view for job applications
  EmailsPage.tsx         # Recruitment email identification & matching
  FeaturePages.tsx       # All other pages selected by `kind` prop
data/mock.ts             # Demo data (Application[], MailItem[], tasks, calendar)
types/index.ts           # Shared: Application, MailItem, TaskItemData, StatusTone
styles/index.css         # Design system (~2500 lines) with light/dark themes
hooks/useTheme.tsx       # Theme context (light / dark / system)
```

### Tauri Config (`src-tauri/`)
- `decorations: false` — custom title bar (`TitleBar.tsx`)
- Window: 1440×900 default, 1024×720 min, resizable, centered
- Capabilities in `capabilities/default.json` — each window API call needs explicit permission

### Routes
| Path | Page | Component |
|------|------|-----------|
| `/` | Home (dashboard) | `HomePage` |
| `/applications` | Applications | `ApplicationsPage` |
| `/emails` | Emails | `EmailsPage` |
| `/preparation` | Interview Prep | `FeaturePages kind="preparation"` |
| `/mock-interview` | Mock Interview | `FeaturePages kind="mock"` |
| `/reviews` | Review | `FeaturePages kind="reviews"` |
| `/question-bank` | Question Bank | `FeaturePages kind="questions"` |
| `/offers` | Offers | `FeaturePages kind="offers"` |
| `/analytics` | Analytics | `FeaturePages kind="analytics"` |
| `/settings` | Settings | `FeaturePages kind="settings"` |

### Design System (CSS Tokens)
- All colors, spacing, typography via CSS custom properties in `:root` / `[data-theme="dark"]`
- Semantic color palette: blue (primary), green (success), orange (warning), purple, teal, red (danger), gray (neutral)
- `StatusTone` type governs badge/card tones across the app
- Reference: `docs/投了吗_UI设计规范.md`

### Window API Permissions (Tauri v2)
Window operations require explicit `core:window:allow-*` capabilities. Current permissions: `close`, `minimize`, `start-dragging`, `set-size`, `set-position`, `outer-size`, `outer-position`, `current-monitor`. Adding new window API calls requires updating `src-tauri/capabilities/default.json`.
