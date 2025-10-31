# Logging System Documentation

## Overview

Colimail now features a comprehensive production-grade logging system based on `tracing` and `tracing-subscriber`. This system records critical information in production builds, helping users trace the root cause of issues when problems occur.

## Logging Features

### Automatic Log Management
- **Daily Rotation**: Creates a new log file each day
- **Automatic Cleanup**: Keeps only the last 7 days of logs, automatically deleting older files
- **JSON Format**: Logs stored in JSON format for easy parsing and analysis
- **Structured Logging**: Includes metadata such as timestamps, log levels, modules, line numbers, and thread IDs

### Log Levels
- **DEBUG**: Debugging information (development mode only)
- **INFO**: General information (application startup, account operations, IDLE monitoring, etc.)
- **WARN**: Warning messages (non-critical errors, degraded operations)
- **ERROR**: Error messages (connection failures, authentication errors, etc.)

### Log Storage Location

Log files are stored in platform-specific user data directories:

- **Windows**: `C:\Users\<Username>\AppData\Roaming\Colimail\logs\`
- **macOS**: `~/Library/Application Support/Colimail/logs/`
- **Linux**: `~/.local/share/Colimail/logs/`

File naming format: `colimail.YYYY-MM-DD.log`

Example: `colimail.2025-10-31.log`

## Backend Logging API

### Get Log Directory
```rust
use tauri::command;

#[command]
pub fn get_log_directory() -> Result<String, String>
```

### Get Current Log File Path
```rust
#[command]
pub fn get_current_log_file() -> Result<String, String>
```

### Read Recent Log Entries
```rust
#[command]
pub fn read_recent_logs(lines: Option<usize>) -> Result<String, String>
```
Default reads the last 100 lines.

### List All Log Files
```rust
#[command]
pub fn list_log_files() -> Result<Vec<String>, String>
```
Returns a list of log files sorted by date (newest first).

### Read Specific Log File
```rust
#[command]
pub fn read_log_file(filename: String) -> Result<String, String>
```

## Frontend Usage Examples

```typescript
import { invoke } from '@tauri-apps/api/core';

// Get log directory
const logDir = await invoke<string>('get_log_directory');
console.log('Log directory:', logDir);

// Get current log file
const currentLogFile = await invoke<string>('get_current_log_file');
console.log('Current log file:', currentLogFile);

// Read last 50 lines of logs
const recentLogs = await invoke<string>('read_recent_logs', { lines: 50 });
console.log('Recent logs:', recentLogs);

// List all log files
const logFiles = await invoke<string[]>('list_log_files');
console.log('Available log files:', logFiles);

// Read a specific log file
const logContent = await invoke<string>('read_log_file', {
  filename: 'colimail.2025-10-31.log'
});
console.log('Log content:', logContent);
```

## Adding Logs to Code

### Basic Usage

```rust
use tracing;

// Info log
tracing::info!("Application started");

// Debug log
tracing::debug!("Processing email batch");

// Warning log
tracing::warn!("Failed to connect, retrying...");

// Error log
tracing::error!("Authentication failed");
```

### Structured Logging (Recommended)

```rust
// Logs with fields
tracing::info!(
    account_id = account.id,
    email = %account.email,
    "Account loaded successfully"
);

// Include error information
tracing::error!(
    error = %e,
    account_id = id,
    "Failed to sync emails"
);

// Multiple fields
tracing::info!(
    folder = %folder_name,
    count = new_emails.len(),
    account_id = config.id,
    "Received new emails"
);
```

### Field Format Reference

- `field = value`: Direct value (numbers, booleans, etc.)
- `field = %value`: Format using `Display` trait
- `field = ?value`: Format using `Debug` trait

## Sample Log Output

```json
{
  "timestamp": "2025-10-31T10:30:45.123456Z",
  "level": "INFO",
  "fields": {
    "message": "Starting IDLE monitoring for all accounts",
    "account_count": 3
  },
  "target": "colimail::main",
  "filename": "src/main.rs",
  "line_number": 277,
  "thread_id": "ThreadId(2)"
}
```

## Troubleshooting Workflow

When users report issues:

1. **Get Log Directory**: Use `get_log_directory()` command
2. **View Recent Logs**: Use `read_recent_logs(100)` to quickly review recent operations
3. **Search for Errors**: Search for `"level":"ERROR"` or `"level":"WARN"` in logs
4. **Locate Issues**: Find problem context using timestamps, account info, and module names
5. **Export Logs**: Send relevant log files to developers for analysis

## Advanced Configuration

### Adjusting Log Level

Users can adjust log levels by setting the `RUST_LOG` environment variable:

```bash
# Windows (PowerShell)
$env:RUST_LOG="debug"
.\Colimail.exe

# Windows (CMD)
set RUST_LOG=debug
Colimail.exe

# Linux/macOS
RUST_LOG=debug ./colimail
```

Available levels:
- `RUST_LOG=error`: Log errors only
- `RUST_LOG=warn`: Log warnings and errors
- `RUST_LOG=info`: Log info, warnings, and errors (default for release builds)
- `RUST_LOG=debug`: Log everything (default for development builds)
- `RUST_LOG=trace`: Log most detailed trace information

### Module-Specific Log Levels

```bash
# Only log trace level for colimail module, warn for everything else
RUST_LOG=warn,colimail=trace ./colimail

# Only log debug level for email sync module
RUST_LOG=info,colimail::commands::emails::sync=debug ./colimail
```

## Performance Impact

- **File Size**: Daily log files typically range from 1-10 MB
- **Performance Overhead**: Asynchronous log writing with minimal impact on app performance (< 1%)
- **Storage Usage**: Maximum 7 days of logs, typically < 70 MB total

## Privacy and Security

- **No Password Logging**: The logging system never records user passwords or sensitive credentials
- **Email Content**: Does not log email body content, only metadata (subject, sender, UID, etc.)
- **Local Storage**: All logs are stored locally only, never uploaded to any server
- **Automatic Cleanup**: Logs are automatically deleted after 7 days to avoid long-term privacy data retention

## Future Plans

- [ ] Add log viewer UI interface
- [ ] Support log export functionality (ZIP packaging)
- [ ] Add log search and filtering features
- [ ] Integrate crash reporting system
- [ ] Support optional anonymous error reporting
