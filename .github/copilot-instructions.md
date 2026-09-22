# Copilot Instructions

## Project Overview

Colimail uses SvelteKit/Svelte 5, Tauri 2, Rust, and SQLite through `sqlx`.
Read [ARCHITECTURE.md](../ARCHITECTURE.md) for the source map and data flow,
and [README.md](../README.md) for setup.

- Register Tauri commands in `src-tauri/src/main.rs` and export them from `commands/mod.rs`.
- Keep frontend `invoke` arguments consistent with Rust command signatures.
- Use Svelte 5 runes for reactive state.
- Keep blocking IMAP/SMTP work off the async executor; use the async pool in `db.rs` for SQL.
- Store credentials through `security.rs` (OS keyring); cached-content encryption lives in `encryption.rs`.

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

See also [CLAUDE.md](../CLAUDE.md):
1. **All code and comments MUST be in English**
2. **Assistant responses should be in Chinese**
3. Use latest API versions of dependencies; request documentation if needed
4. Always run static checks after modifications
5. Model structs must have `pub` fields and be serde-compatible

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

## Dependencies

See `package.json` and `src-tauri/Cargo.toml` for the maintained dependency lists.
