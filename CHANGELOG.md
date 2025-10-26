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

## [0.2.2] - 2025-10-26

### Fixed
- **HTML Email Rendering**: Fixed critical rendering issue where HTML email content broke page layout
  - **Root Cause**: Emails containing complete HTML documents (`<!DOCTYPE>`, `<html>`, `<head>`, `<body>`) were being wrapped with another HTML document structure, creating nested HTML that caused rendering chaos
  - **Solution**: Migrated from `{@html}` inline rendering to `<iframe>` with `srcdoc` attribute
  - Email content is now completely isolated in its own document context
  - Email CSS and HTML structure can no longer affect parent page layout
  - Automatic iframe height adjustment based on content size
  - Added security sandbox restrictions: `allow-same-origin allow-popups allow-popups-to-escape-sandbox`
  - HTML fragments (without DOCTYPE) are still wrapped with proper HTML structure and basic styling
  - Complete HTML documents are rendered as-is within the iframe isolation
  - Fixed issue where sidebar would disappear or page elements would be overridden by aggressive email styles
- **Email Body Detection**: Added smart detection to determine if email HTML is a complete document or fragment
  - Checks for presence of `<!DOCTYPE>` or `<html>` tags to identify document structure
  - Complete documents rendered directly without additional wrapper
  - HTML fragments wrapped with proper `<html>`, `<head>`, and `<body>` tags plus responsive CSS

### Technical Details
- Modified `src-tauri/src/commands/emails/fetch.rs`:
  - Added HTML document structure detection logic
  - Removed nested HTML wrapper for complete documents
  - Preserved wrapper for HTML fragments with improved responsive CSS
- Updated `src/routes/components/EmailBody.svelte`:
  - Replaced `{@html body}` with `<iframe srcdoc={body}>` for complete isolation
  - Added automatic height adjustment via `onload` event handler
  - Configured security sandbox to prevent malicious email scripts while allowing legitimate links
- This fix prevents any email content from affecting the application's UI, regardless of how aggressive the email's CSS or HTML structure might be

## [0.2.1] - 2025-10-26

### Changed
- **UI Architecture**: Migrated to official shadcn-svelte sidebar-09 nested sidebar pattern
  - Implemented three-layer layout following shadcn-svelte best practices
  - First sidebar (icon-only): Account switcher and folder navigation with icons
  - Second sidebar (medium width): Email list with search and filters
  - Main content area: Email body viewer with independent scrolling
  - Each section now scrolls independently without affecting others
  - Eliminated multiple scrollbars issue on main content area
- **Sidebar Components**: Complete reorganization using nested Sidebar.Root pattern
  - `AccountFolderSidebar`: Icon-only left sidebar with account dropdown and folder list
  - `EmailListSidebar`: Middle sidebar with email list, compose button, and unread filter
  - EmailBody now properly constrained within Sidebar.Inset with flex layout
  - Proper z-index layering for header elements
- **Email List Display**: Optimized content layout and typography
  - Removed truncate classes that caused content cutoff with inline badges
  - Sender names and subjects display fully without being cut off by CC/attachment icons
  - Font sizes match official shadcn-svelte examples (text-sm for main content, text-xs for metadata)
  - Improved spacing between inline elements (badges, icons) with gap-1.5
  - Unread indicator (blue dot) properly aligned with sender name
- **Scrolling Behavior**: Fixed independent scrolling for each UI section
  - Email list scrolls independently within second sidebar
  - Email body scrolls independently within main content area
  - Scrolling email body no longer affects email list or overall page
  - Proper overflow constraints with flex-1 and overflow-hidden containers

### Added
- **Unread Filter**: Toggle switch in email list header to show only unread emails
  - Labeled "Unreads" with Switch component
  - Filters email list in real-time without server requests
  - Persistent state during navigation
- **NavUser Component**: Official shadcn-svelte user account menu in sidebar footer
  - Avatar display with user initials fallback
  - Dropdown menu with account management options
  - Professional styling matching shadcn design patterns

### Fixed
- **Layout Issues**: Resolved multiple visual and functional problems
  - Fixed extra scrollbar appearing on right side of email body
  - Corrected email list content being truncated/cut off
  - Resolved cascading scroll behavior where scrolling one section affected others
  - EmailBody now uses flex-1 with overflow-hidden instead of h-screen
  - Proper flex layout hierarchy: Sidebar.Inset > flex flex-col > flex-1 overflow-hidden > EmailBody

### Improved
- **Component Organization**: Better separation of concerns following shadcn patterns
  - AccountFolderSidebar handles account/folder navigation only
  - EmailListSidebar handles email browsing and filtering
  - Clear props interface between components
  - Reduced component coupling for better maintainability

## [0.2.0] - 2025-10-25

### Changed
- **UI Framework**: Migrated entire frontend to shadcn-svelte component library with Tailwind CSS
  - Replaced all custom CSS components with professional shadcn-svelte components
  - Integrated Tailwind CSS for consistent, modern styling
  - Improved component architecture with better composition and reusability
  - Enhanced dark mode support with proper theming
  - Better accessibility support built into components
