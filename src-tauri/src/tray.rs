use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Emitter, Manager,
};

/// Setup system tray with menu and event handlers
pub fn setup_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&settings_item, &quit_item])?;

    // Get the default window icon, with fallback
    let icon = get_default_icon(app)?;

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(on_menu_event)
        .on_tray_icon_event(on_tray_icon_event)
        .build(app)?;

    tracing::info!("System tray initialized successfully");

    Ok(())
}

/// Get the default window icon with proper error handling
fn get_default_icon(app: &App) -> Result<Image<'_>, Box<dyn std::error::Error>> {
    app.default_window_icon()
        .ok_or_else(|| {
            let err = "Default window icon not found";
            tracing::error!("{}", err);
            err.into()
        })
        .cloned()
}

/// Handle system tray menu events
fn on_menu_event(app: &tauri::AppHandle, event: tauri::menu::MenuEvent) {
    match event.id.as_ref() {
        "settings" => {
            if let Some(window) = app.get_webview_window("main") {
                if let Err(e) = window.show() {
                    tracing::error!(error = %e, "Failed to show window");
                }
                if let Err(e) = window.set_focus() {
                    tracing::error!(error = %e, "Failed to set focus");
                }
                // Emit event to open settings dialog
                if let Err(e) = window.emit("open-settings", ()) {
                    tracing::error!(error = %e, "Failed to emit open-settings event");
                }
            } else {
                tracing::error!("Failed to get main window");
            }
        }
        "quit" => {
            tracing::info!("Quit action triggered from system tray");
            app.exit(0);
        }
        other => {
            tracing::debug!("Unknown menu event: {}", other);
        }
    }
}

/// Handle system tray icon click events
fn on_tray_icon_event(tray: &tauri::tray::TrayIcon, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        // Left click: toggle window visibility
        let app = tray.app_handle();
        if let Some(window) = app.get_webview_window("main") {
            match window.is_visible() {
                Ok(true) => {
                    if let Err(e) = window.hide() {
                        tracing::error!(error = %e, "Failed to hide window");
                    }
                }
                Ok(false) => {
                    if let Err(e) = window.show() {
                        tracing::error!(error = %e, "Failed to show window");
                    }
                    if let Err(e) = window.set_focus() {
                        tracing::error!(error = %e, "Failed to set focus");
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to check window visibility");
                }
            }
        } else {
            tracing::error!("Failed to get main window");
        }
    }
}
