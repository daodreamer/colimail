# Testing

Run commands below from the repository root unless another directory is specified. Complete the setup in [README.md](README.md) first. Test counts and coverage depend on the current checkout; use runner output rather than historical reports.

## Frontend

```sh
npm run check
npm run test:run
npm run build
```

Use `npm test` for watch mode, or pass a path to select tests:

```sh
npm run test:run -- src/routes/handlers/email-operations.test.ts
npm run test:run -- --coverage
```

Vitest configuration lives in `vitest.config.ts`, with shared setup in `src/tests/setup.ts`. Tests are colocated with handlers, components and services. Browser and Tauri mocks make these useful for isolated behavior checks; they do not exercise a running desktop backend.

## Rust

```sh
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Unit tests live beside implementation code; integration tests live in `src-tauri/tests/`. Some credential tests require the operating system keyring. Tests marked `#[ignore]` do not run by default.

For focused checks:

```sh
cargo test --manifest-path src-tauri/Cargo.toml --test security_tests
cargo test --manifest-path src-tauri/Cargo.toml --test imap_protocol_tests
cargo test --manifest-path src-tauri/Cargo.toml --test smtp_protocol_tests
```

The protocol suites mix local checks with ignored tests against real servers. To run a real connection test, configure a dedicated test account in the shell:

- IMAP: `TEST_IMAP_EMAIL`, `TEST_IMAP_PASSWORD`, `TEST_IMAP_SERVER`, `TEST_IMAP_PORT`.
- SMTP: `TEST_SMTP_EMAIL`, `TEST_SMTP_PASSWORD`, `TEST_SMTP_SERVER`, `TEST_SMTP_PORT`.
- OAuth test cases also use `TEST_OAUTH2_ACCESS_TOKEN`.

```sh
cargo test --manifest-path src-tauri/Cargo.toml --test imap_protocol_tests test_imap_connection_success -- --ignored --nocapture
cargo test --manifest-path src-tauri/Cargo.toml --test smtp_protocol_tests test_smtp_connection_starttls -- --ignored --nocapture
```

Read each ignored test before enabling it: SMTP send cases send real mail, and some cases use fixed example recipients or ports. Some connection cases return successfully when the network is unavailable, so a green run alone does not prove server connectivity.

## Desktop smoke tests

`e2e-tests/` is a separate WebdriverIO package. Its current configuration targets the Windows debug executable and checks startup and basic UI rendering, not a complete mailbox workflow.

Install `tauri-driver` and a matching native WebDriver, then run:

```sh
cargo install tauri-driver --locked
cd e2e-tests
npm ci
npm test
```

`wdio.conf.js` builds the desktop app before starting the driver. Review its executable path when changing platforms. The fixture scripts in `scripts/` are separate helpers, not an automatic part of this smoke suite.

## Benchmarks

```sh
cargo bench --manifest-path src-tauri/Cargo.toml --bench email_parsing
cargo bench --manifest-path src-tauri/Cargo.toml --bench database_queries
```

Compare results on the same machine and configuration. These measure selected operations, not overall app performance.

## Manual regression checks

Use a test mailbox for changes affecting mail operations:

1. Add an account, load folders, synchronize, and switch accounts or folders.
2. Open cached and uncached messages; check HTML, plain text and attachments.
3. Mark mail read/unread and starred/unstarred, then synchronize again.
4. Compose, save and reopen a draft; send, reply and forward to a test address.
5. Verify new-mail notifications and tray behavior after restart.
6. When relevant, exercise encryption lock/unlock, OAuth callbacks, wallet connection and CMVH verification.

The build workflow in `.github/workflows/ci.yml` is the source of truth for CI checks. Local Vitest, Cargo tests and desktop smoke tests should be run explicitly; do not assume a successful build ran them.