- **Layout Architecture**: Complete redesign with official shadcn-svelte Sidebar component
  - Implemented professional 3-column layout: Sidebar | Email List | Email Content
  - Left sidebar with collapsible navigation and account management
  - Dropdown menu for account selection with visual indicators
  - Folder list integrated directly into sidebar (no separate column)
  - Middle column (400px) dedicated to email list with header controls
  - Right column (flexible) for email content viewing
  - Responsive keyboard shortcuts (Cmd/Ctrl+B to toggle sidebar)
- **Typography**: Integrated Inter font family to match shadcn-svelte official design
  - Primary UI font: Inter with weights 400, 500, 600, 700
  - Fallback stack: System fonts for optimal cross-platform rendering
  - Consistent with shadcn-svelte documentation standards
- **Color Scheme**: Updated sidebar CSS variables to match official shadcn-svelte specification
  - Light mode: Pure grayscale values for cleaner neutral palette
  - Dark mode: Deep backgrounds with proper contrast ratios
  - Removed color tints for professional appearance
  - Updated text contrast in sidebar for better readability
- **Email List Density**: Optimized email card spacing for higher information density
  - Reduced vertical padding from 4px to 2px per card
  - Maintained card spacing at 4px between items
  - Preserved font sizes and readability while showing more emails per screen
  - Total card height reduced to ~46px for compact display
- **Sync Mail Button**: Relocated global email sync control to sidebar footer
  - Renamed from "Refresh" to "Sync Mail" for clarity
  - Moved from header to left sidebar footer (above User Account menu)
  - Added RefreshCw icon with rotate animation during sync
  - Clearly indicates purpose: synchronize all accounts and folders
  - Improved accessibility with proper disabled states

### Added
- **Sidebar Component**: Official shadcn-svelte Sidebar with advanced features
  - `Sidebar.Provider` for state management
  - `Sidebar.Header` with account dropdown menu (Mail icon + email + status)
  - `Sidebar.Content` with scrollable folder list
  - `Sidebar.Footer` with user account menu (订阅方案/设置/登出)
  - Support for collapsible/expandable states
  - Proper hover states and active indicators
- **UI Components**: Implemented shadcn-svelte components across all views
  - `AccountsSidebar`: Button, ScrollArea, Badge, Separator, ButtonGroup (vertical orientation)
  - `FoldersSidebar`: Button, ScrollArea, Skeleton (loading state)
  - `EmailList`: Card, ScrollArea, Badge, Skeleton (loading state with 8 placeholder cards)
  - `EmailBody`: Card, Button, ScrollArea, Separator, ButtonGroup (action toolbar), Skeleton (loading state)
  - `ComposeDialog`: Dialog, Input, Textarea, Button, Label, Badge, ButtonGroup (footer actions)
  - `AttachmentList`: Button, Badge, Skeleton (loading state with 2 placeholder items)
  - `Settings Page`: Card, Input, Label, Separator
  - `Account Page`: Card, Input, Label, Button, ButtonGroup - OAuth2 and manual configuration forms
  - `Notification Window`: Tailwind CSS styling for toast notifications
- **New Dependencies**:
  - `lucide-svelte`: Modern icon library for Svelte
  - `dropdown-menu`: Dropdown component for account/user menus
  - `sheet`: Mobile responsive sheet component
  - `tooltip`: Tooltip component for better UX
- **Loading States**: Added Skeleton components for better loading UX
  - EmailList shows 8 skeleton cards while fetching emails
  - EmailBody displays skeleton for header, metadata, action buttons, and content
  - FoldersSidebar shows 6 skeleton items while loading folders
  - AttachmentList displays 2 skeleton items while fetching attachments
- **Button Groups**: Added official shadcn-svelte ButtonGroup component for better visual hierarchy
  - Grouped related actions (Reply/Forward, Compose/Refresh, etc.)
  - Consistent spacing and borders following shadcn design patterns
  - Improved accessibility with proper ARIA roles
- **Notifications**: Replaced custom toast with Sonner toast library for professional notifications
- **Typography**: Added Tailwind Typography for better email body rendering

### Improved
- **Design Consistency**: All UI elements now follow shadcn-svelte official design patterns
  - Refined button sizes and spacing to match shadcn documentation
  - Improved visual hierarchy with proper font weights and colors
  - Better hover states and focus indicators
  - Consistent use of muted colors for secondary information
- **Email List Compactness**: Achieved mobile-app-like density without sacrificing readability
  - Card vertical padding reduced to 2px (from previous 4px)
  - Inter line spacing optimized to 2px between sender and subject
  - Maintains full card styling with rounded corners and borders
  - Space-efficient layout shows ~30% more emails per screen
- **Sidebar Layout**: Enhanced sidebar footer organization
  - Sync Mail button prominently positioned above user menu
  - Clear visual separation with separator line
  - Improved button hierarchy and accessibility
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
