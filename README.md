# Colimail

<div align="center">

**A lightweight, high-performance desktop email client**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue.svg)](https://v2.tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![SvelteKit](https://img.shields.io/badge/SvelteKit-2.0-red.svg)](https://kit.svelte.dev/)

[Features](#features) • [Installation](#installation) • [Usage](#usage) • [Development](#development) • [Contributing](#contributing)

</div>

---

## Overview

Colimail is a cross-platform desktop email client built with Tauri 2, SvelteKit, and Rust. Designed as a high-performance alternative to traditional email clients like Thunderbird, it handles large email volumes without performance degradation.

### Why Colimail?

- **Lightweight**: Package size ≤ 30 MB, memory footprint ≤ 80 MB (idle)
- **Fast**: Startup ≤ 1.5s, sync 100 emails in ≤ 3s, query latency ≤ 50ms
- **Modern**: Built with Svelte 5 + Tauri 2 for native performance
- **Secure**: Local SQLite storage, OAuth2 support for Gmail/Outlook
- **Cross-platform**: Windows, macOS, Linux

## Features

- Multiple email account support (IMAP/SMTP)
- OAuth2 authentication for Gmail and Microsoft accounts
- Fast local email sync with SQLite
- Rich text email composition
- Attachment handling
- Folder management
- Search and filtering
- Modern, responsive UI
- Background sync and notifications

## Installation

### Download Pre-built Binaries

Visit the [Releases page](https://github.com/daodreamer/colimail/releases) to download the latest version:

- **Windows**: `Colimail_0.1.0_x64_en-US.msi`
- **macOS**: `Colimail_0.1.0_aarch64.dmg` (Apple Silicon) or `Colimail_0.1.0_x64.dmg` (Intel)
- **Linux**: `colimail_0.1.0_amd64.deb` or `colimail_0.1.0_amd64.AppImage`

### Installation Instructions

#### Windows
1. Download the `.msi` installer
2. Double-click to run the installer
3. Follow the installation wizard
4. Launch Colimail from the Start Menu

#### macOS
1. Download the `.dmg` file
2. Open the DMG and drag Colimail to Applications
3. First launch: Right-click → Open (to bypass Gatekeeper)

#### Linux
**Debian/Ubuntu (.deb)**:
```bash
sudo dpkg -i colimail_0.1.0_amd64.deb
```

**AppImage**:
```bash
chmod +x colimail_0.1.0_amd64.AppImage
./colimail_0.1.0_amd64.AppImage
```

## Usage

### First Launch

1. Open Colimail
2. Click "Add Account" in Settings
3. Choose authentication method:
   - **OAuth2**: For Gmail/Outlook (recommended)
   - **Password**: For standard IMAP/SMTP servers

### Adding an Account

**Gmail (OAuth2)**:
1. Select "Gmail" → "Sign in with Google"
2. Complete OAuth2 authorization
3. Email syncs automatically

**Custom IMAP/SMTP**:
1. Enter email address and password
2. Configure server settings:
   - IMAP server: `imap.example.com:993`
   - SMTP server: `smtp.example.com:587`
3. Save and sync

### OAuth2 Setup

For Gmail and Microsoft OAuth2 configuration, see [OAUTH2_SETUP.md](./OAUTH2_SETUP.md).

## Development

### Prerequisites

- **Rust** 1.70+ ([Install](https://rustup.rs/))
- **Node.js** 18+ ([Install](https://nodejs.org/))
- **Platform-specific tools**:
  - Windows: Visual Studio Build Tools
  - macOS: Xcode Command Line Tools
  - Linux: `build-essential`, `libwebkit2gtk-4.0-dev`, `libssl-dev`

### Setup

```bash
# Clone the repository
git clone https://github.com/daodreamer/colimail.git
cd colimail

# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Build Commands

```bash
# Frontend development
npm run dev              # Run SvelteKit dev server
npm run build            # Build frontend
npm run check            # Type checking

# Backend development
cd src-tauri
cargo fmt                # Format code
cargo check              # Check for errors
cargo build              # Build backend
cargo build --release    # Release build

# Full application
npm run tauri dev        # Run desktop app in dev mode
npm run tauri build      # Build production app
```

### Project Structure

```
colimail/
├── src/                    # SvelteKit frontend
│   ├── routes/            # Page routes
│   └── lib/               # Components & utilities
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── main.rs        # Application entry point
│   │   ├── db.rs          # SQLite database layer
│   │   ├── commands/      # Tauri commands
│   │   └── models.rs      # Data models
│   └── Cargo.toml
├── CLAUDE.md              # Development guidelines
└── package.json
```

## Architecture

- **Frontend**: SvelteKit (Svelte 5) + TypeScript
- **Backend**: Rust + Tauri 2
- **Database**: SQLite (via rusqlite)
- **Email**: IMAP (`imap` crate) + SMTP (`lettre`)
- **Authentication**: OAuth2 (`oauth2` crate)

For detailed architecture documentation, see [CLAUDE.md](./CLAUDE.md).

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

### Quick Start

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit changes: `git commit -m 'Add amazing feature'`
4. Push to branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

### Reporting Issues

Found a bug or have a feature request? Please [open an issue](https://github.com/daodreamer/colimail/issues/new).

## Security & Privacy

- **Local Storage**: All emails stored locally in SQLite
- **Password Security**: Passwords stored in plaintext (⚠️ planned for improvement)
- **OAuth2**: Tokens stored securely with platform-specific encryption (planned)
- **No Telemetry**: No data collection or tracking

For security concerns, please see [SECURITY.md](./SECURITY.md).

## Performance Targets

- Startup time: ≤ 1.5 seconds (cold start)
- Memory footprint: ≤ 80 MB (idle)
- CPU usage: ≤ 5% (idle)
- Email sync: 100 emails in ≤ 3 seconds
- Package size: ≤ 30 MB (Windows executable)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

Built with:
- [Tauri](https://tauri.app/) - Desktop application framework
- [SvelteKit](https://kit.svelte.dev/) - Frontend framework
- [Rust](https://www.rust-lang.org/) - Backend language
- [rusqlite](https://github.com/rusqlite/rusqlite) - SQLite bindings
- [lettre](https://github.com/lettre/lettre) - SMTP client

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
