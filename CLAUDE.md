# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Colimail** is a cross-platform desktop email client built with Tauri 2, SvelteKit, and Rust. The goal is to create a lightweight, high-performance alternative to Thunderbird that handles large email volumes without performance degradation.

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
- Rust changes: `cargo fmt && cargo check && cargo clippy -- -D warnings` from `src-tauri/`
- Frontend changes: `npm run check` from root

**IMPORTANT**: The `cargo clippy -- -D warnings` command treats all Clippy warnings as errors. This ensures code quality and prevents warnings from accumulating. Always fix any Clippy warnings before completing a task.

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for the source map and data flow.
The desktop entry point is `src-tauri/src/main.rs`; database access uses the
async `sqlx::SqlitePool` in `db.rs`. Credentials are stored through `security.rs`
using the OS keyring. Cached-content encryption is handled by `encryption.rs`.

When adding a Tauri command, export it from `commands/mod.rs`, register it in
`main.rs`, and keep frontend `invoke` arguments consistent with its Rust signature.
Use Svelte 5 runes for reactive state and keep blocking IMAP/SMTP work off the
async executor.

## Language and Code Style Conventions

Project conventions:
1. **All code and comments must be in English**
2. **Assistant responses should be in Chinese** (relevant for AI assistants configured for this project)
3. Use latest API versions of dependencies; request documentation if needed
4. Always run static checks after modifications

## Execution Rules for Claude Code

These rules **must be followed** during every execution session:

1. **Evaluate Tool Results Before Proceeding**: After receiving tool results, carefully assess their quality and determine the best next action before continuing. Use your reasoning capabilities to plan and iterate based on new information, then take the optimal next step.

2. **Clean Up Temporary Files**: If any temporary files, scripts, or helper files are created during the iteration process, delete them at the end of the task to complete cleanup work.

3. **Implement General Solutions Using Standard Tools**: Use standard available tools to write high-quality, general solutions. Do not create helper scripts or workarounds to complete tasks more efficiently. Implement solutions that work correctly for all valid inputs, not just test cases. Do not hard-code values or create solutions that only work for specific test inputs. Instead, implement general logic that fundamentally solves the problem.
   - Focus on understanding problem requirements and implementing correct algorithms. Tests serve to verify correctness, not to define the solution itself. Provide canonical implementations that follow best practices and software design principles.
   - If the task itself is unreasonable or infeasible, or if any test cases are incorrect, inform the user directly rather than trying to work around them. The final solution should be robust, maintainable, and extensible.

4. **Never Speculate About Unread Code**: Never make assumptions about code you haven't opened. If the user mentions a specific file, you must read that file before responding. Always investigate and read relevant files before answering questions about the codebase. Unless you are highly confident of the correct answer, never make assertions about code before investigating—provide well-reasoned, hallucination-free responses.


You are able to use the Svelte MCP server, where you have access to comprehensive Svelte 5 and SvelteKit documentation. Here's how to use the available tools effectively:

## Available MCP Tools:

### 1. list-sections

Use this FIRST to discover all available documentation sections. Returns a structured list with titles, use_cases, and paths.
When asked about Svelte or SvelteKit topics, ALWAYS use this tool at the start of the chat to find relevant sections.

### 2. get-documentation

Retrieves full documentation content for specific sections. Accepts single or multiple sections.
After calling the list-sections tool, you MUST analyze the returned documentation sections (especially the use_cases field) and then use the get-documentation tool to fetch ALL documentation sections that are relevant for the user's task.

### 3. svelte-autofixer

Analyzes Svelte code and returns issues and suggestions.
You MUST use this tool whenever writing Svelte code before sending it to the user. Keep calling it until no issues or suggestions are returned.

### 4. playground-link

Generates a Svelte Playground link with the provided code.
After completing the code, ask the user if they want a playground link. Only call this tool after user confirmation and NEVER if code was written to files in their project.