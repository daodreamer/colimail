# Changelog

All notable changes to Colimail will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Log viewer UI interface in settings
- Log export functionality (ZIP download)
- Log search and filtering in UI
- Calendar integration
- Multi-language support
- Account settings dialog for user profile management
- Subscription/billing management interface
- Notifications settings dialog

## [0.6.1] - 2025-11-08

### Added
- **🎉 Auto-Update System**: Implemented automatic application updates for seamless version upgrades
  - Integrated Tauri updater plugin with GitHub Releases as update source
  - Automatic update check on application startup (3 seconds delay)
  - Manual update check available in Settings → About section
  - Cryptographic signature verification for secure updates
  - One-click download and install with automatic restart
  - Update notifications with release notes display
  - Support for all platforms (Windows, macOS, Linux)
  - GitHub Actions workflow configured for automatic signing and artifact generation
  - **Note**: Users on v0.6.0 need to manually update to v0.6.1 once. After that, all future updates will be automatic.

- **Settings Dialog: About Section**: New "About" page in Settings
  - Display current application version
  - Link to GitHub repository
  - "Check for Updates" button for manual update checks
  - Auto-update information and status
  - Clean, modern UI with Info icon in sidebar navigation

### Fixed
- **ComposeDialog Positioning**: Fixed dialog centering issue with Tailwind CSS v4
  - Resolved positioning bug where Compose dialog appeared incorrectly positioned (at top of viewport) on initial open
  - Fixed Tailwind CSS v4 compatibility issue with `translate-x-[-50%]` and `translate-y-[-50%]` utilities in Chrome
  - Replaced buggy Tailwind translate utilities with standard CSS `transform: translate(-50%, -50%)`
  - Dialog now consistently appears centered regardless of whether opened via Compose, Reply, or Forward buttons
  - Related to known Tailwind CSS v4 issue affecting bits-ui/shadcn-svelte Dialog components
  - References: [shadcn-svelte#1647](https://github.com/huntabyte/shadcn-svelte/issues/1647), [shadcn-ui#7507](https://github.com/shadcn-ui/ui/issues/7507)

- **Update Check Error Handling**: Improved logging for update checks
  - Downgraded "no release found" errors from ERROR to DEBUG level
  - Added friendly messages for expected failures (e.g., when no releases exist yet)
  - Better error messages in UI when update checks fail

### Changed
- **ComposeDialog Rich Text Toolbar**: Upgraded editing controls for a clearer, more capable compose experience
  - Replaced legacy glyph buttons with lucide icons and highlighted active formatting states
  - Added inline text and highlight color pickers that respect the current selection and sync with external updates

### Technical
- Added `tauri-plugin-updater` dependency
- Generated signing keypair for update verification (`colimail.key` and `colimail.key.pub`)
- Configured updater endpoint: `https://github.com/daodreamer/colimail/releases/latest/download/latest.json`
- Enabled `createUpdaterArtifacts` in Tauri build configuration
- Added `TAURI_SIGNING_PRIVATE_KEY` support in GitHub Actions workflow
- Protected signing key from git commits (added to `.gitignore`)
- Installed `@tauri-apps/plugin-process` for application relaunch after updates

## [0.6.0] - 2025-11-01

### Fixed
- **Google OAuth Flow**: Resolved critical OAuth authentication issues for desktop app
  - Fixed system browser OAuth callback handling with deep link protocol
  - Resolved authentication deadlock when calling `getSession()` in `onAuthStateChange` callback
  - Prevented duplicate OAuth event processing with proper state management
  - Disabled automatic URL detection in Tauri environment to avoid conflicts
  - Configured proper `skipBrowserRedirect` setting for desktop OAuth flow
  - Improved callback page to show success message after triggering deep link
  - Enhanced single-instance handling for OAuth deep links
  - Added comprehensive logging throughout OAuth flow for debugging

### Changed
- OAuth callback now uses HTTP URL (`https://www.colimail.net/auth/callback`) with deep link trigger
- Updated `getCurrentUser()` to accept optional session parameter, avoiding redundant API calls
- Improved `authStore.refreshUser()` to accept optional session for better performance
- Enhanced deep link event listeners with duplicate processing prevention

## [0.5.0] - 2025-10-31

### Added
- **User Authentication System**: Complete Supabase-based authentication for cloud features
  - **Email/Password Registration**: Users can create accounts with email and password
    - Email verification required via confirmation link sent to user's inbox
    - User name support for personalized experience (replaces display_name to avoid confusion with email account display names)
    - Password strength validation: requires lowercase, uppercase, digits, and symbols (8-72 characters)
    - Secure password handling with Supabase authentication
    - Multi-field metadata storage: Sets `name`, `full_name`, and `display_name` for Supabase Dashboard compatibility
  - **Email/Password Login**: Existing users can sign in with credentials
    - Session persistence using localStorage for automatic re-login
    - Manual session refresh for reliable state synchronization
    - Detailed debug logging for troubleshooting authentication issues
  - **Google OAuth Login**: One-click sign in with Google account
    - OAuth 2.0 flow with proper callback handling
    - Automatic user profile creation from Google account data
    - Explicitly requests email scope for Google Suite accounts compatibility
    - Supports both browser and desktop app OAuth callback flows
  - **Session Management**: Robust token handling and persistence
    - Automatic token refresh for long-lived sessions
    - Local session storage with secure token management
    - Cross-tab synchronization via Supabase auth state changes
  - **User Profile**: Displays user information in NavUser component
    - Shows user's display name and email address
    - Avatar support (defaults to user initials)
    - Subscription tier displayed (Free/Pro/Enterprise)
  - **Guest Mode**: App fully functional without authentication
    - All email management features available to guest users
    - Authentication optional - only required for cloud sync and Pro features
    - NavUser shows "Guest" state with Sign In/Create Account options

- **Authentication State Management**: Svelte 5 reactive auth store
  - **AuthStore Class** (`src/lib/stores/auth.svelte.ts`):
    - Reactive state with `$state` runes for user, session, loading status
    - `isAuthenticated` derived getter for checking login state
    - Automatic session detection and user data loading on app startup
    - `onAuthStateChange` listener for real-time auth updates from Supabase
    - `refreshUser()` method for manual state synchronization
    - Sync user data to local SQLite database for offline access
  - **Session Storage**: Custom storage adapter for Supabase
    - Priority: localStorage (reliable, no size limits)
    - Fallback: Tauri secure storage (for smaller sensitive data)
    - Fixes Windows Credential Manager 2560-character limit issue
    - Auto-saves session tokens for persistent login across app restarts
  - **Deep Link Support**: OAuth callback handling via custom URL scheme
    - Listens for `colimail://` deep link events (prepared for future use)
    - Redirects OAuth callbacks to `/auth/callback` page for processing
    - Configured in `tauri.conf.json` with deep-link plugin

- **Authentication UI Components**:
  - **Login Form** (`src/lib/components/login-form.svelte`):
    - Email/password input fields with validation
    - "Login with Google" button with Google logo
    - "Forgot password?" link for password reset
    - Real-time error display for failed login attempts
    - Success feedback with automatic redirect to main app
    - Manual auth store refresh after login for immediate UI update
  - **Signup Form** (`src/lib/components/signup-form.svelte`):
    - Email, password, and display name input fields
    - Password confirmation with real-time validation
    - "Sign up with Google" option for OAuth registration
    - Email verification reminder after successful registration
    - Error handling for duplicate accounts and invalid inputs
  - **Login Page** (`src/routes/auth/login/+page.svelte`):
    - Centered modal layout with app branding
    - "Back to App" button for easy navigation
    - Consistent with shadcn-svelte design patterns
  - **Signup Page** (`src/routes/auth/signup/+page.svelte`):
    - Full-screen responsive layout for registration flow
    - "Back to App" button in top-left corner
    - Professional card-based design
  - **Callback Page** (`src/routes/auth/callback/+page.svelte`):
    - Handles OAuth redirects from Google authentication
    - Environment detection: Shows different UI for browser vs desktop app
    - Browser environment: Displays friendly message with instructions to open desktop app
    - Desktop app environment: Automatically completes authentication and redirects to main app
    - Attempts to trigger deep link (`colimail://`) to open desktop app from browser
    - Success/error states with visual feedback (checkmark/X icon)
    - Forces auth store refresh to ensure UI updates immediately
  - **Verification Page** (`src/routes/auth/verify/+page.svelte`):
    - Dedicated page for email verification workflow
    - Handles email confirmation links from Supabase
    - Environment-aware processing: Different flows for browser vs desktop app
    - Browser environment: Shows "Email Verified!" success page with instructions
    - Desktop app environment: Auto-completes verification and signs user in
    - Automatic deep link attempt to open desktop app from browser
    - Clear success/error messaging with visual feedback
    - Graceful error handling with user-friendly messages

- **NavUser Enhancement**: Dynamic menu based on authentication state
  - **Authenticated Menu**: Full feature access for logged-in users
    - User profile display with name, email, and avatar
    - "Upgrade to Pro" option (placeholder for future subscription system)
    - Account, Billing, Notifications, Settings menu items
    - "Log out" option at bottom of menu
  - **Guest Menu**: Limited options for non-authenticated users
    - Welcome message: "Welcome to Colimail - Sign in to unlock Pro features"
    - "Sign In" button navigating to login page
    - "Create Account" button navigating to signup page
    - Settings option still available for guest users (email account configuration)
  - **Visual Indicators**: Clear distinction between auth states
    - Authenticated: Shows user avatar with initials or profile picture
    - Guest: Shows generic user icon with "Guest" label

- **Backend Auth Commands** (`src-tauri/src/commands/auth.rs`):
  - `get_secure_storage(key)`: Retrieve auth tokens from OS keyring or localStorage
  - `set_secure_storage(key, value)`: Store auth tokens securely
  - `delete_secure_storage(key)`: Remove auth tokens on logout
  - `sync_app_user(...)`: Sync Supabase user data to local SQLite database
  - `get_app_user(user_id)`: Retrieve user profile from local database
  - `delete_app_user(user_id)`: Delete user data from local database

- **Database Schema** (`src-tauri/src/db.rs`):
  - Added `app_user` table for storing user profiles locally:
    - `id TEXT PRIMARY KEY`: Supabase user UUID
    - `email TEXT NOT NULL UNIQUE`: User's email address
    - `name TEXT`: User's name/username (changed from display_name to avoid confusion)
    - `avatar_url TEXT`: Profile picture URL (optional)
    - `subscription_tier TEXT`: free/pro/enterprise (default: 'free')
    - `subscription_expires_at INTEGER`: Unix timestamp for Pro subscription expiry
    - `last_synced_at INTEGER`: Last sync time with Supabase
    - `created_at INTEGER`: Account creation timestamp

- **Comprehensive Documentation** (`AUTH_SETUP_GUIDE.md`):
  - Complete setup guide for Supabase and Google OAuth configuration
  - Step-by-step Google Cloud Console setup instructions
  - Troubleshooting section with common issues and solutions
  - Debug logging guide for diagnosing authentication problems
  - Security best practices and credential management tips
  - Testing procedures for all authentication flows

### Fixed
- **Email Verification Browser Experience**: Resolved confusing error page when clicking verification links
  - **Problem**: Clicking email verification link opened browser showing JavaScript error "Cannot read properties of undefined (reading 'invoke')"
  - **Root Cause**: Verification links redirect to `http://localhost:1420/?code=...` which tried to call Tauri APIs in browser environment
  - **Solution**: Implemented dedicated `/auth/verify` page with environment detection
    - Browser environment: Shows professional "Email Verified!" success page with clear instructions
    - Desktop app environment: Auto-completes verification and signs user in
    - Attempts automatic deep link to open desktop app from browser
    - Added `emailRedirectTo` parameter to signup flow directing to `/auth/verify`
  - **Impact**: Professional, user-friendly verification experience matching mainstream applications
  - **User Experience**: Clear guidance to open desktop app and sign in after verification

- **Google OAuth Redirect URI Configuration**: Fixed "Error 400: redirect_uri_mismatch" for Google OAuth
  - **Problem**: Users couldn't sign in with Google OAuth due to redirect URI mismatch
  - **Root Cause**: Missing redirect URI configuration in Google Cloud Console
  - **Solution**: Added proper redirect URIs in Google Cloud Console OAuth client:
    - `https://[project-id].supabase.co/auth/v1/callback` for Supabase OAuth flow
    - `http://localhost:1420/auth/callback` for local development
  - **Configuration**: Requires separate OAuth clients for different purposes (Desktop for Gmail API, Web for authentication)
  - **Impact**: Google OAuth now works seamlessly for both signup and login

- **Supabase Dashboard Display Name**: Fixed missing display name in Supabase Dashboard user list
  - **Problem**: Supabase Dashboard "Display name" column showed empty despite `raw_user_meta_data` containing user's name
  - **Root Cause**: Dashboard searches for specific field names (`display_name`, `full_name`) not just `name`
  - **Solution**: Updated signup to set multiple metadata fields simultaneously:
    - `name`: Primary field for application use
    - `full_name`: For Supabase Dashboard compatibility
    - `display_name`: For Supabase Dashboard "Display Name" column
  - **Fallback Logic**: `getCurrentUser()` reads from multiple fields with priority: name → full_name → display_name → email username
  - **Impact**: User names now display correctly in Supabase Dashboard for better admin experience

- **Password Validation**: Implemented comprehensive password strength requirements matching Supabase configuration
  - **Added Validation**: `validatePassword()` function checks all requirements before signup
    - Minimum 8 characters, maximum 72 characters
    - At least one lowercase letter (a-z)
    - At least one uppercase letter (A-Z)
    - At least one digit (0-9)
    - At least one symbol (!@#$%^&*...)
  - **User Feedback**: Clear error messages for each validation failure
  - **Impact**: Prevents signup failures due to weak passwords, improves security

- **Windows Credential Manager Size Limit**: Resolved session storage failures
  - **Problem**: Windows limits credentials to 2560 characters (UTF-16 encoded)
  - **Impact**: Supabase session tokens typically exceed this limit (3000+ chars)
  - **Error**: "Failed to store value in keyring: Attribute 'password' is longer than platform limit"
  - **Solution**: Changed storage strategy to prioritize localStorage over keyring
    - localStorage: No size limits, stored in Tauri's local data directory
    - Location: `%APPDATA%\com.colimail.app\webview\localStorage`
    - Security: Still isolated per-app, inaccessible to other applications
  - **Impact**: Session persistence now works reliably on Windows

- **Login State Synchronization**: Fixed UI not updating after successful login
  - **Problem**: User logs in successfully but NavUser still shows "Guest" state
  - **Root Cause**: AuthStore not refreshing immediately after login/callback
  - **Solution**: Added explicit `authStore.refreshUser()` calls after authentication
    - Login form calls refresh after `signInWithEmail()` succeeds
    - Callback page calls refresh after session detection
    - 500ms delay before refresh to allow Supabase session propagation
  - **Enhanced Debugging**: Added detailed console logs throughout auth flow
    - `[AuthStore]` prefix for auth store operations
    - `[Callback]` prefix for OAuth callback processing
    - `[Login]` prefix for login form operations
    - `[Supabase]` prefix for Supabase API calls
    - Logs show session state, user data, and `isAuthenticated` status at each step

- **Logout Behavior**: Fixed logout redirecting to login page instead of staying in app
  - **Previous**: Clicking "Log out" redirected to `/auth/login`
  - **Current**: Clicking "Log out" stays on main UI
  - **Rationale**: Email client functionality works without authentication
  - **UX**: Shows success toast, NavUser updates to guest menu automatically
  - **Implementation**: Removed `goto("/auth/login")`, added `toast.success()` notification

### Improved
- **Email Verification Workflow**: Enhanced user experience to match mainstream application standards
  - **Smart Environment Detection**: Different flows for browser vs desktop app
  - **Professional Browser UI**: Elegant success page with clear instructions when opened in browser
  - **Automatic Deep Links**: Attempts to open desktop app automatically via `colimail://` protocol
  - **Clear User Guidance**: Step-by-step instructions to complete signup (open app → sign in)
  - **Consistent Design**: Follows shadcn-svelte design patterns with proper icons and styling
  - **Error Handling**: Graceful error messages with helpful troubleshooting information

- **OAuth Callback Handling**: Improved reliability and user experience for OAuth flows
  - **Environment-Aware Processing**: Detects browser vs desktop app context
  - **Browser Fallback**: Shows friendly instructions when OAuth completes in browser
  - **Desktop Integration**: Seamless authentication completion in desktop app
  - **Visual Feedback**: Clear loading, success, and error states with appropriate icons
  - **Error Messages**: User-friendly error descriptions with recovery suggestions

- **Supabase Configuration**: Simplified setup process with better documentation
  - **Email Redirect Configuration**: Automatic `emailRedirectTo` parameter in signup
  - **Multi-Field Metadata**: Compatibility with Supabase Dashboard display requirements
  - **Flexible User Data**: Supports multiple name field conventions for broad compatibility
  - **Development-Friendly**: Easy local testing with localhost redirect URLs

### Changed
- **User Profile Field Naming**: Renamed from `display_name` to `name` for clarity
  - **Rationale**: Avoid confusion with email account display names (sender names in emails)
  - **Application Field**: `name` - user's name/username in the application
  - **Email Account Field**: `display_name` - sender name shown in outgoing emails
  - **Clear Separation**: Distinct naming prevents mixing user identity with email sender identity
  - **Database Migration**: Updated `app_user` table schema and all related code
  - **Supabase Compatibility**: Still sets `full_name` and `display_name` in metadata for Dashboard

- **Authentication Architecture**: Dual-layer system for app users vs email accounts
  - **App Users** (new): Stored in Supabase cloud database
    - Managed via Supabase authentication API
    - Profile data synced to local SQLite for offline access
    - Controls cloud features, subscriptions, and sync
  - **Email Accounts** (existing): Stored locally in SQLite
    - IMAP/SMTP credentials remain in secure storage
    - Not affected by app user authentication
    - Users can manage email accounts as guest or authenticated

- **Supabase Client Configuration** (`src/lib/supabase.ts`):
  - Custom storage adapter using localStorage + Tauri secure storage
  - Automatic token refresh enabled for seamless session management
  - Session persistence enabled for cross-restart login
  - URL-based session detection for OAuth callbacks
  - Helper functions: `signUpWithEmail`, `signInWithEmail`, `signInWithGoogle`, `signOut`

### Technical Details
- **Dependencies Added**:
  - `@supabase/supabase-js`: Supabase client library for authentication
  - Integration with existing Tauri secure storage system

- **Supabase Configuration**:
  - Project URL: Configured via `VITE_SUPABASE_URL` environment variable
  - Anon Key: Configured via `VITE_SUPABASE_ANON_KEY` environment variable
  - OAuth Callback: `http://localhost:1420/auth/callback` (development)
  - Email Verification Redirect: `http://localhost:1420/auth/verify` (development)
  - Redirect URLs Configuration Required:
    - `http://localhost:1420/auth/callback` - OAuth callback
    - `http://localhost:1420/auth/verify` - Email verification
    - `colimail://auth/callback` - Deep link for OAuth (optional)
    - `colimail://auth/verify` - Deep link for verification (optional)
  - Production callback: Configurable via Supabase dashboard

- **Email Verification Implementation**:
  - `signUpWithEmail()` function sets `emailRedirectTo: ${window.location.origin}/auth/verify`
  - Supabase email template uses default `{{ .ConfirmationURL }}` variable
  - `/auth/verify` page handles token verification automatically via Supabase client
  - Environment detection: `typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window`
  - Browser flow: Display success UI + attempt deep link + show instructions
  - Desktop flow: Complete verification + refresh auth store + redirect to main app
  - Deep link format: Replaces `http://localhost:1420` with `colimail://`

- **Password Validation Requirements**:
  - Implementation: `validatePassword(password: string): { valid: boolean; message: string }`
  - Length: 8-72 characters (PostgreSQL bcrypt limit)
  - Character types: lowercase (a-z), uppercase (A-Z), digits (0-9), symbols
  - Symbol regex: `/[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>\/?]/`
  - UI integration: Real-time validation in signup form before submission
  - Error messages: Specific feedback for each validation failure

- **User Metadata Storage Strategy**:
  - Three metadata fields set during signup for maximum compatibility:
    - `name`: Primary field used by application (`AppUser.name`)
    - `full_name`: Supabase Dashboard compatible field
    - `display_name`: Supabase Dashboard "Display Name" column
  - All three fields set to the same value during registration
  - `getCurrentUser()` fallback priority: name → full_name → display_name → email username
  - Ensures display in Supabase Dashboard regardless of field naming preferences

- **Debug Logging System**: Comprehensive logging for troubleshooting
  - AuthStore initialization: Session detection, user loading, listener registration
  - Auth state changes: SIGNED_IN, SIGNED_OUT events with user email
  - Manual refresh operations: Session fetching, user data loading, database sync
  - Callback processing: Session detection, token extraction, redirect handling
  - Login operations: Email login attempts, success/failure, refresh triggers
  - Supabase operations: Sign out, getCurrentUser, session retrieval

- **Security Considerations**:
  - Session tokens stored in localStorage (sandboxed per Tauri app)
  - OAuth client secrets never exposed to client-side code
  - Email verification required for password-based registration
  - Rate limiting enforced by Supabase (6 requests per hour for some endpoints)
  - Deep link handler prepared but not yet active (manual callback for now)

- **Code Quality**:
  - All TypeScript code validated with `svelte-check` (0 errors, 0 warnings)
  - Proper error handling with try-catch blocks throughout auth flow
  - User-friendly error messages displayed in UI
  - Console logging for debugging without exposing sensitive data

### Migration Guide
For existing users upgrading to 0.5.0:
1. No data loss - existing email accounts and emails preserved
2. App user authentication is optional - continue using as guest
3. To enable cloud features in future: Create account via "Create Account" button
4. Existing settings and preferences remain unchanged

## [0.4.4] - 2025-10-31

### Added
- **Production Logging System**: Implemented comprehensive logging system for production environments
  - **Automatic Log Rotation**: Daily log files with automatic 7-day retention
  - **JSON Format**: Structured logging with timestamps, levels, modules, line numbers, and thread IDs
  - **Multiple Log Levels**: DEBUG, INFO, WARN, ERROR with appropriate filtering
  - **Platform-Specific Storage**: Logs stored in OS-specific user data directories
    - Windows: `%APPDATA%\Colimail\logs\`
    - macOS: `~/Library/Application Support/Colimail/logs/`
    - Linux: `~/.local/share/Colimail/logs/`
  - **Backend Commands**:
    - `get_log_directory()`: Returns path to log directory
    - `get_current_log_file()`: Returns path to today's log file
    - `read_recent_logs(lines)`: Read last N lines from current log
    - `list_log_files()`: List all available log files
    - `read_log_file(filename)`: Read specific log file content
  - **Structured Logging**: Uses `tracing` crate with field-based logging
    - Example: `tracing::info!(account_id = id, email = %email, "Account loaded")`
  - **Performance Optimized**: Async file I/O with minimal overhead (< 1% CPU)
  - **Security Aware**: Never logs passwords or email content, only metadata
  - **UI Integration**: LogViewer component for viewing logs in the application
  - **Development Mode**: Console logging with pretty formatting for debugging
  - **Production Mode**: File-only logging to reduce overhead
- **System Tray Integration**: Implemented complete system tray functionality for Windows
  - Application minimizes to system tray instead of taskbar when window is closed
  - Left-click tray icon: Toggle window visibility (show/hide)
  - Right-click tray icon: Display context menu with Settings and Quit options
  - Settings option navigates to application settings (opens Settings dialog)
  - Quit option completely exits the application
  - Configurable behavior: Users can choose between "Close to system tray" or "Exit application" in Advanced settings
  - **Close Button Behavior**:
    - Default: Clicking window close button (X) hides window to system tray
    - Optional: Can be configured to exit application directly
  - **Menu Display Fix**: Left-click no longer shows context menu (menu only appears on right-click)
  - Settings location: Settings dialog → Advanced tab → System tray section
  - Setting name: "Close to system tray" (simple, clear naming following industry standards)
  - Helpful description explains behavior and how to restore window
  - Warning message when disabled: "Closing the window will exit the application completely"
- **Sender Display Name Support**: Implemented display name functionality for email sending
  - Recipients now see sender's name (e.g., "John Doe") instead of just email address
  - Display name field added to account configuration (optional)
  - Works for both manual and OAuth2 account setups
  - Smart detection from sent emails folder on first account setup
  - Automatic suggestion dialog when display name is detected from existing sent emails
  - Manual "Detect" button in account settings to auto-fill from sent emails
  - Supports multiple languages and special characters in display names
  - IMAP folder detection improved with attribute checking (RFC 6154 \Sent flag)
  - Fallback to pattern matching for servers without special-use folder support
  - Display name properly formatted in From header: "Display Name <email@example.com>"
  - Applied to all email sending operations: compose, reply, and forward

### Changed
- **Notification System Refactored**: Migrated from custom notification windows to native system notifications
  - Replaced custom Tauri window-based notifications with native OS notifications (Windows Action Center)
  - Notifications now appear in system desktop notification area instead of application window
  - Improved user experience with system-native notification styling
  - Better integration with Windows notification settings and focus assist
  - Notification text changed to English for better consistency

### Fixed
- **IDLE Connection Stability**: Fixed IDLE manager reconnection issues
  - IDLE sessions now continue running after detecting new emails instead of disconnecting
  - Removed 30-second reconnection delay that caused missed notifications
  - Improved event listener cleanup in frontend for better resource management
  - Added notification permission checks and request handling on app startup
- **CRITICAL: Account ID Preservation**: Fixed account ID being regenerated when updating display name or other account settings
  - **Root Cause**: `INSERT OR REPLACE` SQL statement was deleting old row and creating new row with new ID
  - **Impact**: After updating display name, sending emails failed with "Could not find selected account configuration" error
  - **Solution**: Changed from `INSERT OR REPLACE` to conditional `UPDATE` (when ID exists) or `INSERT` (when ID is null)
  - Account IDs now remain stable throughout account lifetime
  - Fixes email sending errors after any account modification
  - Ensures `selectedAccountId` consistently matches account configuration
- **OAuth2 Account Display Name Editing**: Enabled display name editing for OAuth2 accounts
  - Previously OAuth2 accounts were completely locked from editing
  - Now allows editing display name while protecting server settings
  - Shows clear UI message: "You can only edit the display name"
  - OAuth2 authentication tokens and server settings remain protected
  - Display name detection also works for OAuth2 accounts
- **Account State Synchronization**: Fixed account list not updating in UI after modifications
  - Proper async/await handling when reloading accounts after updates
  - ManageAccountDialog now correctly updates both local and global account state
  - Email composition uses fresh account data after any modifications

### Technical Details
- **System Tray Implementation** (`src-tauri/src/main.rs`):
  - **Tauri Features**: Added `tray-icon` feature to Tauri dependency in Cargo.toml
  - **Menu Creation**: Built context menu with `MenuItem::with_id()` for Settings and Quit items
  - **Tray Icon Builder**:
    - Used `TrayIconBuilder::new()` with application icon
    - `.menu(&menu)` attaches context menu for right-click
    - `.show_menu_on_left_click(false)` prevents menu on left-click
  - **Event Handlers**:
    - `on_menu_event()`: Handles Settings (emits "open-settings" event) and Quit (exits app) menu clicks
    - `on_tray_icon_event()`: Handles left-click to toggle window visibility
  - **Window Close Handler**:
    - Intercepts `WindowEvent::CloseRequested` event
    - Calls `api.prevent_close()` to prevent default close behavior
    - Checks `get_minimize_to_tray()` setting from database
    - If enabled: Hides window with `window.hide()`
    - If disabled: Exits application with `app.exit(0)`
  - **Settings Event**: Frontend listens for "open-settings" event to show SettingsDialog
- **Database Schema** (`src-tauri/src/db.rs`):
  - Added `minimize_to_tray` setting with default value `'true'`
  - SQL: `INSERT OR IGNORE INTO settings (key, value) VALUES ('minimize_to_tray', 'true')`
- **Settings Commands** (`src-tauri/src/commands/notifications.rs`):
  - `get_minimize_to_tray()`: Retrieves setting from SQLite database
  - `set_minimize_to_tray(enabled: bool)`: Saves setting to database
  - Uses `INSERT OR REPLACE` pattern for atomic updates
- **Frontend Integration**:
  - **Event Listener** (`src/routes/+page.svelte`):
    - Added `listen("open-settings")` event listener in `onMount`
    - Sets `showSettingsDialog = true` when event received
    - Properly cleaned up with `unlisten()` on component unmount
  - **Settings Dialog** (`src/routes/components/SettingsDialog.svelte`):
    - Added `minimizeToTray` state variable (default: true)
    - Loads setting via `invoke("get_minimize_to_tray")` on dialog open
    - Saves setting via `invoke("set_minimize_to_tray")` with other preferences
    - UI in Advanced tab with checkbox, description, and conditional warning
  - **Naming Research**: Analyzed naming conventions from popular apps (Slack, Discord, Telegram, Spotify)
  - **Final Naming**: "Close to system tray" (concise and industry-standard)
- **Code Quality**:
  - All Rust code validated with `cargo fmt`, `cargo check`, and `cargo clippy -- -D warnings`
  - Frontend code validated with `svelte-check` (0 errors, 0 warnings)
  - Zero compilation warnings or type errors
- **Display Name Detection** (`src-tauri/src/commands/detect_display_name.rs`):
  - New command: `detect_display_name_from_sent` analyzes sent emails to extract display name
  - IMAP folder detection with two-tier strategy:
    1. Priority: Check folder attributes for RFC 6154 `\Sent` flag
    2. Fallback: Pattern matching on folder names in 12 languages
  - Supports multiple languages: English, Chinese, Japanese, French, Spanish, German, Italian, Swedish
  - Uses `mail-parser` crate to parse email From header and extract display name
  - Checks last 5 sent emails for display name detection
  - OAuth2 authentication support with `ensure_valid_token` integration
- **Account Configuration** (`src-tauri/src/models.rs`):
  - Added `display_name: Option<String>` field to `AccountConfig` struct
  - Serialized with `#[serde(skip_serializing_if = "Option::is_none")]` for clean JSON output
- **Database Schema** (`src-tauri/src/db.rs`):
  - Added `display_name TEXT` column to accounts table
  - Migration: `ALTER TABLE accounts ADD COLUMN display_name TEXT` for existing installations
  - Auto-migration runs on app startup, safe for both new and existing databases
- **Account Save Logic** (`src-tauri/src/commands/accounts.rs`):
  - **Critical Fix**: Replaced `INSERT OR REPLACE` with conditional logic
  - When `config.id` exists: Use `UPDATE` to preserve ID
  - When `config.id` is None: Use `INSERT` to create new account with auto-generated ID
  - Prevents ID regeneration that caused "account not found" errors
  - Display name included in both INSERT and UPDATE operations
- **Email Sending** (`src-tauri/src/commands/send.rs`):
  - Updated `send_email`, `reply_email`, and `forward_email` functions
  - From mailbox construction: `format!("{} <{}>", display_name, email)` when display_name exists
  - Fallback to email-only format when display_name is empty or None
  - Uses `lettre::message::Mailbox::parse()` for RFC 5322 compliant formatting
- **Frontend Components**:
  - `AddAccountDialog.svelte`:
    - Added display_name input field with placeholder "Your Name (e.g., John Doe)"
    - Auto-detection after account creation for both manual and OAuth2 flows
    - Suggestion dialog with blue info box showing detected name
    - "Use This Name" / "Skip" options for user choice
  - `ManageAccountDialog.svelte`:
    - Added display_name input field with "Detect" button
    - OAuth2 accounts: Only display_name editable, other fields hidden/disabled
    - Clear UI message for OAuth2 accounts explaining limitations
    - Fixed account reload logic to preserve selectedAccount after updates
  - State management uses Svelte 5 `$state` runes for reactivity
- **Code Quality**:
  - All Rust code formatted with `cargo fmt`
  - Compiled without errors via `cargo check`
  - Zero Clippy warnings with `cargo clippy -- -D warnings`
  - TypeScript/Svelte code validated with `svelte-check` (0 errors, 0 warnings)

## [0.4.3] - 2025-10-30

### Added
- **Folder Management**: Implemented complete folder creation and deletion functionality
  - **Create Folders**: Users can create new email folders for better organization
    - Smart server detection: Automatically checks if IMAP server supports remote folder creation
    - Remote folders: Synced with email server (Gmail, Outlook, etc.) when supported
    - Local folders: Client-only folders when server doesn't support folder creation
    - Visual indicator: Local folders display HardDrive icon to distinguish from remote folders
    - Dialog UI: Clean modal with folder name input and Enter key support
  - **Delete Folders**: Remove unwanted folders with confirmation
    - System folder protection: Prevents deletion of Inbox, Sent, Drafts, Trash, Junk/Spam
    - Hover UI: Delete button (X icon) appears on hover over folder items
    - Confirmation dialog: Clear warning with different messages for local vs remote folders
    - Smart navigation: Automatically switches to Inbox if deleted folder was selected
    - Bidirectional sync: Remote folder deletion syncs with email server
  - **New Folder Button**: Prominent "New Folder" button at bottom of folder list
  - **Backend Commands**:
    - `check_folder_capabilities`: Detects IMAP server support for folder operations
    - `create_remote_folder`: Creates folder on IMAP server and syncs to database
    - `delete_remote_folder`: Deletes folder from IMAP server and database
    - `create_local_folder`: Creates local-only folder in database
    - `delete_local_folder`: Deletes local folder from database
  - **Database Schema**: Added `is_local` column to folders table (INTEGER, default 0)
  - **Type Safety**: Extended `Folder` interface with `is_local?: boolean` field
  - **Auto-refresh**: Folder list automatically updates after creation/deletion
  - **Toast Notifications**: Success/error feedback for all folder operations

### Improved
- **Code Architecture Refactoring**: Major refactoring of `+page.svelte` for better maintainability and modularity
  - **Problem**: `+page.svelte` had grown to 1,668 lines, making it difficult to maintain and understand
  - **Solution**: Extracted handler functions into 5 focused modules, reducing main file to 576 lines (65% reduction)
  - **Handler Modules** (1,633 total lines organized by domain):
    - `handlers/email-operations.ts` (515 lines): Email viewing, reading/unread status, starring, deletion, attachments
    - `handlers/account-folder.ts` (336 lines): Account selection, folder navigation, email loading, sync logic
    - `handlers/sync-idle.ts` (293 lines): Auto-sync timers, IDLE push notifications, manual refresh
    - `handlers/compose-send.ts` (260 lines): Email composition, reply, forward, attachment handling, sending
    - `handlers/draft-management.ts` (229 lines): Draft creation, auto-save, loading, deletion
  - **Benefits**:
    - **Easier maintenance**: Each module has single responsibility, easier to locate and fix bugs
    - **Better readability**: Main page file now only contains wrapper functions and UI layout
    - **Improved testability**: Handler modules can be unit tested independently
    - **Clear separation of concerns**: Business logic separated from UI presentation layer
    - **No functionality changes**: All existing features work identically, only code organization improved
- **Context Menu System**: Implemented comprehensive right-click context menu functionality with global state management
  - **Global Context Menu State Management**: Ensures only one context menu can be open at a time across the entire application
    - Implemented centralized state in `+page.svelte`: `openContextMenuType` (folder/email/null) and `openContextMenuId`
    - All context menus (folders and emails) share this global state via `onContextMenuChange` callback
    - Opening any context menu automatically closes all others, preventing multiple overlapping menus
    - Clean architecture: parent component manages global state, child components use derived state to check if their menu is open
  - **Global Browser Context Menu Disabled**: Blocked browser's default right-click menu for professional appearance
    - Added global `contextmenu` event listener in `+layout.svelte` with `preventDefault()`
    - Ensures only custom shadcn-svelte ContextMenu appears throughout the application
  - **Email List Context Menu**: Added right-click menu for email items with contextually relevant actions
    - Right-click any email to access quick actions without selecting it first
    - Menu options dynamically adapt based on email state:
      - "Open Email" (Mail icon) - Opens the full email content
      - "Mark as Read" (Eye icon) - Shows only for unread emails
      - "Mark as Unread" (EyeOff icon) - Shows only for read emails
      - "Add/Remove Star" (Star icon) - Toggle flagged status
      - "Delete Email" (Trash2 icon, destructive styling) - Move to trash or permanently delete
    - Smart menu management: Only one context menu can be open at a time
    - Controlled component pattern: `openContextMenuUid` tracks currently open menu by email UID
    - Auto-closes previous menu when opening a new one
    - Menu closes automatically after action selection
    - All actions use optimistic UI updates for instant feedback
  - **Folder Context Menu**: Right-click menu for folder operations
    - Prevents accidental deletion when clicking folders
    - Right-click any folder to reveal context menu with "Delete Folder" option
    - Uses shadcn-svelte ContextMenu component with destructive styling
    - Trash icon clearly indicates delete action
    - System folders remain protected (no context menu for Inbox, Sent, etc.)
    - Better user experience: intentional right-click vs accidental hover click
    - Smart menu management: Only one context menu can be open at a time
    - Auto-closes previous menu when opening a new one
    - Menu closes automatically when clicking delete action
- **Delete Account Notification**: Migrated from inline Alert component to toast notification for better UX
  - Replaced prominent green Alert card with non-intrusive `toast.success()` notification
  - Uses same format as official shadcn-svelte toast example with title and description
  - Toast message: "Account deleted successfully" with full account details in description
  - Consistent notification style across all account management operations
  - Cleaner UI without auto-dismissing alert banners taking up space
  - Removed unused state variables (`showSuccessAlert`, `deletedEmail`)
  - Removed unused icon import (`CheckCircle2Icon`)

### Improved
- **Backend Code Organization**: Refactored email synchronization module for better maintainability
  - **Problem**: `sync.rs` had grown to 1183 lines, making it difficult to understand and maintain
  - **Solution**: Split into 6 focused sub-modules within `sync/` directory, reducing complexity and improving code organization
  - **Module Structure**:
    - `sync/mod.rs` (88 lines): Main entry point, coordinates sync operations and exports public commands
    - `sync/parse.rs` (156 lines): Email header parsing from IMAP FETCH responses
    - `sync/sync_core.rs` (287 lines): Core incremental sync algorithm and deletion detection
    - `sync/sync_fetch.rs` (326 lines): Batch fetching logic with adaptive sizing and reconnection
    - `sync/sync_flags.rs` (230 lines): Flag synchronization (read/starred status)
    - `sync/sync_state.rs` (106 lines): Sync state management (UIDVALIDITY, highest UID)
  - **Benefits**:
    - **Better encapsulation**: Internal modules are private, only public commands exposed
    - **Easier maintenance**: Each module has single responsibility (~100-300 lines)
    - **Improved readability**: Clear separation between fetching, parsing, state management, and flag sync
    - **Better testing**: Modules can be tested independently
    - **No functionality changes**: All existing sync features work identically
  - **Code Quality**: All changes validated with `cargo fmt`, `cargo check`, and `cargo clippy -- -D warnings`
- **IDLE Manager Code Organization**: Refactored IDLE connection manager for better maintainability
  - **Problem**: `idle_manager.rs` had grown to 770 lines, making it difficult to understand and maintain
  - **Solution**: Split into 5 focused sub-modules within `idle_manager/` directory, reducing complexity and improving code organization
  - **Module Structure**:
    - `idle_manager/mod.rs` (12 lines): Module entry point, exports public interfaces
    - `idle_manager/types.rs` (60 lines): Data type definitions and global state
    - `idle_manager/manager.rs` (240 lines): Core IDLE manager and command processing loop
    - `idle_manager/session.rs` (228 lines): IDLE session handling and connection retry logic
    - `idle_manager/notification.rs` (267 lines): Notification system and window management
  - **Benefits**:
    - **Clear separation of concerns**: Types, manager, session, and notification logic in separate modules
    - **Easier maintenance**: Each module has single responsibility (~60-270 lines)
    - **Improved readability**: Clear separation between connection management, session handling, and notifications
    - **Better encapsulation**: Internal types not exposed publicly
    - **No functionality changes**: All existing IDLE features work identically
  - **Code Quality**: All changes validated with `cargo fmt`, `cargo check`, and `cargo clippy -- -D warnings`

### Technical Details
- **Code Refactoring Architecture**:
  - **Module Exports**: All handler modules export individual functions, not classes or objects
  - **State Management**: All modules import and use `appState` from `lib/state.svelte` for reactive updates
  - **Wrapper Pattern**: Main `+page.svelte` creates lightweight wrapper functions that call handler modules with correct parameters
  - **Type Safety**: All handler functions are fully typed with TypeScript, parameters explicitly typed
  - **Error Handling**: Each module maintains consistent error handling patterns with appState.error updates
  - **Import Strategy**: Used namespace imports (`import * as EmailOps`) for clarity and organization
  - **No Circular Dependencies**: Clear one-way dependency flow: +page.svelte → handlers → state/utilities
  - **Backward Compatibility**: All function signatures remain the same, existing component props unchanged
  - **Migration Path**: Original file backed up as `+page.svelte.backup` for reference
- **Context Menu Implementation**:
  - **Global State Management** (`src/routes/+page.svelte`):
    - Added state variables: `openContextMenuType: 'folder' | 'email' | null` and `openContextMenuId: string | number | null`
    - Created callback function: `onContextMenuChange(type, id)` passed to all components with context menus
    - Both `AccountFolderSidebar` and `EmailListSidebar` receive these props for coordinated state management
    - State updates are centralized, ensuring only one menu can be open globally
  - **Global Context Menu Disable** (`src/routes/+layout.svelte`):
    - `onMount` lifecycle hook registers global `contextmenu` event listener
    - Event handler calls `preventDefault()` to block browser's default menu
    - Cleanup function removes listener on component unmount
    - Allows shadcn-svelte ContextMenu to function while blocking browser menu
  - **Folder Context Menu** (`src/routes/components/AccountFolderSidebar.svelte`):
    - Receives `openContextMenuType`, `openContextMenuId`, and `onContextMenuChange` props
    - Derived function `isFolderContextMenuOpen(folderName)` checks if specific folder's menu is open
    - ContextMenu.Root `open` prop bound to derived state: `open={isFolderContextMenuOpen(folder.name)}`
    - `onOpenChange` callback updates global state via `onContextMenuChange(isOpen ? 'folder' : null, isOpen ? folder.name : null)`
    - All menu item clicks call `onContextMenuChange(null, null)` to close menu
  - **Email List Context Menu** (`src/routes/components/EmailListSidebar.svelte`):
    - Added imports: `ContextMenu` components, lucide-svelte icons (Mail, Trash2, Star, Eye, EyeOff)
    - New optional callback props: `onMarkAsRead`, `onMarkAsUnread`, `onDeleteEmail`, plus global state props
    - Receives `openContextMenuType`, `openContextMenuId`, and `onContextMenuChange` props for global coordination
    - Derived function `isEmailContextMenuOpen(uid)` checks if specific email's menu is open
    - ContextMenu.Root `open` prop bound to derived state: `open={isEmailContextMenuOpen(email.uid)}`
    - `onOpenChange` callback updates global state via `onContextMenuChange(isOpen ? 'email' : null, isOpen ? email.uid : null)`
    - Each email wrapped in `ContextMenu.Root` with controlled `open` state
    - Conditional menu items based on email state (seen/unseen)
    - Menu closes automatically via `onContextMenuChange(null, null)` after each action
    - Separator elements group related actions visually
    - Destructive styling for Delete action matches design system
  - **Parent Component Handlers** (`src/routes/+page.svelte`):
    - `handleMarkEmailAsRead(uid)`: Marks specific email as read with optimistic UI update
    - `handleMarkEmailAsUnread(uid)`: Marks specific email as unread with optimistic UI update
    - `handleDeleteEmailFromContextMenu(uid)`: Deletes email with confirmation for trash folder
    - All handlers use optimistic updates with rollback on error
    - Handlers passed to EmailListSidebar component props
    - Reuses existing IMAP commands: `mark_email_as_read`, `mark_email_as_unread`, `move_email_to_trash`
- **Folder Management Implementation** (`src-tauri/src/commands/folders.rs`):
  - UTF-7 encoding support for non-ASCII folder names (via `encode_folder_name` helper)
  - Capability detection using IMAP `capabilities()` command
  - IMAP CREATE/DELETE operations wrapped in `tokio::spawn_blocking` for async safety
  - Smart error handling with folder name cloning to avoid borrow checker issues
  - Automatic database migration for `is_local` column on existing installations
- **Folder UI Components** (`src/routes/components/AccountFolderSidebar.svelte`):
  - Dialog component for folder creation with real-time validation
  - AlertDialog component for deletion confirmation with destructive styling
  - ContextMenu component for right-click folder actions (replaces hover delete button)
  - Right-click context menu shows "Delete Folder" with Trash2Icon and destructive styling
  - Controlled ContextMenu state: `openContextMenuFolder` tracks currently open menu by folder name
  - `open` and `onOpenChange` props ensure only one menu is open at a time
  - Smart icon selection: HardDrive icon for local folders, standard icons for remote
  - `canDeleteFolder()` helper prevents deletion of system folders
  - Callbacks: `onFolderCreated` and `onFolderDeleted` for parent state refresh
- **Main Page Integration** (`src/routes/+page.svelte`):
  - `handleFolderCreated()`: Reloads folder list after creation
  - `handleFolderDeleted()`: Reloads folders and switches to Inbox if needed
  - Seamless integration with existing account/folder switching logic
- **Code Quality**:
  - All Rust code formatted with `cargo fmt`
  - Compiled without errors via `cargo check`
  - Zero Clippy warnings with `cargo clippy -- -D warnings`
  - TypeScript/Svelte code validated with `svelte-check` (0 errors, 0 warnings)

## [0.4.2] - 2025-10-29

### Added
- **Local Draft Storage**: Implemented SQLite-based local draft storage system
  - **Problem Solved**: Gmail IMAP APPEND creates drafts invisible in web interface/other clients
  - **Root Cause**: Gmail API uses draft containers separate from IMAP message system
  - **Solution**: Store drafts locally in SQLite database instead of syncing to IMAP server
  - **Database Schema**: Added `drafts` table with columns: `id`, `account_id`, `to_addr`, `cc_addr`, `subject`, `body`, `attachments`, `draft_type`, `original_email_id`, `created_at`, `updated_at`
  - **Draft Types**: Supports three types - "compose" (new email), "reply" (reply to email), "forward" (forward email)
  - **Auto-save**: Drafts automatically saved every 3 seconds while composing
  - **Attachment Support**: Full attachment handling with file metadata preservation
  - **Commands**:
    - `save_draft`: Create or update draft in local database
    - `load_draft`: Retrieve draft with all metadata and attachments
    - `list_drafts`: List all drafts for an account
    - `delete_draft`: Remove draft from database
  - **Deletion on Send**: Drafts automatically deleted after successful email send

### Improved
- **Drafts UI Consistency**: Redesigned DraftsList component to match EmailListSidebar style
  - **Unified Layout**: Uses same Sidebar component structure as email folders
  - **Consistent Header**: Title + search box + pagination (identical to email list)
  - **Matching List Items**: Same two-row layout, hover effects, and selection highlighting
  - **Search Functionality**: Real-time search across subject, recipient, and CC fields
  - **Pagination Support**: 50 drafts per page with same pagination component
  - **Loading States**: Skeleton placeholders matching email list loading UI
  - **Draft Type Badges**: Color-coded badges - Reply (secondary), Forward (outline), Draft (default)
  - **Delete Button**: Trash icon appearing on hover, consistent with star icon pattern
  - **Time Formatting**: Uses same `formatLocalDateTime` utility as email list
  - **Visual Consistency**: Identical spacing, fonts, colors, and animations

- **Dialog UI Standardization**: Replaced system dialogs with shadcn-svelte components
  - **Confirm Delete Draft**: Custom `ConfirmDialog` component using AlertDialog
    - Professional modal with overlay and proper styling
    - Destructive variant with red "Delete" button for clarity
    - Clear warning message: "This action cannot be undone"
    - Replaces native `ask()` dialog for better UX
  - **Email Sent Success**: Changed from blocking `message()` dialog to toast notification
    - Non-intrusive success message using `toast.success()`
    - Doesn't interrupt user workflow
    - Consistent with other app notifications
  - **ConfirmDialog Component**: Reusable confirmation dialog
    - Props: `title`, `description`, `confirmText`, `cancelText`, `variant`
    - Supports `destructive` and `default` button styles
    - Two-way binding with `$bindable()` for `open` state
    - Clean API without `asChild` or `builders` complexity

### Fixed
- **IDLE Connection Limit**: Fixed "Too many simultaneous connections" error with Gmail
  - **Problem**: IDLE manager created connections for every folder (INBOX, Sent, Drafts, Important, Spam, Trash, etc.)
  - **Root Cause**: Each account with 9 folders created 9 IDLE connections, exceeding Gmail's 15 connection limit
  - **Solution**: Limited IDLE monitoring to INBOX folder only
  - **Implementation**: Modified `StartAllForAccount` command to filter for INBOX/收件箱 folders
  - **Impact**: Reduced connection count from 9+ per account to 1 per account
  - **Code Change**: Changed from `tasks.insert(key, task)` for all folders to conditional insertion only for INBOX
  - **Clippy Fix**: Changed `vec!["INBOX", "收件箱"]` to array `["INBOX", "收件箱"]` to avoid useless Vec allocation

### Technical Details
- **Draft Manager** (`src/routes/lib/draft-manager.ts`):
  - `saveDraft()`: Saves draft with account ID, recipients, subject, body, attachments, draft type
  - `loadDraft()`: Returns draft data with parsed attachments JSON
  - `listDrafts()`: Returns array of draft metadata for account
  - `deleteDraft()`: Removes draft by ID
  - `filesToDraftAttachments()`: Converts File objects to database-compatible format
  - `draftAttachmentsToFiles()`: Converts database attachments back to File objects
  - Auto-save debouncing with 3-second delay

- **Backend Commands** (`src-tauri/src/commands/drafts.rs`):
  - SQLite-based implementation using `sqlx` async database driver
  - Manual row mapping for `list_drafts` to handle `DraftType` enum deserialization
  - `DraftType` serialization using serde JSON with lowercase format
  - Foreign key constraint on `account_id` with CASCADE delete
  - Timestamps stored as Unix epoch integers

- **UI Components**:
  - `DraftsList.svelte`: Complete redesign matching EmailListSidebar
  - `ConfirmDialog.svelte`: Reusable confirmation dialog component
  - `AccountFolderSidebar.svelte`: Added Drafts button with FilePenIcon
  - Drafts folder toggle controlled by `appState.showDraftsFolder` state

- **Database Migration**:
  - Table creation in `db::init()` with IF NOT EXISTS guard
  - Supports both new installations and existing databases
  - No data loss for users upgrading from IMAP draft version

## [0.4.1] - 2025-10-29

### Added
- **Email Star/Flag Feature**: Implemented full email starring/flagging functionality with bidirectional sync
  - **UI Components**: Added star icon (⭐/☆) to email list for each email
  - **Click to Toggle**: Users can click star icon to mark/unmark emails as starred
  - **Bidirectional Sync**: Star status syncs between client and email server
    - Starring email in client updates server via IMAP `\Flagged` flag
    - Starring email in webmail (Gmail, etc.) syncs to client automatically
  - **IDLE Real-time Updates**: Uses IMAP IDLE `FlagsChanged` events for instant sync
  - **Performance Optimized**: Only syncs specific UID when flag changes detected (35x faster than full sync)
  - **Database Support**: Added `flagged` column to emails table for local caching
  - **Backend Commands**:
    - `mark_email_as_flagged`: Sets `\Flagged` flag on IMAP server and updates cache
    - `mark_email_as_unflagged`: Removes `\Flagged` flag on IMAP server and updates cache
    - `sync_email_flags`: Syncs all email flags in folder (used during incremental sync)
    - `sync_specific_email_flags`: Syncs single email flag efficiently (used for IDLE events)
  - **Accessibility**: Star button supports keyboard navigation (Enter/Space) and screen readers

### Improved
- **Optimistic UI Updates**: Enhanced user experience with instant visual feedback for flag operations
  - **Zero Latency Response**: Star and read/unread status update immediately on click (0ms perceived delay)
  - **Smart Rollback**: Automatically reverts UI changes if server request fails
  - **Error Handling**: Shows clear error message and restores previous state on failure
  - **Network Independent**: UI updates before server responds, making app feel extremely responsive
  - **Consistent UX**: Applied optimistic update pattern to both star toggle and read/unread toggle
  - **Performance Impact**: Eliminates 500-1000ms perceived latency from user interactions
- **Flag Sync Performance**: Optimized flag synchronization strategy for large mailboxes
  - **Incremental Sync**: Background flag sync runs asynchronously during normal email sync
  - **Performance Metrics**: 217 emails synced in 1.7 seconds (1.65s IMAP + 0.02s database)
  - **Batched Fetching**: Processes flags in batches of 100 emails to avoid server overload
  - **Smart Detection**: Only updates database rows where flags actually changed
  - **IDLE Optimization**: FlagsChanged events trigger single-UID sync instead of full mailbox scan
  - **35x Speedup**: Single email flag sync takes ~0.05s vs 1.7s for full mailbox (tested on 217 emails)
  - **Detailed Logging**: Performance metrics logged for monitoring (IMAP time, DB time, changed count)

### Added
- **IMAP ID Command Support**: Implemented IMAP client identification for Chinese email providers
  - **Supported Providers**: 163.com (NetEase), 126.com, qq.com, sina.com, yeah.net, sohu.com
  - **Problem Solved**: "Unsafe Login. Please contact kefu@188.com" error when accessing 163.com folders
  - **RFC 2971 Compliance**: Sends IMAP ID command to identify client to mail server
  - **Client Information**: Identifies as "Colimail" with version number and vendor details
  - **Implementation**: Automatically sends ID command after authentication for Chinese mail providers
  - **Fallback Handling**: Continues authentication even if ID command fails (some servers don't support it)
  - **Technical Details**: Uses `Session::run_command_and_read_response()` public API method
  - **Provider Detection**: Smart domain matching to detect Chinese email providers automatically
  - **Security**: No sensitive information sent in ID command (only client name/version)
  - **Impact**: 163.com, 126.com, and other NetEase mailboxes now fully functional
- **IDLE Capability Detection**: Added automatic detection of IDLE support before attempting real-time monitoring
  - **Problem Solved**: "Bad Response: command not support" errors on servers without IDLE (e.g., 163.com)
  - **RFC 2177 Compliance**: Checks server CAPABILITIES before using IDLE extension
  - **Smart Detection**: Uses `capabilities.has_str("IDLE")` to verify server support
  - **Graceful Degradation**: Stops IDLE monitoring when server doesn't support it
  - **User Guidance**: Displays helpful message to use manual sync (Sync Mail button) instead
  - **No Infinite Loops**: Prevents endless reconnection attempts when IDLE is unsupported
  - **Impact**: 163.com and other servers without IDLE no longer spam error logs

### Fixed
- **SMTP Port 465 SSL/TLS Support**: Fixed SMTP connection failures for Chinese email providers using port 465
  - **Problem**: 163.com (NetEase) and other Chinese providers failed with "response error: incomplete response"
  - **Root Cause**: Code used STARTTLS for all SMTP ports, but port 465 requires direct SSL/TLS connection
  - **Port 465**: Direct SSL/TLS connection (implicit TLS) - used by 163.com, 126.com, QQ Mail, etc.
  - **Port 587**: STARTTLS connection (explicit TLS) - used by Gmail, Outlook, etc.
  - **Solution**: Auto-detect port and use appropriate connection method
    - Port 465: `AsyncSmtpTransport::relay()` with direct SSL/TLS
    - Port 587: `AsyncSmtpTransport::starttls_relay()` with STARTTLS upgrade
  - **Files Modified**: `test_connection.rs` and `send.rs` (all 3 functions: `send_email`, `reply_email`, `forward_email`)
  - **Impact**: Both OAuth2 and basic authentication now work correctly with port 465
  - **Testing**: Successfully tested with 163.com account (SMTP connection and email sending)
- **UTF-7 Folder Name Encoding**: Fixed issue where folders with non-ASCII names (German, French, Russian, Chinese, etc.) could not be accessed
  - **Problem**: Folders with special characters like German umlauts (ä, ö, ü) were being skipped with "unsupported folder name" error
  - **Root Cause**: IMAP uses Modified UTF-7 encoding for folder names (e.g., `Entw&APw-rfe` for "Entwürfe"), but code was using decoded UTF-8 names for SELECT operations
  - **Example**: GMX German folders `Entwürfe` (Drafts) and `Gelöscht` (Deleted) were inaccessible
  - **Solution**: Store raw UTF-7 encoded names in database `name` field for IMAP operations, while `display_name` contains decoded UTF-8 for UI display
  - **Impact**: Now supports folders in any language with special characters (German ä/ö/ü, French é/è/à, Spanish ñ, Russian Cyrillic, Chinese/Japanese characters)
  - **Technical**: Changed `folder.name` to store `raw_name` (UTF-7 encoded) instead of `decoded_name`, ensuring IMAP SELECT operations use correct encoding
- **Background BODYSTRUCTURE Fetch Retry Logic**: Fixed issue where failed emails were not retried during background attachment detection
  - **Problem**: When batch BODYSTRUCTURE fetch failed (e.g., connection error), the task would reconnect but skip failed emails entirely, leaving them with NULL `has_attachments` status
  - **Root Cause**: Error handling would `continue` to next batch without tracking failed UIDs, causing infinite "X emails pending" notifications on every folder click
  - **Solution**: Implemented two-tier retry mechanism
    - Track all failed UIDs during batch processing in `failed_uids` list
    - After batch loop completes, retry failed emails individually (one by one)
    - Connection errors during individual retry: reconnect and continue
    - Non-connection errors (e.g., malformed email structure): mark as "no attachments" to prevent infinite retry loops
  - **Impact**: Emails with problematic BODYSTRUCTURE (rare edge cases) no longer get stuck in pending state
  - **User Experience**: No more persistent "5 emails pending" messages that never resolve
  - **Reliability**: Background task now processes 100% of emails, even when encountering server disconnections or malformed messages

### Improved
- **Long-Running Connection Management**: Enhanced connection stability during bulk BODYSTRUCTURE processing
  - Preventive reconnection every 100 batches to avoid server-side timeout disconnections
  - Reactive reconnection on Bye/TagMismatch/Connection errors with 2-second retry delay
  - Graceful error recovery: break loop only if reconnection fails to prevent cascading errors
  - Successfully tested with 5000+ email processing without connection failures

## [0.4.0] - 2025-10-27

### Added
- **Email Provider Presets**: Added 13 pre-configured email provider presets for manual account configuration
  - Includes popular providers: Yahoo Mail, iCloud Mail, Zoho Mail, ProtonMail, Fastmail, GMX, AOL, QQ Mail, 163 Mail, 126 Mail, Sina Mail
  - Custom option for advanced users to configure any email provider
  - Combobox component with search functionality for easy provider selection
  - Auto-fills IMAP/SMTP server addresses and ports when provider is selected
  - Reduces configuration errors and improves user experience
  - Each preset includes descriptive text (e.g., "Apple iCloud Mail (requires app-specific password)")
- **Connection Testing**: Implemented pre-validation connection test before account creation
  - Added "Test Connection" button to verify IMAP and SMTP settings
  - Tests both IMAP and SMTP connections independently
  - Visual feedback with green checkmarks (✓) for successful connections and red crosses (✗) for failures
  - Displays specific error messages if connection fails (e.g., authentication failure, server unreachable)
  - "Create Account" button only enables after successful connection test
  - Prevents users from adding non-functional accounts to the system
  - Async testing with loading state: "Testing Connection..." indicator
- **OAuth2 Email Input Validation**: Enhanced OAuth2 email input to prevent common mistakes
  - Automatically validates email format as user types
  - Prevents users from entering full email addresses like "username@gmail.com"
  - Users only need to enter username portion (e.g., "username")
  - System automatically appends provider domain (@gmail.com or @outlook.com)
  - Displays warning message when "@" symbol is detected in input
  - Reduces authentication errors caused by incorrect email format

### Improved
- **Add Account Dialog Layout**: Optimized manual configuration form for better visual hierarchy
  - Two-column layout: Email/Password fields on left, IMAP/SMTP settings on right
  - IMAP and SMTP server/port inputs displayed horizontally (server and port on same line)
  - Server address inputs use `flex-1` to automatically fill available space
  - Port inputs maintain fixed width (`w-20`) for consistent sizing
  - Reduced visual clutter and improved form compactness
  - Better utilization of dialog width for more efficient space usage

### Fixed
- **Email Date Parsing**: Resolved incorrect date display issue for emails with malformed or missing Date headers
  - **Root Cause**: Some emails have empty or invalid Date headers (e.g., future dates, missing Date field)
  - When Date header parsing failed, system used current time (`Utc::now()`) as fallback, showing email fetch time instead of actual email time
  - **Solution**: Implemented three-tier date parsing fallback mechanism
    1. First attempt: Parse standard Date header (RFC 2822/3339 formats)
    2. Second attempt: Use IMAP INTERNALDATE (server's received time) when Date header fails
    3. Last resort: Use current time only when both previous methods fail
  - Added INTERNALDATE to all FETCH commands: `"(UID ENVELOPE BODYSTRUCTURE FLAGS INTERNALDATE)"`
  - INTERNALDATE provides reliable server-side timestamp even when email's Date header is corrupted
  - Particularly fixes GMX inbox emails and other providers with unreliable Date headers
  - Example: Email with Date="Thu, 26 Jun 2025 09:34:37 +0200" (future date) now correctly shows INTERNALDATE="Sun, 31 Mar 2024 10:51:26 +0000" (actual receive time)
- **Attachment Detection in Email List**: Fixed attachment indicators not showing immediately when emails are fetched
  - **Root Cause**: Email sync operations skipped fetching BODYSTRUCTURE to avoid parsing issues with non-ASCII attachment filenames
  - Attachment detection only occurred when users clicked to open an email, causing paperclip icons (📎) to appear belatedly
  - **Solution**: Re-enabled BODYSTRUCTURE fetching during all sync operations (full sync, incremental sync, and initial sync)
  - Updated `parse_email_headers()` in `sync.rs` to use `check_for_attachments()` function on BODYSTRUCTURE data
  - Modified `save_emails_to_cache()` in `cache.rs` to properly persist `has_attachments` flag to database
  - IMAP FETCH command updated from `"(UID ENVELOPE FLAGS INTERNALDATE)"` to `"(UID ENVELOPE BODYSTRUCTURE FLAGS INTERNALDATE)"`
  - Attachment indicators now display immediately in email list upon sync, matching behavior of direct fetch operations
  - Users can see which emails have attachments without needing to open them first
  - `check_for_attachments()` uses debug format string matching which is immune to non-ASCII filename encoding issues

### Improved
- **Console Logging Optimization**: Reduced excessive debug output during email synchronization
  - **Email Cache Logging**: Changed `save_emails_to_cache()` to output single success message instead of logging each email individually
    - Before: `💾 Saving email UID XXX to cache` × N times (where N = number of emails)
    - After: `✅ Saved N emails to cache for folder XXX` (single message)
    - Error cases still output individual UID and error details for debugging
  - **Date Parsing Logging**: Silenced verbose date parsing messages for common scenarios
    - Emails with empty Date headers (`(No Date)`) no longer generate console output when INTERNALDATE succeeds
    - Only logs warnings when Date header has invalid content (not just empty)
    - Maintains detailed error logging when both Date and INTERNALDATE parsing fail
  - Result: Syncing 5000+ emails now produces clean, readable logs instead of thousands of repetitive messages

### Changed
- **CRITICAL: imap Crate Migration**: Upgraded from `imap 2.4.1` to `imap 3.0.0-alpha.15` to resolve critical limitations
  - **Motivation**: Version 2.4.1 had severe limitations with GMX email provider (500-email fetch limit) and crashes on complex BODYSTRUCTURE parsing
  - **Breaking Changes**: Complete API redesign requiring systematic migration of all IMAP operations
  - **Migration Scope**: Updated 12 connection points across 5 modules: `idle_manager.rs`, `fetch.rs`, `sync.rs`, `delete.rs`, `send.rs`
  - **Connection API**: 
    - Old: `Client::connect()` → `Client::secure_connect()`
    - New: `ClientBuilder::new(domain).native_tls().connect()` with builder pattern
    - Replaced tuple-based connection with explicit hostname/port parameters
    - Enhanced TLS configuration with native TLS connector support
  - **Session API**:
    - Old: `client.login(user, pass).ok()` returns `Option<Session>`
    - New: `client.login(user, pass)` returns `Result<Session, Error>`
    - Explicit error handling required for all login operations
  - **Fetch API**:
    - Old: `session.fetch()` with `mailparse` integration
    - New: Requires explicit fetching of `BODY[]` or `BODY.PEEK[]`
    - Message body no longer included by default in fetch operations
    - Enhanced lazy fetching for better performance on large mailboxes
  - **IDLE API**: Complete redesign following RFC 2177 specification
    - Old: `session.idle().wait_keepalive()` with manual loop
    - New: `session.idle()` returns `Handle` directly (no Result wrapper)
    - Uses `Handle::wait_while(callback)` with `FnMut(UnsolicitedResponse) -> bool` signature
    - Callback returns `true` to continue, `false` to stop monitoring
    - UnsolicitedResponse variants: `Exists`, `Recent`, `Expunge`, `Fetch`
    - Implemented smart detection: exits IDLE on `Exists` when new message count increases
    - Automatic 29-minute keepalive per RFC 2177 (built-in, no manual timer needed)
    - Removed legacy `detect_mailbox_changes()` helper function (logic now in callback)
  - **Folder Attributes**:
    - Old: Limited attribute access via `imap::types`
    - New: Full `NameAttribute` enum support via `imap-proto 0.16.4` dependency
    - Added `NoSelect` attribute checking for proper folder selectability validation
    - Fixed trash folder detection in `delete.rs` (replaced TODO workaround)
  - **Performance Improvements**:
    - Lazy body fetching reduces initial sync time by ~40%
    - GMX provider now supports fetching all emails (no 500-email limit)
    - No more BODYSTRUCTURE parsing crashes on complex multipart messages
    - IDLE notifications now more responsive with fine-grained UnsolicitedResponse handling

### Added
- **New Dependency**: `imap-proto = "0.16.4"`
  - Required for accessing `NameAttribute` enum not directly exported by imap crate
  - Provides low-level IMAP protocol types for advanced folder attribute checking
  - Used in `delete.rs` for `NoSelect` attribute detection

### Fixed
- **GMX Provider Compatibility**: Eliminated 500-email fetch limit imposed by imap 2.4.1
  - Users can now fetch complete email history from GMX accounts
  - No more artificial truncation of email list
- **BODYSTRUCTURE Crashes**: Resolved parsing failures on complex multipart messages
  - Application no longer crashes on emails with nested attachments or unusual MIME structures
  - Improved reliability when syncing mailboxes with diverse message formats
- **IDLE Reliability**: Fixed real-time notification system with proper RFC 2177 implementation
  - More accurate detection of new messages via `UnsolicitedResponse::Exists`
  - Proper handling of expunge events and flag changes
  - Reduced false positives in mailbox change detection
- **Folder Selection**: Fixed incorrect folder selectability checks in trash operations
  - Properly detects `\NoSelect` attribute to avoid selecting non-selectable folders
  - Prevents errors when attempting to move emails to parent/container folders

### Technical Details
- **Migration Pattern**: All IMAP connections now follow consistent builder pattern:
  ```rust
  let domain = format!("{}:{}", imap_host, imap_port);
  let tls = native_tls::TlsConnector::builder().build()
      .map_err(|e| format!("TLS error: {}", e))?;
  let client = ClientBuilder::new(&domain).native_tls(tls)
      .connect().map_err(|e| format!("Connection failed: {}", e))?;
  let session = client.login(username, password)
      .map_err(|e| format!("Login failed: {:?}", e))?;
  ```
- **IDLE Implementation**: Real-time notifications use callback-based pattern:
  ```rust
  let mut idle_handle = imap_session.idle();
  idle_handle.wait_while(|response| {
      match response {
          UnsolicitedResponse::Exists(count) => {
              if count > prev_exists {
                  // Emit notification, return false to stop
                  return false;
              }
          }
          _ => { /* Continue monitoring */ }
      }
      true
  })?;
  ```
- **Code Quality**: All changes validated with `cargo fmt`, `cargo check`, and `cargo clippy -- -D warnings`
- **Documentation Reference**: Migration based on official imap 3.0.0-alpha.15 documentation from docs.rs

## [0.3.0] - 2025-10-26

### Security
- **Credential Encryption**: Implemented OS-native keyring storage for all sensitive credentials
  - **Replaced plaintext SQLite storage** with encrypted OS keychain (Windows Credential Manager, macOS Keychain, Linux Secret Service)
  - All passwords and OAuth2 tokens now encrypted at rest by the operating system
  - Removed sensitive columns (`password`, `access_token`, `refresh_token`) from SQLite database
  - Database now stores only non-sensitive metadata (email addresses, server configurations, sync state)
  - Added `keyring` crate v3.6.3 with `windows-native` feature for proper Windows Credential Manager integration
  - Credentials stored separately by field type (password, access_token, refresh_token, expiration) to avoid platform size limits
  - Implemented hash-based short keys (8-character identifiers) to comply with Windows naming constraints
  - **Automatic chunking** for long values (OAuth2 tokens): Values exceeding 1200 characters split into multiple keyring entries
  - Windows UTF-16 encoding overhead handled correctly (2 bytes per character)
  - Each credential chunk limited to 1200 UTF-8 characters (~2400 UTF-16 bytes) to stay under Windows 2560-byte limit
  - Seamless retrieval of chunked credentials reassembled from multiple keyring entries
  - **Backward compatibility**: Existing accounts automatically migrated to secure storage on next login
  - Enhanced security for both Basic Authentication and OAuth2 flows (Gmail, Outlook, custom IMAP)
  
### Added
- **Security Module** (`src-tauri/src/security.rs`): Complete credential management system
  - `store_credentials()`: Securely stores account credentials in OS keyring
  - `get_credentials()`: Retrieves encrypted credentials from OS keyring
  - `delete_credentials()`: Removes credentials from OS keyring on account deletion
  - `update_credentials()`: Updates specific credential fields (e.g., refreshed OAuth2 tokens)
  - Supports partial updates without affecting other fields
  - Smart chunking for values exceeding platform limits
  - Email-to-hash mapping for efficient credential lookup

### Technical Details
- Added dependency: `keyring = { version = "3.6", features = ["windows-native"] }`
- Service name for keyring entries: `com.colimail.app`
- Maximum credential length: 1200 UTF-8 characters (accounts for UTF-16 encoding overhead)
- Credential key format: `{8-char-hash}:{field-type}` (e.g., `6bafc6a9:at` for access token)
- Field types: `pwd` (password), `at` (access_token), `rt` (refresh_token), `exp` (expiration), `email` (email mapping)
- Chunked credential format: `{key}:count` stores chunk count, `{key}:chunk{N}` stores each chunk
- Database schema updated: Removed `password` and OAuth2 token columns from `accounts` table
- Modified commands: `save_account_config`, `load_account_configs`, `delete_account_config`, `complete_oauth2_flow`, `ensure_valid_token`
- UTF-8 character boundary awareness: Chunking algorithm preserves valid UTF-8 sequences
- Error handling: Detailed error messages for debugging keyring operations
- Cross-platform support: Windows Credential Manager, macOS Keychain, Linux Secret Service (libsecret)

## [0.2.5] - 2025-10-26

### Improved
- **Release Builds**: Enhanced GitHub Actions workflow for better cross-platform support
  - Added Linux support with Ubuntu 20.04 base for maximum compatibility
  - Generates both `.deb` and `.AppImage` packages for Linux
  - Optimized macOS DMG packaging configuration with proper window layout
  - Added macOS minimum system version requirement (10.13)
  - Improved build reliability with explicit dependency installation
  - All three platforms (Windows, macOS, Linux) build independently without affecting each other

### Technical Details
- Updated `.github/workflows/release.yml`:
  - Added `ubuntu-20.04` platform to matrix builds
  - Installed Linux dependencies: `libgtk-3-dev`, `libwebkit2gtk-4.0-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf`, `libssl-dev`
  - Added Linux artifact upload step for `.deb` and `.AppImage` files
  - Enhanced macOS dependency installation step
  - Added `tauriScript` configuration for consistent build commands
  - Added optional Tauri signing environment variables support
- Updated `src-tauri/tauri.conf.json`:
  - Added macOS-specific bundle configuration with DMG window layout
  - Set minimum macOS version to 10.13 for broader compatibility
  - Configured app and Applications folder positions in DMG

## [0.2.4] - 2025-10-26

### Added
- **Email Pagination**: Implemented pagination for email list to improve performance with large mailboxes
  - Default 50 emails per page for optimal loading speed
  - Compact pagination controls between search box and email list
  - Page navigation with previous/next arrow buttons
  - Direct page jump by entering page number (press Enter to confirm)
  - Email range display showing current range (e.g., "1-50/2300")
  - Total email count indicator for folder visibility
  - Automatic page reset when switching folders or applying filters
  - Smart page adjustment when filtering reduces results beyond current page
- **Full Email Sync**: Removed development limitations to enable complete mailbox synchronization
  - Changed from fetching recent 20-100 emails to fetching all emails in mailbox
  - `fetch_emails` command now retrieves complete email list from IMAP server
  - `sync_emails` command performs full synchronization on first sync or UIDVALIDITY change
  - Incremental sync continues to work efficiently for new messages
  - Enables proper email management for users with large email archives
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
- **Pagination UI**: Optimized layout for compact and clean appearance
  - Reduced element spacing from `gap-3` to `gap-1.5` for tighter layout
  - Minimized button size from `h-7 w-7` to `h-6 w-6`
  - Reduced icon size from `h-4 w-4` to `h-3.5 w-3.5`
  - Compacted input box from `h-7 w-12` to `h-6 w-10`
  - Reduced internal spacing with `gap-1` in page number section
  - Added `whitespace-nowrap` and `shrink-0` to prevent layout breaking
  - All pagination elements fit comfortably in one line
  - Sidebar header spacing optimized from `gap-3.5` to `gap-1` with `p-1` padding
  - Removed redundant border-bottom from pagination component (email list already has borders)
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
- Created `src/routes/components/Pagination.svelte`:
  - Compact pagination control with previous/next buttons and page input
  - Props: `currentPage`, `totalPages`, `pageSize`, `totalItems`, `onPageChange`
  - Displays email range: `{startItem}-{endItem}/{totalItems}`
  - Input validation for page numbers with Enter key submission
  - Automatic reset to valid page on invalid input
  - Uses Lucide icons: `ChevronLeft`, `ChevronRight`
  - Responsive sizing with minimal footprint (`text-xs`, compact buttons)
- Modified `src/routes/lib/state.svelte.ts`:
  - Added `currentPage` state (default: 1)
  - Added `pageSize` state (default: 50)
  - Updated `resetFolderState()` to reset `currentPage` to 1
- Updated `src/routes/components/EmailListSidebar.svelte`:
  - Added pagination props: `currentPage`, `pageSize`, `onPageChange`
  - Implemented `paginatedEmails` derived function to slice email array
  - Calculated `totalPages` based on filtered email count
  - Added safety check to reset page when exceeds total pages (e.g., after filtering)
  - Integrated `Pagination` component between search box and email list
  - Optimized header spacing with `gap-1` and `p-1` for compact layout
- Updated `src/routes/+page.svelte`:
  - Added `handlePageChange()` function to update `currentPage` state
  - Clears selected email when changing pages for cleaner navigation
  - Passes pagination props to `EmailListSidebar` component
- Modified `src-tauri/src/commands/emails/fetch.rs`:
  - Changed `fetch_emails` sequence range from `{start}:{total}` (last 20) to `1:{total}` (all emails)
  - Updated log messages to indicate fetching all messages
- Modified `src-tauri/src/commands/emails/sync.rs`:
  - Changed full sync from fetching last 100 to fetching all emails (`1:{total}`)
  - Applied to both UIDVALIDITY change scenario and first sync scenario
  - Incremental sync logic unchanged (still fetches only new messages)
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
