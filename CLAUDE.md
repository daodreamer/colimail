# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**MailDesk** is a cross-platform desktop email client built with Tauri 2, SvelteKit, and Rust. The goal is to create a lightweight, high-performance alternative to Thunderbird that handles large email volumes without performance degradation.

- **Frontend**: SvelteKit (Svelte 5 with runes) + TypeScript
- **Backend**: Rust with Tauri 2
- **Database**: SQLite for local persistence
- **Email protocols**: IMAP (via `imap` crate), SMTP (via `lettre`)

## Development Commands

### Frontend Development
```bash
npm install                    # Install dependencies
npm run dev                    # Run SvelteKit dev server (web-only preview)
npm run tauri dev              # Run full Tauri desktop app in development mode
npm run check                  # Run TypeScript and Svelte type checking
npm run build                  # Build frontend for production
```

### Backend Development
```bash
cd src-tauri
cargo fmt                      # Format Rust code
cargo check                    # Check for compilation errors
cargo build                    # Build Rust backend
cargo build --release          # Build optimized release version
```

### Running Static Checks
After any code changes, **always run** the appropriate checks before reporting completion:
- Rust changes: `cargo fmt && cargo check` from `src-tauri/`
- Frontend changes: `npm run check` from root

## Architecture

### Backend Structure (Rust/Tauri)

**Entry Point**: `src-tauri/src/main.rs`
- Initializes SQLite database via `db::init()`
- Registers Tauri commands using `tauri::generate_handler!`
- Note: `src-tauri/src/lib.rs` contains legacy/example code; actual app logic is in `main.rs`

**Database Layer**: `src-tauri/src/db.rs`
- Uses `lazy_static` + `Mutex<Connection>` for thread-safe global SQLite connection
- Access via `db::connection()` helper function
- Schema initialization in `db::init()` - extend here when adding new tables
- Database stored in platform-specific user data directory (via `directories` crate)

**Commands Layer**: `src-tauri/src/commands/`
- `accounts.rs`: Account CRUD operations (`save_account_config`, `load_account_configs`)
- `emails.rs`: IMAP email fetching (`fetch_emails`, `fetch_email_body`)
- `send.rs`: SMTP email sending (`send_email`)
- `mod.rs`: Re-exports all command functions

**Data Models**: `src-tauri/src/models.rs`
- Shared structs between Rust backend and TypeScript frontend
- All fields must be `pub` and serde-compatible for JSON serialization
- `AccountConfig`: Email account credentials and server settings
- `EmailHeader`: Basic email metadata (uid, subject, from, date)

**Key Architecture Patterns**:
- Long-running IMAP/SMTP operations use `tokio::task::spawn_blocking` to avoid blocking the async runtime
- All blocking I/O (network, database) should be kept off the main async executor
- Database uses `rusqlite` (synchronous) wrapped in mutex, not `sqlx` async API

### Frontend Structure (SvelteKit)

**Routes**: `src/routes/`
- `+page.svelte`: Main inbox UI
- `settings/+page.svelte`: Account settings form
- Uses SvelteKit file-based routing conventions

**State Management**:
- Svelte 5 runes (`$state`, `$derived`, `$effect`)
- Keep reactive state consistent with runes API

**Backend Communication**:
- Use Tauri's `invoke()` function: `invoke("command_name", { ...payload })`
- Payload structure must match Rust command function parameters
- Return types are serialized from Rust `Result<T, String>` to TypeScript `Promise<T>`

**Build Configuration**:
- `svelte.config.js`: Uses `@sveltejs/adapter-static` with SPA mode (fallback to `index.html`)
- `vite.config.js`: Dev server on port 1420, HMR on 1421, ignores `src-tauri` directory

### Adding New Commands

1. Define struct in `src-tauri/src/models.rs` if needed (with `#[derive(Serialize, Deserialize)]`)
2. Create function in appropriate `src-tauri/src/commands/*.rs` file with `#[command]` attribute
3. Export function from `src-tauri/src/commands/mod.rs`
4. Register in `tauri::generate_handler!` array in `src-tauri/src/main.rs`
5. Call from frontend using `invoke("command_name", payload)`

### Database Schema

Current tables (defined in `db::init()`):
- `accounts`: Email account credentials (id, email, password, imap_server, imap_port, smtp_server, smtp_port)

**Note**: Passwords are currently stored in plaintext - a known limitation for future improvement.

## Language and Code Style Conventions

Per `copilot_rules.md`:
1. **All code and comments must be in English**
2. **Assistant responses should be in Chinese** (relevant for AI assistants configured for this project)
3. Use latest API versions of dependencies; request documentation if needed
4. Always run static checks after modifications

## Performance Targets

From `development_requirements.md`, the project aims for:
- Startup time: ≤ 1.5 seconds (cold start)
- Memory footprint: ≤ 80 MB (idle)
- CPU usage: ≤ 5% (idle)
- Email sync: 100 emails in ≤ 3 seconds
- Local query latency: ≤ 50 ms
- Package size: ≤ 30 MB (Windows executable)

## Crate Dependencies (Key)

- `tauri` v2: Desktop application framework
- `tokio`: Async runtime (used with "full" features)
- `imap`: IMAP protocol client
- `lettre`: SMTP email sending (with tokio runtime and native-tls)
- `rusqlite`: Synchronous SQLite bindings (note: NOT using `sqlx` for DB operations)
- `serde` / `serde_json`: Serialization for command payloads
- `mailparse`: Email message parsing
- `directories`: Platform-specific directory paths
- `lazy_static`: Global static database connection

## Execution Rules for Claude Code

These rules **must be followed** during every execution session:

1. **Evaluate Tool Results Before Proceeding**: After receiving tool results, carefully assess their quality and determine the best next action before continuing. Use your reasoning capabilities to plan and iterate based on new information, then take the optimal next step.

2. **Clean Up Temporary Files**: If any temporary files, scripts, or helper files are created during the iteration process, delete them at the end of the task to complete cleanup work.

3. **Implement General Solutions Using Standard Tools**: Use standard available tools to write high-quality, general solutions. Do not create helper scripts or workarounds to complete tasks more efficiently. Implement solutions that work correctly for all valid inputs, not just test cases. Do not hard-code values or create solutions that only work for specific test inputs. Instead, implement general logic that fundamentally solves the problem.
   - Focus on understanding problem requirements and implementing correct algorithms. Tests serve to verify correctness, not to define the solution itself. Provide canonical implementations that follow best practices and software design principles.
   - If the task itself is unreasonable or infeasible, or if any test cases are incorrect, inform the user directly rather than trying to work around them. The final solution should be robust, maintainable, and extensible.

4. **Never Speculate About Unread Code**: Never make assumptions about code you haven't opened. If the user mentions a specific file, you must read that file before responding. Always investigate and read relevant files before answering questions about the codebase. Unless you are highly confident of the correct answer, never make assertions about code before investigating—provide well-reasoned, hallucination-free responses.
