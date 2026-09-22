# Architecture

Colimail has a SvelteKit frontend inside a Tauri desktop window and a Rust backend for mail protocols, local storage and operating-system integration. The frontend invokes Tauri commands for requests; backend events report asynchronous updates such as IMAP IDLE notifications.

## Frontend

| Location | Responsibility |
| --- | --- |
| `src/routes/+page.svelte` | Mailbox page and UI composition |
| `src/routes/lib/page-controller.svelte.ts` | Page lifecycle and coordination |
| `src/routes/lib/state.svelte.ts` | Mailbox selection, message lists and UI state |
| `src/routes/handlers/` | Account/folder actions, compose/send, drafts, email operations and sync |
| `src/routes/components/` | Mailbox-specific components |
| `src/lib/components/` | Shared UI components |
| `src/lib/stores/` | Authentication and wallet state |
| `src/lib/supabase.ts` | App sign-in and session storage |
| `src/lib/cmvh/` | Blockchain verification, rewards and shared CMVH types |
| `src/lib/services/ens-resolver.ts` | ENS resolution and caching |

Keep page composition separate from mail actions. When changing an interaction, follow the component callback into its handler and then the corresponding Rust command. Shared TypeScript types and Rust serialized models must agree on field names and optional values.

## Backend

| Location under `src-tauri/src/` | Responsibility |
| --- | --- |
| `main.rs` | Desktop entry point and Tauri command registration |
| `setup.rs`, `tray.rs` | Startup integrations, deep links, updates and tray behavior |
| `lib.rs` | Library exports used by Rust integration tests; also contains a separate template runner |
| `commands/` | Tauri-facing account, folder, draft, send, auth, settings and other commands |
| `commands/emails/` | Fetching, cache access, synchronization, flags, attachments and MIME decoding |
| `idle_commands.rs`, `idle_manager/` | IMAP IDLE command interface and background sessions |
| `db.rs` | SQLite initialization and schema migrations |
| `models.rs` | Serialized account, folder, email and draft models |
| `security.rs` | OS keyring credential storage |
| `encryption.rs`, `commands/encryption_manager.rs` | Local content encryption and lock/unlock commands |
| `oauth2_config.rs` | Mail-provider OAuth credentials and token flows |
| `cmvh/`, `ens/` | Email signature processing and verification/name caches |
| `logger.rs` | Structured logs, rotation and log file access |

The desktop command list in `main.rs` is authoritative. Adding a command requires exporting it from the appropriate module and registering it there; exporting it from `lib.rs` alone does not make it available to the desktop frontend.

## Mail and storage flow

1. The mailbox page loads account and folder state through Tauri commands.
2. Cached message metadata comes from SQLite. Synchronization fetches remote mail and updates the cache; bodies and attachments have their own fetch paths.
3. Incremental sync uses folder UID validity and highest-UID state. Message identity must retain account and folder context as well as the IMAP UID.
4. User actions update mail state through handlers and backend commands. Preserve cache updates, UI rollback and error handling when changing optimistic operations.
5. IDLE sessions notify the frontend of remote changes. Page cleanup must release event listeners and timers.

Blocking mail protocol work is generally dispatched with `spawn_blocking`. Follow the surrounding command's threading and database-access pattern when making changes.

`db.rs` owns the schema for accounts, folders, messages, attachments, drafts, sync state, settings and feature caches. Read its SQL and migrations instead of maintaining a second schema definition in documentation. Database and log directories are computed separately in `db.rs` and `logger.rs`; neither should be inferred from the Tauri application identifier.

## Authentication and optional features

Mailbox accounts use IMAP/SMTP credentials or provider OAuth tokens. Supabase represents the app user and has a separate session lifecycle. WalletConnect manages wallet sessions for CMVH signatures and rewards. Keep these identities and logout paths distinct.

Mailbox credentials use the OS keyring; cached mail lives in SQLite. Local encryption has explicit enable, unlock and lock operations. Preserve those boundaries when changing storage, and avoid putting credentials or message content into new logs.

## Making changes

- For a UI action, start with the component and its handler; add or adjust a focused test beside the affected code.
- For a backend command, check its registration, serialized inputs/outputs and any cache or event side effects.
- For a database change, handle both a fresh database and an existing database through `db.rs` initialization/migrations.
- For startup or packaging, inspect `main.rs`, `setup.rs`, `src-tauri/tauri.conf.json` and `.github/workflows/`.

See [TESTING.md](TESTING.md) for checks and [README.md](README.md) for local setup.
