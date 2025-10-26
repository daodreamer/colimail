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

## [0.2.4] - 2025-10-26

### Added
- **Manage Account Dialog**: New centralized account management interface
  - Added "Manage Account" option in account dropdown menu (below "Add Account")
  - Uses `CircleUserRound` icon from Lucide Svelte for better visual representation
  - Follows shadcn-svelte design patterns with sidebar navigation layout
  - View all configured email accounts in a sidebar list
  - View detailed account information including authentication type, IMAP/SMTP servers
  - Edit account configuration for Basic authentication accounts
  - OAuth2 accounts show informative message that configuration cannot be edited manually
  - Delete accounts with confirmation dialog
  - Auto-selects first account when dialog opens for better UX
  - Uses Breadcrumb navigation showing current selected account

### Improved
- **Delete Account Confirmation**: Enhanced delete confirmation with shadcn-svelte Alert Dialog
  - Replaced native confirmation dialog with styled Alert Dialog component
  - Features destructive variant Alert with red warning style
  - Shows `AlertCircle` icon for visual warning indication
  - Lists all data that will be permanently deleted:
    - Account configuration
    - All emails
    - All folders
    - All attachments
  - Clear "Cancel" and "Delete Account" action buttons
- **Delete Success Feedback**: Improved post-deletion user feedback
  - Replaced toast notification with prominent Alert component
  - Shows green success Alert with `CheckCircle2` icon
  - Displays deleted email address for confirmation
  - Auto-dismisses after 5 seconds
  - More visible and professional than previous toast notification

### Technical Details
- Created `src/routes/components/ManageAccountDialog.svelte`:
  - Implements shadcn-svelte sidebar layout pattern matching Settings dialog
  - Uses `Sidebar.Provider`, `Sidebar.Root`, `Sidebar.Content` for account list navigation
  - Each account shows email address and authentication type badge (OAuth2/Basic)
  - Main content area shows account details in card format with `bg-muted/50 rounded-xl p-6` styling
  - Dialog size: `md:max-w-[700px] lg:max-w-[900px]`, height: `600px`
  - Edit mode uses grid layout for IMAP/SMTP fields (2 columns)
  - Integrated Alert Dialog component for delete confirmation
  - Integrated Alert component for success/warning messages
  - Added `showDeleteDialog`, `showSuccessAlert`, and `deletedEmail` state management
- Updated `src/routes/lib/types.ts`:
  - Added `AuthType` type: `"basic" | "oauth2"`
  - Extended `AccountConfig` interface with OAuth2 fields:
    - `auth_type?: AuthType`
    - `access_token?: string`
    - `refresh_token?: string`
    - `token_expires_at?: number`
  - Changed `password` field to optional to support OAuth2 accounts
- Updated `src/routes/components/AccountFolderSidebar.svelte`:
  - Added `onManageAccounts` callback prop
  - Added "Manage Account" menu item with `CircleUserRound` icon
- Updated `src/routes/+page.svelte`:
  - Added `showManageAccountDialog` state
  - Added `handleAccountDeleted` function to handle account deletion
  - Added `handleAccountUpdated` function to refresh account list
  - Integrated ManageAccountDialog component with proper callbacks
- Installed new shadcn-svelte components:
  - `alert-dialog`: For delete confirmation
  - `alert`: For success/warning messages

## [0.2.3] - 2025-10-26

### Improved
- **Settings Dialog UI**: Redesigned settings interface following shadcn-svelte sidebar-13 official pattern
  - Migrated from full-page route to modal dialog for better UX
  - Implemented nested sidebar navigation with 5 sections: Notifications, Appearance, Language & region, Privacy & visibility, Advanced
  - Added breadcrumb navigation (Settings > [Current Section])
  - Settings now opens as centered dialog overlay on main UI
  - Main interface remains visible but dimmed when settings is open
  - Dialog size: 700-800px width (responsive), 500px max height
  - Left sidebar with icon-based navigation matching official examples
  - Content area with proper overflow handling and scrolling
  - Uses official Sidebar, Breadcrumb, and Dialog components from shadcn-svelte
  - Notifications section fully functional with sync interval and notification preferences
  - Other sections show placeholder UI (coming soon)
  - Consistent font sizes, spacing, and colors matching sidebar-13 specification
- **Add Account Dialog UI**: Converted account creation from full-page to modal dialog
  - Changed from route navigation (`/account`) to centered dialog overlay
  - Maintains all functionality: OAuth2 (Google/Outlook) and manual IMAP/SMTP configuration
  - Dialog width: 500px, centered on screen with overlay
  - Automatically closes on successful account addition
  - Triggers account list refresh and auto-selects first account if none selected
  - Main UI remains visible but dimmed during account setup
  - Improved user flow: users stay in context instead of navigating away
  - Uses Tabs component for OAuth2/Manual switching
  - Provider selection with visual icons and buttons
  - Form validation and error handling with toast notifications
- **Folder Display Consistency**: Unified folder name display across all UI components
  - Email list header now shows folder display name (e.g., "Important") instead of IMAP path (e.g., "[Gmail]/Important")
  - Folder names are now consistent between left sidebar and email list header
  - Improved visual consistency and user experience
- **Folder Icons**: Implemented official shadcn-svelte icon system for folder navigation
  - Migrated from generic folder icon to semantic icons based on folder type
  - `InboxIcon`: Inbox folders
  - `FileIcon`: Draft folders
  - `SendIcon`: Sent folders
  - `ArchiveXIcon`: Junk/Spam folders
  - `Trash2Icon`: Trash/Deleted folders
  - `FolderIcon`: Default icon for other folders
  - Smart icon selection based on folder name or display name patterns
  - Icons match the official shadcn-svelte sidebar-09 example design

