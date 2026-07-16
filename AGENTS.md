# Repository Guidelines

## Project Structure & Module Organization

The product lives in `apps/desktop/`. Its React/TypeScript frontend is under `src/`: routes in `app/`, UI in `components/`, views in `pages/`, state in `hooks/`, Tauri/browser adapters in `services/`, and models in `types/`. Keep demo fixtures in `data/` isolated from persisted data. Global styles are in `src/styles/index.css`; static assets belong in `public/`.

The Rust/Tauri backend is in `src-tauri/`. Put thin IPC handlers in `src/commands/`, domain logic in `src/database/`, and numbered SQLite migrations in `migrations/`. Frontend code must access desktop capabilities through services and commands, never the database or credential store directly. Documentation assets are under `docs/`.

## Build, Test, and Development Commands

- `npm install`: install dependencies (Node.js 20+, npm 10+).
- `npm run dev`: start the Vite browser demo without native integrations.
- `npm run tauri -- dev`: run the Windows desktop app with native integrations.
- `npm run typecheck`: run strict TypeScript validation.
- `npm run check:rust`: check Rust formatting, run Clippy with warnings denied, and execute Rust tests.
- `npm run check`: run all pre-PR validation.
- `npm run tauri:build`: create the Windows executable and NSIS installer.

## Coding Style & Naming Conventions

Use two-space indentation, double quotes, and semicolons in TypeScript; use standard `rustfmt` output in Rust. Name React components/files with `PascalCase`, functions and variables with `camelCase`, hooks as `useXxx`, and Rust modules/functions with `snake_case`. Strict TypeScript checks reject unused code, implicit returns, fallthrough, and unchecked indexing. Keep command handlers narrow.

## Testing Guidelines

Rust unit tests live beside implementation code in `#[cfg(test)]` modules and use descriptive `snake_case` names. Test migrations, transactions, parsing, validation, and failure paths. No frontend test framework or coverage threshold is configured; typecheck and build every UI change, then manually verify affected browser and Tauri flows. Run `npm run check` before submission.

## Commit & Pull Request Guidelines

History uses Conventional Commit prefixes such as `feat:`, `fix:`, and `chore:` with concise summaries. Keep commits focused and include migrations with consuming code. Pull requests should explain the change, validation, and linked issues. Include screenshots for UI work and note schema, credential, provider, or packaging impact.

## Security & Data Changes

Never commit `.env` files, API keys, mailbox credentials, OAuth tokens, databases, or generated build artifacts. Preserve transaction boundaries and integrity checks for database moves, backups, restores, and migrations. Treat changes to CSP, Tauri capabilities, external URLs, AI/ASR data sharing, and network input limits as security-sensitive.
