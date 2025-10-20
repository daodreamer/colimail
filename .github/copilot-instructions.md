# Copilot Instructions

## Project Overview
- This repo builds a desktop mail client using Tauri 2 with a SvelteKit frontend and a Rust backend.
- Frontend lives under `src/`, using Svelte 5 runes ($state) and communicates with backend via `@tauri-apps/api`.
- Backend lives under `src-tauri/src`, exposing commands for accounts, email fetching, and sending; SQLite used for persistence.
- Reference `copilot_rules.md` for collaboration norms (Chinese replies, English code/comments, run static checks).

## Backend (Rust/Tauri)
- Entry point `src-tauri/src/main.rs` registers commands from `src-tauri/src/commands`.
- Database access handled through `db.rs` with a global `rusqlite` connection stored behind `lazy_static` `Mutex`; reuse via `connection()`.
- Commands split into modules: `accounts.rs` for CRUD, `emails.rs` for IMAP fetch, `send.rs` for SMTP using `lettre`.
- Long-running IMAP tasks run inside `tokio::task::spawn_blocking`; keep blocking I/O off the async runtime.
- When adding a new command, export from `commands/mod.rs` and register it in `tauri::generate_handler!`.

## Frontend (SvelteKit)
- Routes follow SvelteKit conventions: inbox UI in `src/routes/+page.svelte`, settings form in `src/routes/settings/+page.svelte`.
- State uses Svelte 5 `$state` runes; keep new stores consistent for reactivity.
- Backend commands invoked via `invoke("command_name", { ... })`; ensure payload shapes mirror Rust structs in `models.rs`.
- Styling co-located within Svelte components; prefer CSS variables defined in the component for theming.

## Workflows
- Install JS deps with `npm install` (project uses npm scripts).
- For desktop dev, run `npm run tauri dev`; for web-only preview use `npm run dev`.
- After Rust changes: `cd src-tauri`, run `cargo fmt` and `cargo check`.
- After frontend changes run `npm run check` for type/sanity validation.
- Static analysis is mandatory post-edit before reporting results.

## Conventions & Tips
- Model structs shared with frontend live in `models.rs`; keep fields `pub` and serde-friendly.
- SQLite schema managed in `db::init`; extend here when adding tables.
- Avoid blocking network or database calls on the main thread; prefer `spawn_blocking`.
- Sensitive data (passwords) currently stored plaintext; note TODOs if touching auth logic.
- Keep all new code/comments in English per `copilot_rules.md`, but summarize behavior in Chinese responses.
