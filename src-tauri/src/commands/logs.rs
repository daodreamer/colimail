use crate::logger;
use std::fs::File;
use std::io::Write;
use tauri::command;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

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

/// Export all log files as a ZIP archive
///
/// Creates a ZIP file containing all log files in the application's data directory.
/// The ZIP file is saved with a timestamped filename for easy identification.
///
/// # Returns
/// * `Ok(String)` - The absolute path to the created ZIP file
/// * `Err(String)` - Error message if the export fails
#[command]
pub fn export_logs_as_zip() -> Result<String, String> {
    tracing::info!("Starting log export to ZIP");

    // Get log directory
    let log_dir = logger::get_log_dir()?;

    // Get project directory (parent of logs directory)
    let project_dir = log_dir
        .parent()
        .ok_or_else(|| "Failed to get project directory".to_string())?;

    // Create timestamp for ZIP filename
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let zip_filename = format!("colimail_logs_{}.zip", timestamp);
    let zip_path = project_dir.join(&zip_filename);

    tracing::info!(
        zip_path = %zip_path.display(),
        "Creating ZIP archive"
    );

    // Create ZIP file
    let zip_file =
        File::create(&zip_path).map_err(|e| format!("Failed to create ZIP file: {}", e))?;

    let mut zip = ZipWriter::new(zip_file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(6));

    // Get all log files
    let log_files = match std::fs::read_dir(&log_dir) {
        Ok(entries) => {
            let mut files = Vec::new();
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("log") {
                    files.push(path);
                }
            }
            files
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to read log directory");
            return Err(format!("Failed to read log directory: {}", e));
        }
    };

    tracing::info!(file_count = log_files.len(), "Found log files to compress");

    // Add each log file to the ZIP
    for log_file_path in log_files {
        let filename = log_file_path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| "Invalid log file name".to_string())?;

        tracing::debug!(filename = %filename, "Adding file to ZIP");

        // Start a new file in the ZIP
        zip.start_file(filename, options)
            .map_err(|e| format!("Failed to add file '{}' to ZIP: {}", filename, e))?;

        // Read and write the file content
        let content = std::fs::read(&log_file_path)
            .map_err(|e| format!("Failed to read log file '{}': {}", filename, e))?;

        zip.write_all(&content)
            .map_err(|e| format!("Failed to write file '{}' to ZIP: {}", filename, e))?;
    }

    // Add a README file with instructions
    let readme_content = format!(
        "Colimail Log Export\n\
        ===================\n\n\
        Export Date: {}\n\
        Application Version: {}\n\n\
        These log files have been exported for debugging purposes.\n\
        Please attach this ZIP file when reporting issues to the development team.\n\n\
        Files included:\n\
        - All log files from the application's log directory\n\
        - Log files use JSON format for easy parsing\n\n\
        For more information, visit:\n\
        https://github.com/daodreamer/colimail\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        env!("CARGO_PKG_VERSION")
    );

    zip.start_file("README.txt", options)
        .map_err(|e| format!("Failed to add README to ZIP: {}", e))?;

    zip.write_all(readme_content.as_bytes())
        .map_err(|e| format!("Failed to write README to ZIP: {}", e))?;

    // Finish writing the ZIP
    zip.finish()
        .map_err(|e| format!("Failed to finalize ZIP file: {}", e))?;

    let zip_path_str = zip_path.to_string_lossy().to_string();

    tracing::info!(
        zip_path = %zip_path_str,
        "Log export completed successfully"
    );

    Ok(zip_path_str)
}
