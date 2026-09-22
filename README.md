# Colimail

Colimail is a desktop email client built with Tauri 2, Rust, SvelteKit and Svelte 5. It supports IMAP/SMTP accounts, a local SQLite cache, drafts, attachments, folders and IMAP IDLE notifications. Optional features include Supabase sign-in, local content encryption, and CMVH email signatures and rewards through WalletConnect.

## Run locally

Use Node.js 20 or newer, Rust stable, and the native build tools required by Tauri. On Windows this includes Visual Studio C++ Build Tools and WebView2. The CI configuration lists the Linux build dependencies.

From the repository root:

```sh
npm ci
```

Prepare configuration before starting:

- Copy `.env.example` to `.env` if it does not already exist, then set `VITE_SUPABASE_URL` and `VITE_SUPABASE_ANON_KEY`. App sign-in is optional, but the current frontend initializes the Supabase client at import time and requires these configuration values.
- Copy `src-tauri/oauth2_credentials.json.example` to `src-tauri/oauth2_credentials.json` if it does not already exist. Tauri bundles this file; real provider credentials are needed for mailbox OAuth login. See `src-tauri/src/oauth2_config.rs` for loading order and environment fallbacks.

```sh
npm run tauri dev
```

Add a mailbox using provider settings or custom IMAP/SMTP settings, test the connection, and synchronize mail. Mailbox credentials and app sign-in are separate.

## Common commands

Run these from the repository root:

| Command | Purpose |
| --- | --- |
| `npm run dev` | Frontend preview; desktop commands require Tauri |
| `npm run tauri dev` | Desktop development app |
| `npm run check` | Svelte and TypeScript checks |
| `npm run test:run` | Frontend tests, once |
| `npm run build` | Frontend production build |
| `npm run tauri build` | Desktop release build and installers |

See [TESTING.md](TESTING.md) for Rust checks, test prerequisites and manual verification.

## Find your way around

- [ARCHITECTURE.md](ARCHITECTURE.md): code layout, data flow and maintenance entry points.
- [CONTRIBUTING.md](CONTRIBUTING.md): contribution workflow.
- [CHANGELOG.md](CHANGELOG.md): release history.
- `src/routes/`: mailbox page, components and action handlers.
- `src/lib/`: shared components, authentication, wallet and CMVH services.
- `src-tauri/src/`: desktop commands, mail protocols, storage and security.
- `.github/workflows/`: build checks and release packaging.

For troubleshooting, the app supports log viewing and ZIP export. Open `log-viewer.html` in a browser to inspect exported logs. Runtime log locations come from `src-tauri/src/logger.rs`; database locations come from `src-tauri/src/db.rs`.

Licensed under [MIT](LICENSE).
