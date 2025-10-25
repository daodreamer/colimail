# Changelog

All notable changes to Colimail will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Password encryption using platform keychain
- Secure OAuth2 token storage
- Email search functionality
- Calendar integration
- Multi-language support

## [0.2.0] - TBD

### Changed
- **UI Framework**: Migrated entire frontend to shadcn-svelte component library with Tailwind CSS
  - Replaced all custom CSS components with professional shadcn-svelte components
  - Integrated Tailwind CSS for consistent, modern styling
  - Improved component architecture with better composition and reusability
  - Enhanced dark mode support with proper theming
  - Better accessibility support built into components

### Added
- **UI Components**: Implemented shadcn-svelte components across all views
  - `AccountsSidebar`: Button, ScrollArea, Badge, Separator
  - `FoldersSidebar`: Button, ScrollArea
  - `EmailList`: Card, ScrollArea, Badge
  - `EmailBody`: Card, Button, ScrollArea, Separator
  - `ComposeDialog`: Dialog, Input, Textarea, Button, Label, Badge
  - `AttachmentList`: Button, Badge
  - `Settings Page`: Card, Input, Label, Separator
- **Notifications**: Replaced custom toast with Sonner toast library for professional notifications
- **Typography**: Added Tailwind Typography for better email body rendering

### Improved
- **Design Consistency**: All UI elements now follow a unified design system
- **User Experience**: Smoother interactions and transitions
- **Code Maintainability**: Simplified component code with utility-first CSS
- **Performance**: Optimized component rendering with Tailwind's JIT compiler

## [0.1.4] - 2025-10-25

### Fixed
- **Critical**: Fixed database schema initialization issue on macOS causing "no such column: cc_addr" error
  - Moved all required columns (`cc_addr`, `has_attachments`, `flags`, `seen`) directly into CREATE TABLE statement
  - Ensured proper column initialization for fresh database installations
  - Maintained backward compatibility with existing databases through migration statements
  - Issue primarily affected macOS M4 chip users with new installations
- **UI**: Fixed notification window positioning issue on macOS with Retina displays
  - **Root cause**: Used `PhysicalPosition` with logical coordinates, causing incorrect positioning on Retina displays
  - **Solution**: Changed to `LogicalPosition` to properly handle DPI scaling
  - Notification windows now correctly appear in bottom-right corner on all screen resolutions
  - Properly converts physical pixels (4096x2304) to logical pixels (2048x1152) for Retina displays
  - Uses appropriate coordinate system: logical coordinates for logical pixel calculations
  - Adjusted margins to 20px each for comfortable spacing above Dock/taskbar
  - Added comprehensive logging for debugging (physical/logical dimensions, scale factor, window size)

## [0.1.1] - 2025-10-24

### Fixed
- GitHub Actions permissions for automated releases
- macOS Apple Silicon build artifact paths
- Repository references updated to daodreamer/colimail

### Changed
- Streamlined release builds to Windows x64 and macOS Apple Silicon only
- Removed Linux and macOS Intel builds from automated releases

## [0.1.0] - 2025-10-24 [YANKED]

**Note**: This release was yanked due to GitHub Actions configuration issues. Use v0.1.1 instead.

### Added
- Initial beta release
- Multiple email account support (IMAP/SMTP)
- OAuth2 authentication for Gmail and Microsoft accounts
- Email sync with SQLite local storage
- Send and receive emails
- Attachment handling
- Folder management (INBOX, Sent, Drafts, etc.)
- Rich text email composition
- Cross-platform support (Windows, macOS, Linux)
- Modern UI built with Svelte 5 and Tauri 2
- Background email sync
- System notifications for new emails

### Known Issues
- Passwords stored in plaintext (security improvement planned)
- OAuth2 tokens not encrypted
- Limited search functionality
- No email filtering or rules
- No spam detection

### Performance
- Startup time: ~1.5 seconds
- Memory usage: ~70 MB (idle)
- Sync speed: ~100 emails in 3 seconds

---

## Version History

### [0.1.0] - 2025-01-24
Initial public beta release for testing

---

**Legend**:
- `Added`: New features
- `Changed`: Changes to existing functionality
- `Deprecated`: Soon-to-be removed features
- `Removed`: Removed features
- `Fixed`: Bug fixes
- `Security`: Security improvements