### Added
- **Auto-Select First Account**: Application now automatically selects the first email account on startup
  - Eliminates the need for manual account selection after app launch
  - Immediately loads folders and emails for better user experience
  - Only applies when accounts exist and none is currently selected
- **Empty State UI**: Enhanced empty state when no accounts are configured
  - Replaced "Select an account" text with a centered add account button
  - Large square button with plus icon for intuitive account addition
  - Button uses dashed border with hover effects following shadcn design patterns
  - Direct navigation to account setup page on click
- **Compose Button Relocation**: Moved email composition button to more prominent location
  - Relocated from email list sidebar to left navigation sidebar
  - Now positioned above Sync Mail button in sidebar footer
  - Uses `PenSquare` icon for better visual recognition of compose action
  - Styled with primary color theme for high visibility
  - Includes hover effects and tooltip ("Compose Email")
  - Disabled state when no account is selected
- **Email Search Functionality**: Added real-time search capability to email list
  - Search input placed at top of email list sidebar (below folder name)
  - Placeholder text: "Type to search..." matching official shadcn design
  - Searches across email subject, sender (from), and recipient (to) fields
  - Case-insensitive partial matching for flexible searching
  - Works seamlessly with "Unreads" filter - both can be active simultaneously
  - Shows "No emails match your search" when no results found
  - Real-time filtering as user types with no lag

### Fixed
- **Settings Menu Item**: Restored missing Settings option in user account menu
  - Re-added Settings menu item below Notifications in user dropdown
  - Uses `SettingsIcon` (gear icon) for clear identification
  - Properly connects to existing Settings page (`/settings`)
  - Menu now includes: Account, Billing, Notifications, Settings, and Log out

### Technical Details
- Created `src/routes/components/SettingsDialog.svelte`:
  - Implements shadcn-svelte sidebar-13 pattern with nested sidebar navigation
  - Uses `Sidebar.Provider`, `Sidebar.Root`, `Sidebar.Content` for navigation structure
  - Implements `Sidebar.Menu`, `Sidebar.MenuItem`, `Sidebar.MenuButton` for menu items
  - Navigation items use Lucide icons: `BellIcon`, `PaintbrushIcon`, `GlobeIcon`, `LockIcon`, `SettingsIcon`
  - Breadcrumb component shows Settings > [Current Page] navigation path
  - Content area with `h-[480px]` fixed height and `overflow-y-auto` for scrolling
  - Settings content cards use `bg-muted/50 rounded-xl p-6` styling
  - Dialog configuration: `max-w-[700px] lg:max-w-[800px]`, `max-h-[500px]`, `trapFocus={false}`
  - Implements all notification settings with proper form controls and save functionality
- Created `src/routes/components/AddAccountDialog.svelte`:
  - Converted from full-page component to dialog-based component
  - Uses `Dialog.Root`, `Dialog.Content` with `max-w-[500px]` width
  - Card component with `border-0 shadow-none` to avoid double borders
  - Tabs component for OAuth2/Manual configuration switching
  - Provider selection buttons with SVG icons for Google and Outlook
  - Form handling with validation and error states
  - OAuth2 flow with browser redirect handling (`openUrl` renamed to avoid conflict with `open` prop)
  - Callbacks: `onOpenChange` for dialog state, `onAccountAdded` for parent notification
  - Auto-closes dialog on successful account creation
- Modified `src/routes/+page.svelte`:
  - Added `showSettingsDialog` and `showAddAccountDialog` state variables
  - Imported `SettingsDialog` and `AddAccountDialog` components
  - Changed `onSettings` callback from `window.location.href = '/settings'` to `showSettingsDialog = true`
  - Changed `onAddAccount` callback from `window.location.href = '/account'` to `showAddAccountDialog = true`
  - Added `handleAccountAdded()` function to reload accounts and auto-select first account
  - Integrated both dialogs with proper state binding and callbacks
- Modified `src/routes/+page.svelte`:
  - Added auto-selection logic in `onMount()` lifecycle hook
  - Automatically calls `handleAccountClick()` for first account if available
  - Updated `AccountFolderSidebar` to receive `onComposeClick` callback
  - Removed `onComposeClick` from `EmailListSidebar` props
- Updated `src/routes/components/EmailListSidebar.svelte`:
  - Added `folders` prop to access folder metadata
  - Created `currentFolderDisplayName` derived state to resolve display name
  - Changed header to display `display_name` instead of raw `name` field
  - Removed Compose button from email list header
  - Added `searchQuery` state variable for search functionality
  - Implemented `filteredEmails` derived function with dual filtering (unreads + search)
  - Added `Sidebar.Input` component for search functionality
  - Enhanced empty state handling for search results
- Enhanced `src/routes/components/AccountFolderSidebar.svelte`:
  - Imported all folder-specific icons from lucide-svelte
  - Implemented `getFolderIcon()` function for intelligent icon selection
  - Uses `@const` directive to compute icon component per folder in loop
  - Supports pattern matching on both `name` and `display_name` fields
  - Added `onComposeClick` prop and callback handler
  - Added Compose button with `PenSquare` icon in sidebar footer
  - Positioned Compose button above Sync Mail button
  - Applied primary color styling for visual prominence
  - Added `onSettings` callback pass-through to `NavUser` component
- Updated `src/lib/components/nav-user.svelte`:
  - Imported `SettingsIcon` from lucide-svelte
  - Added optional `onSettings` callback prop
  - Added Settings menu item with conditional rendering
  - Positioned Settings below Notifications, above Log out separator

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
