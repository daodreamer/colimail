# Copilot Instructions

## Project Overview

**Colimail** is a cross-platform desktop email client built with Tauri 2, SvelteKit, and Rust. The goal is to create a lightweight, high-performance alternative to Thunderbird that handles large email volumes without performance degradation.

- **Frontend**: SvelteKit (Svelte 5 with runes) + TypeScript under `src/`
- **Backend**: Rust with Tauri 2 under `src-tauri/src`
- **Database**: SQLite with `rusqlite` for local persistence
- **Email Protocols**: IMAP (via `imap` crate), SMTP (via `lettre`)
- **Performance Targets**: ≤1.5s startup, ≤80MB idle memory, 100 emails in ≤3s

## Backend Architecture (Rust/Tauri)

### Entry Point & Module Structure
- `src-tauri/src/main.rs`: Initializes DB via `db::init()`, registers commands with `tauri::generate_handler!`
- `src-tauri/src/lib.rs`: Contains legacy/example code; actual app logic is in `main.rs`
- `src-tauri/src/db.rs`: Thread-safe global SQLite connection using `lazy_static` + `Mutex<Connection>`; access via `db::connection()`
- `src-tauri/src/models.rs`: Shared structs (`AccountConfig`, `EmailHeader`) with `pub` fields and serde serialization
- `src-tauri/src/commands/`: Command modules organized by domain
  - `accounts.rs`: Account CRUD (`save_account_config`, `load_account_configs`)
  - `emails.rs`: IMAP fetching (`fetch_emails`, `fetch_email_body`)
  - `send.rs`: SMTP sending (`send_email`)
  - `mod.rs`: Re-exports all command functions

### Key Architecture Patterns
- Long-running IMAP/SMTP operations use `tokio::task::spawn_blocking` to avoid blocking async runtime
- Database uses `rusqlite` (synchronous) wrapped in mutex, **NOT** `sqlx` async API
- All blocking I/O (network, database) must be kept off main async executor
- Database schema defined in `db::init()`; extend here when adding tables
- Database stored in platform-specific user data directory (via `directories` crate)

### Adding New Commands
1. Define struct in `models.rs` with `#[derive(Serialize, Deserialize)]` if needed
2. Create function in appropriate `commands/*.rs` file with `#[command]` attribute
3. Export function from `commands/mod.rs`
4. Register in `tauri::generate_handler!` array in `main.rs`
5. Call from frontend using `invoke("command_name", payload)`

## Frontend Architecture (SvelteKit)

### Routes & State
- Routes follow SvelteKit conventions: `src/routes/+page.svelte` (inbox), `src/routes/settings/+page.svelte` (settings)
- State management uses Svelte 5 runes (`$state`, `$derived`, `$effect`)
- Backend commands invoked via `invoke("command_name", { ...payload })`; payload must match Rust struct shapes
- Styling co-located in components; prefer CSS variables for theming

### Build Configuration
- `svelte.config.js`: Uses `@sveltejs/adapter-static` with SPA mode (fallback to `index.html`)
- `vite.config.js`: Dev server on port 1420, HMR on 1421, ignores `src-tauri` directory

## Development Workflows

### Commands
```bash
# Frontend
npm install                    # Install dependencies
npm run dev                    # Run SvelteKit dev server (web-only preview)
npm run tauri dev              # Run full Tauri desktop app in development mode
npm run check                  # Run TypeScript and Svelte type checking
npm run build                  # Build frontend for production

# Backend (run from src-tauri/)
cargo fmt                      # Format Rust code
cargo check                    # Check for compilation errors
cargo clippy -- -D warnings    # Lint with Clippy (treats warnings as errors)
cargo build                    # Build debug version
cargo build --release          # Build optimized release version
```

### Static Checks (MANDATORY)
After any code changes, **always run** appropriate checks before reporting completion:
- **Rust changes**: `cargo fmt && cargo check && cargo clippy -- -D warnings` from `src-tauri/`
- **Frontend changes**: `npm run check` from root
- **IMPORTANT**: `cargo clippy -- -D warnings` treats all warnings as errors to maintain code quality

## Language and Code Conventions

Per `copilot_rules.md` and `CLAUDE.md`:
1. **All code and comments MUST be in English**
2. **Assistant responses should be in Chinese**
3. Use latest API versions of dependencies; request documentation if needed
4. Always run static checks after modifications
5. Model structs must have `pub` fields and be serde-compatible
6. Passwords currently stored plaintext (known limitation for future improvement)

## Execution Rules (MUST FOLLOW)

1. **Evaluate Tool Results Before Proceeding**: After receiving tool results, carefully assess their quality and determine the best next action. Use reasoning to plan and iterate based on new information.

2. **Clean Up Temporary Files**: Delete any temporary files, scripts, or helper files created during iteration at task completion.

3. **Implement General Solutions Using Standard Tools**: 
   - Write high-quality, general solutions using standard tools
   - Do NOT create helper scripts or workarounds for efficiency
   - Implement solutions that work for all valid inputs, not just test cases
   - Do NOT hard-code values or create test-specific solutions
   - Focus on understanding requirements and implementing correct algorithms
   - Provide canonical implementations following best practices
   - If task is unreasonable or infeasible, inform user directly

4. **Never Speculate About Unread Code**: 
   - Never make assumptions about code you haven't opened
   - If user mentions a specific file, read it before responding
   - Always investigate and read relevant files before answering
   - Never make assertions about code before investigating
   - Provide well-reasoned, hallucination-free responses

## Key Dependencies

- `tauri` v2: Desktop application framework
- `tokio`: Async runtime (with "full" features)
- `imap`: IMAP protocol client
- `lettre`: SMTP email sending (with tokio runtime and native-tls)
- `rusqlite`: Synchronous SQLite bindings (NOT using `sqlx`)
- `serde` / `serde_json`: Serialization for command payloads
- `mailparse`: Email message parsing
- `directories`: Platform-specific directory paths
- `lazy_static`: Global static database connection
