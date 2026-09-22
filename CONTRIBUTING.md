# Contributing to Colimail

Use [README.md](README.md) to set up the app, [ARCHITECTURE.md](ARCHITECTURE.md)
to locate code, and [TESTING.md](TESTING.md) for test commands and environment requirements.

## Workflow

1. Create a branch for a focused change.
2. Follow the existing code style. Use English for code and comments, meaningful names,
   and comments that explain constraints or decisions rather than repeat the code.
3. Add or update tests for changed behavior. For refactoring, verify existing behavior
   before and after the change.
4. Run the relevant checks below and update documentation affected by the change.
5. Open a pull request describing the problem, resulting behavior, validation, and any limitations.

For frontend changes, run from the repository root:

```bash
npm run check
npm run test:run
npm run build
```

For Rust changes, run from `src-tauri/`:

```bash
cargo fmt
cargo check
cargo clippy -- -D warnings
cargo test
```

Use Svelte 5 runes for reactive state. Match frontend command payloads to Rust
signatures. Handle errors explicitly and keep blocking network work off the async executor.
Do not commit credentials, signing keys, generated reports, or personal configuration.

## Local builds and release signing

Use `npm run tauri dev` for everyday development; it does not require an updater
signing key. Production updater artifacts require the maintainer's
`TAURI_SIGNING_PRIVATE_KEY`.

For a local package without updater artifacts, temporarily set
`bundle.createUpdaterArtifacts` to `false` in `src-tauri/tauri.conf.json`, then run
`npm run tauri build`. Restore that setting before submitting changes. Keep the
updater public key and endpoint intact.

## Manual checks

For mail changes, check the affected workflow with a test mailbox: account setup,
sync, read/star changes, compose/reply/forward, drafts, attachments, or deletion.
For CMVH or wallet changes, check signing and verification, WalletConnect connection,
cancellation and session restoration, and ENS resolution as applicable. Exercise
reward transactions only in a deliberately configured test environment with test funds.

## Issues and pull requests

For bugs, include the app version, OS, reproduction steps, expected and actual
behavior, and relevant logs with private data removed. For proposals, explain the
user problem and expected behavior. Keep feedback respectful and constructive.

Use a descriptive commit message, such as `fix: preserve selection after sync` or
`refactor: share email status updates`. Keep each pull request focused and link
related issues when available.
