# Colimail

<div align="center">

**A next‑generation, high‑performance desktop email client**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue.svg)](https://v2.tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-1.74+-orange.svg)](https://www.rust-lang.org/)
[![SvelteKit](https://img.shields.io/badge/SvelteKit-2.x-red.svg)](https://kit.svelte.dev/)
[![Status](https://img.shields.io/badge/status-beta-purple.svg)](#)

[Overview](#overview) • [Highlights](#highlights) • [Feature Matrix](#feature-matrix) • [Architecture](#architecture) • [Installation](#installation) • [Usage](#usage) • [Development](#development) • [Commands](#tauri-commands) • [Database](#database-schema) • [Logging](#logging--observability) • [Security](#security--privacy) • [Performance](#performance-targets) • [Roadmap](#roadmap) • [Contributing](#contributing)

</div>

---

## Overview

Colimail is a cross‑platform desktop email client built with **Tauri 2**, **Rust**, and **SvelteKit (Svelte 5 runes)**. It focuses on predictable performance even with very large mailboxes (thousands to tens of thousands of messages) and provides an offline‑first experience with a local SQLite cache, secure credential storage, and real‑time push (IMAP IDLE) where supported.

### Vision
Deliver a fast, resource‑efficient alternative to heavy legacy clients (e.g. Thunderbird) while modernizing UX, security practices, and extensibility. Authentication for app users (Supabase) is optional: core mail operations work fully in guest mode.

### Core Principles
1. Offline‑first: Everything essential is cached locally; operations degrade gracefully.
2. Non‑blocking I/O: All heavy IMAP/SMTP/database work off the main async executor via `spawn_blocking`.
3. Incremental sync: Minimal deltas, efficient flag updates, attachment detection background tasks.
4. Secure by default: Credentials in OS keyring, tokens chunked to avoid platform limits.
5. Progressive enhancement: Guest mode → optional cloud features (user profile, subscription, future sync).

## Highlights (v1.0.0 latest)

Recent major capabilities (see full [CHANGELOG](./CHANGELOG.md)):

| Area | Key Additions |
|------|---------------|
| **CMVH Email Signing** | **Cryptographic email signing using EIP-712 standard; local & on-chain verification (Arbitrum); signature caching; first-time user onboarding guide** |
| **Wallet Integration** | **WalletConnect QR code-based mobile wallet connection; ENS name resolution with 3-layer caching; unified wallet status display; auto-session timeout** |
| **Reward Pool** | **Attach wACT cryptocurrency rewards to emails; on-chain reward creation/claiming; rewards dashboard; 30-day expiry with cancellation support** |
| Authentication | Supabase email/password + Google OAuth; user profile sync to local DB; deep link handling; robust session refresh & environment detection |
| Security | OS keyring credential storage with chunking; **AES-256-GCM local data encryption** for email content; Argon2 key derivation; zero-knowledge master password; auto-update with signature verification |
| Logging | Structured JSON logs, rotation (daily, 7‑day retention), backend log viewer, export & filtering groundwork |
| UX Refactors | Modularized large Svelte pages into domain handlers; shadcn‑svelte sidebar patterns (sidebar‑09 / sidebar‑13); dialog‑based account & settings management |
| Real‑time | Refactored IMAP IDLE manager, split into focused modules; single‑folder monitoring to respect provider limits |
| Email Ops | Flag/star sync (bidirectional), background BODYSTRUCTURE attachment detection with retry, UTF‑7 folder handling |
| Drafts | Local SQLite draft storage (compose/reply/forward) with auto‑save & attachment metadata |
| Folders | Create/delete (remote vs local), system folder protection, context menus |
| System Tray | Windows tray integration with minimize‑to‑tray preference |
| Performance | Full mailbox sync (no artificial cap); batch & adaptive fetching; optimized logging noise reduction; ENS batch resolution (50x improvement) |

## Feature Matrix

| Category | Implemented | Notes |
|----------|-------------|-------|
| Multi‑Account IMAP/SMTP | ✅ | OAuth2 (Gmail/Outlook) & Basic auth; presets for 13 providers |
| **CMVH Email Signing** | ✅ | **EIP-712 cryptographic signatures; local & on-chain verification (Arbitrum)** |
| **Wallet Integration (WalletConnect)** | ✅ | **Mobile wallet QR code connection; MetaMask, Trust Wallet, Rainbow support** |
| **ENS Name Resolution** | ✅ | **3-layer caching (Memory/SQLite/RPC); batch optimization; 90%+ cache hit rate** |
| **Reward Pool (wACT)** | ✅ | **Attach crypto rewards to emails; on-chain claiming; rewards dashboard** |
| OAuth2 Deep Link | ✅ | Desktop + browser flow; `colimail://auth/callback` scheme |
| User Auth (Supabase) | ✅ | Optional; guest mode fully supported |
| Local Cache | ✅ | SQLite + incremental sync; flag & attachment metadata |
| Drafts | ✅ | Local only (avoids Gmail IMAP draft mismatch) |
| Rich Compose | ✅ | Reply / Forward / Attachments / Display name support |
| Folders CRUD | ✅ | Remote capability detection + local fallback |
| Search & Filter | ✅ | Subject / From / To + unread toggle; pagination |
| Real‑time Push (IDLE) | ✅ | Inbox only (provider connection limits) |
| System Notifications | ✅ | New mail, actions; permission bootstrap |
| System Tray | ✅ | Minimize to tray configurable |
| Structured Logging | ✅ | JSON + live viewer; rotation |
| Credential Security | ✅ | OS keyring + chunking; password column removed |
| **Local Data Encryption** | ✅ | **AES-256-GCM encryption for cached email content** |
| Display Name Detection | ✅ | Sent folder heuristic & multi‑language matching |
| Attachment Detection | ✅ | Background BODYSTRUCTURE processing & retry |
| Performance Targets | 🔄 | Constant tuning (see below) |
| Multi‑Language UI | ⏳ | Planned |
| Calendar Integration | ⏳ | Planned |
| Billing / Subscription | ⏳ | Planned (metadata tier field present) |

Legend: ✅ implemented • 🔄 ongoing optimization • ⏳ planned

## Architecture

### High Level
Frontend (SvelteKit + runes) calls Rust backend via Tauri invoke commands. Rust performs IMAP/SMTP, secure credential operations, structured logging, and local persistence. Supabase auth adds an "app user" layer separate from email account configurations.

### Backend Modules
| Module | Purpose |
|--------|---------|
| `main.rs` | App bootstrap, command registration, system tray, deep link & notification permission handling, IDLE auto‑start |
| `db.rs` | SQLite initialization & migrations (accounts, folders, emails, drafts, app_user, settings, cmvh_verification_cache, ens_cache) |
| `commands/` | Domain commands (accounts, emails sync & fetch, folders CRUD, drafts, send, auth, logs, notifications, OAuth2, test_connection, display name detection, **cmvh, send_cmvh**) |
| **`cmvh/`** | **EIP-712 email signing (signer.rs), signature verification (verifier.rs), CMVH header parsing (parser.rs), RFC5322 MIME building (mime.rs), verification caching (cache.rs), types (types.rs)** |
| `idle_manager/` | Real‑time IMAP IDLE session orchestration split into manager/session/notification/types |
| `security.rs` | OS keyring credential chunking, retrieval, deletion, updates |
| `encryption.rs` | **AES-256-GCM encryption/decryption with Argon2 key derivation** |
| `logger.rs` | Structured tracing subscriber setup, rotation, directory exposure |
| `oauth2_config.rs` | Desktop OAuth credentials initialization |
| `models.rs` | Shared structs (e.g. `AccountConfig`, draft & folder models, user profile) |

### Frontend Structure
| Area | Highlights |
|------|-----------|
| `routes/+page.svelte` | Orchestrates UI layout & delegates logic to `handlers/*` modules |
| `handlers/` | Split concerns: account-folder, compose-send, draft-management, email-operations, sync-idle |
| `stores/auth.svelte.ts` | Supabase session + user reactive rune store with manual refresh capability |
| **`stores/walletconnect.svelte.ts`** | **WalletConnect state management; wallet connection, session restoration, error handling** |
| **`stores/wallet.svelte.ts`** | **Unified wallet interface wrapping WalletConnect; ENS resolution integration** |
| **`lib/cmvh/`** | **CMVH TypeScript modules: blockchain.ts (smart contract), verifier.ts (signature verification), reward-service.ts (wACT rewards), types.ts, config.ts** |
| **`lib/services/ens-resolver.ts`** | **3-layer ENS caching (Memory/SQLite/RPC); batch resolution; request deduplication** |
| `lib/state.svelte.ts` | Global application state (selected account/folder, pagination, filters) |
| `components/*.svelte` | UI building blocks (Sidebar, dialogs, email list, body viewer, log viewer, **WalletConnectDialog**) |
| Shadcn‑svelte | Adopted official sidebar & dialog patterns for consistency |

### Async & Performance Patterns
* IMAP/SMTP & heavy parsing via `tokio::spawn_blocking` to avoid saturating async runtime.
* Incremental sync tracks `UIDVALIDITY`, highest UID, flags, attachments; reduces redundant fetches.
* Background tasks (BODYSTRUCTURE, flag sync) are resilient with retry + failure marking to prevent infinite loops.
* Logging uses async appender with daily rotation, minimal overhead (<1% CPU typical).

## Installation

Pre‑built packages are published on the [Releases page](https://github.com/daodreamer/colimail/releases). (Names will update as versions progress.)

Current supported targets:
* Windows x64
* macOS (Apple Silicon) — Intel support considered on demand
* Linux (.deb / AppImage) planned in upcoming release cycle (workflows partially prepared)

### Windows
1. Download latest `.msi` installer
2. Execute installer wizard
3. Launch from Start Menu (first run initializes DB + logs)

### macOS (Apple Silicon)
1. Download `.dmg`
2. Drag Colimail to Applications
3. If Gatekeeper prompts: right‑click → Open

### Environment Variables (Development)
Create a `.env` or use your shell for Supabase:
```
VITE_SUPABASE_URL=https://YOUR_PROJECT.supabase.co
VITE_SUPABASE_ANON_KEY=YOUR_KEY
```

## Usage

### First Launch Flow
1. Start app → auto‑creates DB & log directory.
2. If no accounts: UI presents Add Account dialog entry point.
3. Add account via OAuth2 (Gmail / Outlook) or Manual (provider preset or custom). Test Connection ensures settings validity before creation.
4. Initial full sync populates local cache; attachment & flag background tasks kick in.
5. IDLE auto‑monitoring starts (Inbox only) → system notifications for new mail.

### Account Types
* Basic (username/password) — password stored only in OS keyring.
* OAuth2 — access/refresh tokens chunked & stored securely; refresh handled server‑side logic.

### Display Name
Optional per account; auto‑detection scans recent Sent items. Can be manually edited (including OAuth2 accounts).

### Draft Workflow
Compose ↔ Auto‑save (3s debounce) ↔ Send deletes draft. Reply/Forward preserve original reference metadata.

### Search & Pagination
Client‑side search (subject/from/to) + unread toggle. Pagination defaults to 50 items/page; full count displayed.

## Development

### Prerequisites
* Rust stable (1.74+ recommended)
* Node.js 18+
* Platform toolchain (VS Build Tools / Xcode CLT / Linux GTK + WebKit2 deps)

### Install & Run
```bash
git clone https://github.com/daodreamer/colimail.git
cd colimail
npm install
npm run tauri dev
```

### Scripts
```bash
npm run dev          # Frontend only (web preview)
npm run tauri dev    # Full desktop app
npm run build        # Frontend production build
npm run check        # Type & Svelte validation

cd src-tauri
cargo fmt && cargo check && cargo clippy -- -D warnings
cargo build --release
```

### Project Structure (Simplified)
```
src/              # SvelteKit frontend
   routes/         # Pages & dialogs (auth/, settings/, handlers/ modules)
   lib/            # Stores, global state, utilities
   components/     # Reusable UI pieces
src-tauri/
   src/
      main.rs       # Entry & tray/deep link/IDLE bootstrap
      db.rs         # SQLite init & migrations
      security.rs   # Keyring credential logic
      logger.rs     # Tracing + rotation
      commands/     # Tauri commands (domain modules)
      idle_manager/ # Real-time IMAP IDLE subsystem
      models.rs     # Shared data structs
CHANGELOG.md
README.md
```

## Tauri Commands

Backend exposes a rich command surface (invoked via `invoke("command_name", payload)`):

| Group | Examples |
|-------|----------|
| Accounts | `save_account_config`, `load_account_configs`, `delete_account`, `detect_display_name_from_sent` |
| Emails | `fetch_emails`, `fetch_email_body`, `fetch_email_body_cached`, `load_emails_from_cache`, `sync_emails`, `sync_email_flags`, `sync_specific_email_flags`, `mark_email_as_read`, `mark_email_as_unread`, `mark_email_as_flagged`, `mark_email_as_unflagged`, `move_email_to_trash`, `delete_email` |
| Folders | `fetch_folders`, `sync_folders`, `load_folders`, `create_remote_folder`, `delete_remote_folder`, `create_local_folder`, `delete_local_folder`, `check_folder_capabilities` |
| Drafts | `save_draft`, `load_draft`, `list_drafts`, `delete_draft` |
| Sending | `send_email`, `reply_email`, `forward_email`, `get_attachment_size_limit`, `download_attachment`, `save_attachment_to_file` |
| Auth (App User) | `get_secure_storage`, `set_secure_storage`, `delete_secure_storage`, `sync_app_user`, `get_app_user`, `delete_app_user` |
| OAuth2 Flow | `start_oauth2_flow`, `listen_for_oauth_callback`, `complete_oauth2_flow` |
| Connection Test | `test_connection` |
| IDLE | `start_idle`, `stop_idle`, `stop_all_idle`, `is_idle_active`, `start_idle_for_account`, `stop_idle_for_account`, `start_idle_for_all_accounts` |
| Logs | `get_log_directory`, `get_current_log_file`, `read_recent_logs`, `list_log_files`, `read_log_file` |
| Notifications/Settings | `get_sync_interval`, `set_sync_interval`, `get_notification_enabled`, `set_notification_enabled`, `get_sound_enabled`, `set_sound_enabled`, `get_minimize_to_tray`, `set_minimize_to_tray`, `should_sync`, `get_last_sync_time` |

## Database Schema

SQLite schema (simplified, see `db.rs` for authoritative definitions):
* `accounts` — id, email, servers, ports, auth_type, display_name, oauth metadata
* `folders` — id, account_id, name (UTF‑7 encoded raw), display_name, is_local, sync metadata
* `emails` — id, account_id, folder_id, uid, subject, from_addr, to_addr, cc_addr, date, flags, seen, has_attachments, internaldate
* `drafts` — id, account_id, to_addr, cc_addr, subject, body, attachments (JSON), draft_type, original_email_id, timestamps
* `app_user` — id, email, name, avatar_url, subscription_tier, subscription_expires_at, last_synced_at, created_at
* `settings` — key/value (e.g. `minimize_to_tray`, `sync_interval`, notification preferences)

Design notes:
* Attachments metadata stored separate from binary payload (fetched on demand).
* Flags & seen status cached for fast filtering & optimistic UI.
* Drafts localized to avoid IMAP provider inconsistencies (e.g. Gmail).

## Logging & Observability

Structured tracing (JSON) + optional pretty console in dev. Commands to inspect:
* `get_log_directory` – base path (per‑platform user data dir)
* `get_current_log_file` – today's rotating file
* `read_recent_logs(lines)` – tail N lines
* `list_log_files` – available historical files
* `read_log_file(filename)` – complete file content
* `export_logs_as_zip` – **NEW in v0.6.3**: Export all logs as timestamped ZIP archive

Rotation: daily; retention: 7 days. Sensitive data (passwords, tokens, message bodies) deliberately excluded.

### Developer Tools

#### Log Viewer (`log-viewer.html`)

A standalone HTML tool for analyzing exported log files (located in project root, **not included in release builds**):

**Features:**
* 📦 Drag & drop ZIP or LOG files
* 🎯 Multi-level filtering (TRACE/DEBUG/INFO/WARN/ERROR)
* 🔍 Real-time keyword search with highlighting
* 📊 Statistics dashboard with log level distribution chart
* 🐛 Error categorization (Connection/Timeout/Parsing/Operation failures)
* ⚡ Performance analysis (Top 10 slowest operations)
* 💾 Export filtered results as TXT/CSV/ZIP
* 🎨 Expandable JSON metadata for each log entry
* 🌐 Zero dependencies - runs in any modern browser

**Usage:**
1. Export logs from app: Settings → About → Export Logs as ZIP
2. Open `log-viewer.html` in your browser
3. Drag the exported ZIP file onto the viewer
4. Use filters and search to analyze logs
5. Export filtered subsets for bug reports

**Location:** Root directory (`maildesk/log-viewer.html`)
**Platform:** Cross-platform (Windows/macOS/Linux)
**Privacy:** All processing done locally in browser, no data sent anywhere

## Security & Privacy

| Aspect | Status |
|--------|--------|
| Credential storage | OS keyring via `keyring` crate; chunking for long tokens |
| Password plaintext in DB | Removed (migrated to keyring) |
| OAuth2 tokens | Stored securely (chunked) |
| **Local data encryption** | **✅ AES-256-GCM for email subjects, bodies, and attachments** |
| **Key management** | **Zero-knowledge: Argon2 key derivation from master password** |
| **Session security** | **Keys in memory only; auto-locked on app close** |
| Session (Supabase) | Local storage (sandboxed) + optional secure storage fallback |
| Email content | Local only; no external telemetry |
| Future | Multi‑tenant cloud sync (opt‑in); biometric unlock |

### Encryption Details

**What's Protected:**
- Email subjects (`emails.subject`) - encrypted with AES-256-GCM
- Email bodies (`emails.body`) - encrypted with AES-256-GCM
- Attachment data (`attachments.data`) - encrypted with AES-256-GCM

**Security Features:**
- **Algorithm**: AES-256-GCM (authenticated encryption)
- **Key Derivation**: Argon2id (memory-hard, brute-force resistant)
- **Key Storage**: Memory only - never persisted to disk
- **Session Isolation**: Encryption locked on app close; password required on restart
- **Zero-Knowledge**: Master password cannot be recovered if lost
- **Memory Safety**: Keys securely cleared using `zeroize` crate

**Performance**: < 10% overhead for encrypt/decrypt operations

**Setup**: Settings → Privacy & visibility → Enable encryption (8+ character password required)

**Documentation**: See [ENCRYPTION_USAGE.md](./ENCRYPTION_USAGE.md) and [ENCRYPTION_TESTING.md](./ENCRYPTION_TESTING.md)

Report concerns via [SECURITY.md](./SECURITY.md).

## Performance Targets

| Metric | Target | Current Status* |
|--------|--------|-----------------|
| Cold startup | ≤ 1.5s | Achieved on reference Windows/macOS test rigs |
| Idle memory | ≤ 80MB | ~70MB (recent measurements) |
| Sync throughput | 100 emails ≤ 3s | Achieved; large mailbox full sync optimized |
| Flag update latency | < 300ms | Single‑UID sync ~50ms |
| Package size | ≤ 30MB | Within range (depends on platform) |

*Indicative values – verify with your environment; logging verbosity & antivirus can affect results.

## Roadmap

Short‑term (0.7.x):
* Log search & export (ZIP)
* Multi‑language UI (i18n framework integration)
* Notification preference expansion (per account/folder)
* Calendar integration groundwork (ICS parsing, provider discovery)

Mid‑term (0.8.x – 0.9.x):
* Advanced search index (SQLite FTS or Tantivy)
* Rule engine (filter, auto‑tag, move)
* Encrypted local message store (AES‑GCM with user key)
* Subscription / billing UI & entitlement checks

Long‑term (1.x):
* Cross‑device sync (optional cloud) for flags, drafts, rules
* Plugin system (scriptable automations)
* Smart categorization & ML‑assisted triage

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for workflow & standards. Please ensure:
1. `cargo fmt && cargo check && cargo clippy -- -D warnings` passes
2. `npm run check` yields 0 errors/warnings
3. No secrets added to repository; use env vars & keyring

### Quick Start
```bash
git checkout -b feature/my-improvement
# ... changes ...
git commit -m "feat: add X"
git push origin feature/my-improvement
```
Open a Pull Request with description, screenshots (UI), and performance notes if relevant.

## Recommended IDE Setup

VS Code + Svelte extension + Tauri extension + rust-analyzer. Enable format‑on‑save for Rust & TypeScript and run `npm run check` before commits.

## License

MIT License – see [LICENSE](./LICENSE).

## Acknowledgments

Built on great open source work: Tauri, SvelteKit, Supabase, tokio, lettre, imap, sqlx, tracing, and the broader Rust/Svelte ecosystems.

---

For historical changes, consult the full [CHANGELOG](./CHANGELOG.md).
