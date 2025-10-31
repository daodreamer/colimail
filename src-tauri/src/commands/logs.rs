use crate::logger;
use tauri::command;

/// Get the directory where log files are stored
#[command]
pub fn get_log_directory() -> Result<String, String> {
    let log_dir = logger::get_log_dir()?;
    Ok(log_dir.to_string_lossy().to_string())
}

/// Get the path to the current log file
#[command]
pub fn get_current_log_file() -> Result<String, String> {
    let log_file = logger::get_current_log_file()?;
    Ok(log_file.to_string_lossy().to_string())
}

/// Read the most recent log entries
///
/// # Arguments
/// * `lines` - Number of recent lines to read (default: 100)
#[command]
pub fn read_recent_logs(lines: Option<usize>) -> Result<String, String> {
    let lines = lines.unwrap_or(100);
    logger::read_recent_logs(lines)
}

/// List all available log files
#[command]
pub fn list_log_files() -> Result<Vec<String>, String> {
    let log_dir = logger::get_log_dir()?;

    let mut log_files = Vec::new();

    match std::fs::read_dir(&log_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("log") {
                    if let Some(filename) = path.file_name() {
                        log_files.push(filename.to_string_lossy().to_string());
                    }
                }
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "Failed to read log directory");
            return Err(format!("Failed to read log directory: {}", e));
        }
    }

    // Sort by filename (which includes date, so it sorts chronologically)
    log_files.sort();
    log_files.reverse(); // Most recent first

    Ok(log_files)
}

/// Read a specific log file by filename
#[command]
pub fn read_log_file(filename: String) -> Result<String, String> {
    let log_dir = logger::get_log_dir()?;
    let log_file = log_dir.join(&filename);

    // Security check: ensure the path is within the log directory
    if !log_file.starts_with(&log_dir) {
        tracing::warn!(filename = %filename, "Attempted to access file outside log directory");
        return Err("Invalid log file path".to_string());
    }

    if !log_file.exists() {
        return Err(format!("Log file '{}' not found", filename));
    }

    std::fs::read_to_string(&log_file).map_err(|e| format!("Failed to read log file: {}", e))
}
