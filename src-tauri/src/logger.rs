use directories::ProjectDirs;
use std::path::PathBuf;
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Get the log directory for the application
pub fn get_log_dir() -> Result<PathBuf, String> {
    let project_dirs = ProjectDirs::from("", "", "Colimail")
        .ok_or_else(|| "Failed to determine project directories".to_string())?;

    let log_dir = project_dirs.data_dir().join("logs");

    // Create logs directory if it doesn't exist
    std::fs::create_dir_all(&log_dir)
        .map_err(|e| format!("Failed to create log directory: {}", e))?;

    Ok(log_dir)
}

/// Initialize the logging system
///
/// This sets up:
/// - File logging with daily rotation (keeps last 7 days)
/// - Console logging (only in debug mode)
/// - Appropriate log levels for production vs development
pub fn init() -> Result<(), String> {
    let log_dir = get_log_dir()?;

    // Create a rolling file appender (rotates daily, keeps last 7 days)
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("colimail")
        .filename_suffix("log")
        .max_log_files(7)
        .build(&log_dir)
        .map_err(|e| format!("Failed to create file appender: {}", e))?;

    // Determine log level based on build type
    #[cfg(debug_assertions)]
    let default_level = Level::DEBUG;

    #[cfg(not(debug_assertions))]
    let default_level = Level::INFO;

    // Create environment filter
    // Users can override log level by setting RUST_LOG environment variable
    // Example: RUST_LOG=debug or RUST_LOG=colimail=trace
    let env_filter = EnvFilter::builder()
        .with_default_directive(default_level.into())
        .from_env_lossy()
        // Filter out noisy dependencies in production
        .add_directive("hyper=warn".parse().unwrap())
        .add_directive("reqwest=warn".parse().unwrap())
        .add_directive("sqlx=warn".parse().unwrap());

    // Create file logging layer with JSON format for easy parsing
    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_file(true)
        .json();

    // In debug mode, also log to console with pretty formatting
    #[cfg(debug_assertions)]
    {
        let console_layer = fmt::layer()
            .with_target(true)
            .with_line_number(true)
            .pretty();

        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .with(console_layer)
            .init();
    }

    // In release mode, only log to file
    #[cfg(not(debug_assertions))]
    {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .init();
    }

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        log_dir = %log_dir.display(),
        "Logging system initialized"
    );

    Ok(())
}

/// Get the path to the current log file
#[allow(dead_code)]
pub fn get_current_log_file() -> Result<PathBuf, String> {
    let log_dir = get_log_dir()?;
    let today = chrono::Local::now().format("%Y-%m-%d");
    let log_file = log_dir.join(format!("colimail.{}.log", today));
    Ok(log_file)
}

/// Read the most recent log entries
#[allow(dead_code)]
pub fn read_recent_logs(lines: usize) -> Result<String, String> {
    let log_file = get_current_log_file()?;

    if !log_file.exists() {
        return Ok("No log file found for today.".to_string());
    }

    let content = std::fs::read_to_string(&log_file)
        .map_err(|e| format!("Failed to read log file: {}", e))?;

    // Get the last N lines
    let all_lines: Vec<&str> = content.lines().collect();
    let start = all_lines.len().saturating_sub(lines);
    let recent_lines = all_lines[start..].join("\n");

    Ok(recent_lines)
}
