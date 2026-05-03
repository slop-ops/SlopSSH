use tauri::{
    AppHandle, Manager,
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

pub fn create_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, tauri::Error> {
    let file_menu = Submenu::with_items(
        app,
        "File",
        true,
        &[
            &MenuItem::with_id(app, "new_session", "New Session", true, None::<&str>)?,
            &MenuItem::with_id(
                app,
                "import_sessions",
                "Import Sessions",
                true,
                None::<&str>,
            )?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "close_tab", "Close Tab", true, Some("Ctrl+W"))?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "quit", "Quit", true, Some("Ctrl+Q"))?,
        ],
    )?;

    let edit_menu = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &MenuItem::with_id(app, "copy", "Copy", true, Some("Ctrl+Shift+C"))?,
            &MenuItem::with_id(app, "paste", "Paste", true, Some("Ctrl+Shift+V"))?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "select_all", "Select All", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "settings", "Settings", true, Some("Ctrl+,"))?,
        ],
    )?;

    let session_menu = Submenu::with_items(
        app,
        "Session",
        true,
        &[
            &MenuItem::with_id(app, "connect", "Connect", true, None::<&str>)?,
            &MenuItem::with_id(app, "disconnect", "Disconnect", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "duplicate", "Duplicate Session", true, None::<&str>)?,
            &MenuItem::with_id(app, "delete_session", "Delete Session", true, None::<&str>)?,
        ],
    )?;

    let view_menu = Submenu::with_items(
        app,
        "View",
        true,
        &[
            &MenuItem::with_id(
                app,
                "toggle_sidebar",
                "Toggle Sidebar",
                true,
                Some("Ctrl+B"),
            )?,
            &MenuItem::with_id(app, "local_terminal", "Local Terminal", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "zoom_in", "Zoom In", true, Some("Ctrl+="))?,
            &MenuItem::with_id(app, "zoom_out", "Zoom Out", true, Some("Ctrl+-"))?,
            &MenuItem::with_id(app, "zoom_reset", "Reset Zoom", true, Some("Ctrl+0"))?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "fullscreen", "Toggle Fullscreen", true, Some("F11"))?,
        ],
    )?;

    let tools_menu = Submenu::with_items(
        app,
        "Tools",
        true,
        &[
            &MenuItem::with_id(app, "file_browser", "File Browser", true, None::<&str>)?,
            &MenuItem::with_id(app, "process_viewer", "Process Viewer", true, None::<&str>)?,
            &MenuItem::with_id(app, "log_viewer", "Log Viewer", true, None::<&str>)?,
            &MenuItem::with_id(app, "disk_analyzer", "Disk Analyzer", true, None::<&str>)?,
            &MenuItem::with_id(app, "search", "Search", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(
                app,
                "port_forwarding",
                "Port Forwarding",
                true,
                None::<&str>,
            )?,
            &MenuItem::with_id(app, "port_viewer", "Port Viewer", true, None::<&str>)?,
            &MenuItem::with_id(app, "key_manager", "SSH Key Manager", true, None::<&str>)?,
        ],
    )?;

    let help_menu = Submenu::with_items(
        app,
        "Help",
        true,
        &[
            &MenuItem::with_id(app, "about", "About SlopSSH", true, None::<&str>)?,
            &MenuItem::with_id(
                app,
                "check_updates",
                "Check for Updates",
                true,
                None::<&str>,
            )?,
        ],
    )?;

    Menu::with_items(
        app,
        &[
            &file_menu,
            &edit_menu,
            &session_menu,
            &view_menu,
            &tools_menu,
            &help_menu,
        ],
    )
}

pub fn create_tray(app: &AppHandle) -> Result<(), tauri::Error> {
    let show = MenuItem::with_id(app, "tray_show", "Show Window", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "tray_quit", "Quit", true, None::<&str>)?;

    let tray_menu = Menu::with_items(app, &[&show, &quit])?;

    TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().cloned().unwrap_or_else(|| {
            tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png")).unwrap_or_else(
                |_| {
                    let empty: &[u8] = &[0, 0, 0, 0];
                    tauri::image::Image::new(empty, 1, 1)
                },
            )
        }))
        .tooltip("SlopSSH")
        .menu(&tray_menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "tray_show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "tray_quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
