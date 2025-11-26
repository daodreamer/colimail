use crate::commands::get_minimize_to_tray;
use crate::oauth2_config;
use tauri::{App, AppHandle, Emitter, Manager};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::UpdaterExt;

/// Initialize application setup, including plugins, deep links, notifications, and updates
pub fn init_app(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize OAuth2 credentials
    oauth2_config::init_credentials(app.handle());

    // Setup deep link handler for OAuth callbacks
    setup_deep_link_handler(app)?;

    // Request notification permission (important for Windows)
    setup_notification_permission(app);

    // Handle window close event - minimize to tray or close based on user setting
    setup_window_close_handler(app)?;

    // Check for updates on startup
    spawn_update_checker(app.handle().clone());

    tracing::info!("Application setup completed successfully");

    Ok(())
}

/// Setup deep link handler for OAuth callbacks
fn setup_deep_link_handler(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    app.deep_link().register_all()?;

    let handle = app.handle().clone();
    app.deep_link().on_open_url(move |event| {
        // event.urls() consumes event, so call it once and store result
        let urls = event.urls();
        if urls.is_empty() {
            tracing::warn!("Deep link plugin: on_open_url called but no URLs provided");
            return;
        }

        tracing::info!(
            "Deep link plugin: on_open_url triggered with URL: {}",
            urls[0]
        );

        let url = &urls[0];
        if url.scheme() == "colimail" && url.host_str() == Some("auth") {
            // Extract authorization code from query params
            if let Some(code) = url.query_pairs().find(|(key, _)| key == "code") {
                tracing::info!(
                    "Deep link plugin: OAuth code found, emitting oauth-code-received event"
                );

                // Emit event to frontend with the code
                if let Some(window) = handle.get_webview_window("main") {
                    let code_str = code.1.to_string();
                    tracing::info!("Deep link plugin: Emitting event with code: {}", code_str);
                    match window.emit("oauth-code-received", code_str) {
                        Ok(_) => tracing::info!("Deep link plugin: Event emitted successfully"),
                        Err(e) => tracing::error!("Deep link plugin: Failed to emit event: {}", e),
                    }
                    let _ = window.show();
                    let _ = window.set_focus();
                } else {
                    tracing::error!("Deep link plugin: Failed to get main window");
                }
            } else {
                tracing::info!("Deep link plugin: No code parameter found in URL");
            }
        } else {
            tracing::info!(
                "Deep link plugin: URL is not an auth callback: scheme={}, host={:?}",
                url.scheme(),
                url.host_str()
            );
        }
    });

    Ok(())
}

/// Setup notification permission request
fn setup_notification_permission(app: &App) {
    let notification = app.handle().notification();

    // Check and request permission
    match notification.permission_state() {
        Ok(state) => {
            tracing::info!(?state, "Notification permission state");
            if state.to_string() != "granted" {
                tracing::info!("Requesting notification permission");
                match notification.request_permission() {
                    Ok(_) => tracing::info!("Notification permission requested"),
                    Err(e) => {
                        tracing::error!(
                            error = %e,
                            "Failed to request notification permission"
                        );
                    }
                }
            }
        }
        Err(e) => tracing::error!(error = %e, "Failed to check notification permission"),
    }
}

/// Setup window close handler for minimize to tray functionality
fn setup_window_close_handler(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window("main") {
        let app_handle = app.handle().clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let app_handle_clone = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    // Check user preference for minimize to tray
                    let minimize_to_tray = get_minimize_to_tray().await.unwrap_or(true);

                    if minimize_to_tray {
                        // Hide window instead of closing
                        if let Some(window) = app_handle_clone.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    } else {
                        // Exit the application
                        app_handle_clone.exit(0);
                    }
                });

                // Prevent default close behavior
                api.prevent_close();
            }
        });
    }

    Ok(())
}

/// Spawn background task to check for application updates
fn spawn_update_checker(app_handle: AppHandle) {
    tokio::spawn(async move {
        // Wait a bit for the app to fully initialize
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        tracing::info!("Checking for application updates");

        match app_handle.updater() {
            Ok(updater_builder) => match updater_builder.check().await {
                Ok(Some(update)) => {
                    tracing::info!(
                        version = %update.version,
                        current_version = %update.current_version,
                        "Update available"
                    );

                    // Emit event to frontend to notify user about update
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let update_info = serde_json::json!({
                            "version": update.version,
                            "current_version": update.current_version,
                            "body": update.body.unwrap_or_default(),
                            "date": update.date,
                        });
                        match window.emit("update-available", update_info) {
                            Ok(_) => tracing::info!("Update notification sent to frontend"),
                            Err(e) => tracing::error!(
                                error = %e,
                                "Failed to emit update-available event"
                            ),
                        }
                    } else {
                        tracing::error!("Failed to get main window for update notification");
                    }
                }
                Ok(None) => {
                    tracing::info!("Application is up to date");
                }
                Err(e) => {
                    // This is expected when there's no release yet or network issues
                    // Log as debug instead of error to avoid alarming users
                    tracing::debug!(
                        error = %e,
                        "Could not check for updates (this is normal if no releases exist yet)"
                    );
                }
            },
            Err(e) => {
                tracing::warn!(error = %e, "Failed to initialize updater");
            }
        }
    });
}

/// Setup single instance callback handler for deep links
pub fn setup_single_instance_callback(app: &AppHandle, args: Vec<String>) {
    tracing::info!("Single instance callback triggered, args: {:?}", args);

    // Check if args contain a deep link
    for arg in args.iter() {
        if arg.starts_with("colimail://") {
            tracing::info!("Processing deep link from second instance: {}", arg);

            // Parse the deep link
            if let Ok(url) = url::Url::parse(arg) {
                if url.scheme() == "colimail" && url.host_str() == Some("auth") {
                    // Extract authorization code from query params
                    if let Some(code) = url.query_pairs().find(|(key, _)| key == "code") {
                        tracing::info!("OAuth code received from second instance via deep link");

                        // Emit event to frontend with the code
                        if let Some(window) = app.get_webview_window("main") {
                            let code_str = code.1.to_string();
                            tracing::info!(
                                "Emitting oauth-code-received event with code: {}",
                                code_str
                            );
                            match window.emit("oauth-code-received", code_str) {
                                Ok(_) => tracing::info!("Event emitted successfully"),
                                Err(e) => tracing::error!("Failed to emit event: {}", e),
                            }
                            let _ = window.show();
                            let _ = window.set_focus();
                            let _ = window.unminimize();
                        } else {
                            tracing::error!("Failed to get main window");
                        }
                    }
                }
            }
        }
    }

    // Bring the existing window to front
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.unminimize();
    }
}
