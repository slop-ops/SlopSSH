use tauri::State;

use crate::AppState;

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    tracing::debug!("get_settings");
    let settings = state.settings.lock().await;
    serde_json::to_value(&*settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_settings(
    state: State<'_, AppState>,
    settings: serde_json::Value,
) -> Result<(), String> {
    tracing::info!("save_settings");
    let mut settings_guard = state.settings.lock().await;
    let mut new_settings: slopssh_core::config::settings::Settings =
        serde_json::from_value(settings).map_err(|e| e.to_string())?;
    slopssh_core::config::settings::SettingsManager::save(&mut new_settings)
        .map_err(|e| e.to_string())?;
    *settings_guard = new_settings;
    Ok(())
}

#[tauri::command]
pub fn detect_editors() -> Result<serde_json::Value, String> {
    tracing::debug!("detect_editors");
    let editors = slopssh_core::config::editor::detect_editors();
    serde_json::to_value(&editors).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_in_editor(state: State<'_, AppState>, file_path: String) -> Result<(), String> {
    tracing::debug!(file_path = %file_path, "open_in_editor");
    let settings = state.settings.lock().await;
    let editor = &settings.external_editor;
    slopssh_core::config::editor::open_in_editor(editor, &file_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_version() -> String {
    tracing::debug!("get_version");
    slopssh_core::version().to_string()
}

#[tauri::command]
pub async fn check_for_updates() -> Result<serde_json::Value, String> {
    tracing::info!("check_for_updates");
    let checker = slopssh_core::updater::github::UpdateChecker::new(
        "slop-ops",
        "SlopSSH",
        slopssh_core::version(),
    );
    match checker.check_for_update().await {
        Ok(Some(info)) => Ok(serde_json::json!({
            "has_update": info.is_newer,
            "version": info.latest_version,
            "download_url": info.download_url,
            "release_notes": info.release_notes,
            "release_url": info.release_url,
        })),
        Ok(None) => Ok(serde_json::json!({"has_update": false})),
        Err(e) => Err(format!("Update check failed: {}", e)),
    }
}

#[tauri::command]
pub async fn download_update(update_info: serde_json::Value) -> Result<serde_json::Value, String> {
    tracing::info!("download_update");
    let info: slopssh_core::updater::github::UpdateInfo =
        serde_json::from_value(update_info).map_err(|e| e.to_string())?;
    let checker = slopssh_core::updater::github::UpdateChecker::new(
        "slop-ops",
        "SlopSSH",
        slopssh_core::version(),
    );
    let path = checker
        .download_update(&info)
        .await
        .map_err(|e| format!("Download failed: {}", e))?;
    Ok(serde_json::json!({
        "path": path.to_string_lossy(),
        "size": std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0),
    }))
}

#[tauri::command]
pub async fn update_tray_tooltip(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let ssh_count = {
        let mgr = state.ssh_manager.lock().await;
        mgr.connected_session_ids().len()
    };
    let transfer_count = {
        let transfers = state.transfer_engine.list_progress().await;
        transfers
            .iter()
            .filter(|t| {
                t.status == slopssh_core::file_transfer::progress::TransferStatus::InProgress
            })
            .count()
    };
    let mut parts = vec!["SlopSSH".to_string()];
    if ssh_count > 0 {
        parts.push(format!(
            "{} active session{}",
            ssh_count,
            if ssh_count > 1 { "s" } else { "" }
        ));
    }
    if transfer_count > 0 {
        parts.push(format!(
            "{} transfer{}",
            transfer_count,
            if transfer_count > 1 { "s" } else { "" }
        ));
    }
    let tooltip = parts.join(" | ");
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_tooltip(Some(&tooltip));
    }
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SystemFontInfo {
    pub fonts: Vec<String>,
    pub default_font: String,
}

pub fn detect_system_monospace_fonts() -> Vec<String> {
    let mut fonts = Vec::new();

    #[cfg(target_os = "linux")]
    {
        if let Some(output) = std::process::Command::new("fc-list")
            .args(["--format=%{family}\n", ":spacing=mono"])
            .output()
            .ok()
            .filter(|o| o.status.success())
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                for family in line.split(',') {
                    let trimmed = family.trim();
                    if !trimmed.is_empty()
                        && !fonts
                            .iter()
                            .any(|f: &String| f.eq_ignore_ascii_case(trimmed))
                    {
                        fonts.push(trimmed.to_string());
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let known_mono = [
            "Cascadia Code",
            "Cascadia Mono",
            "Consolas",
            "Courier New",
            "Lucida Console",
            "JetBrains Mono",
            "Fira Code",
            "Source Code Pro",
            "DejaVu Sans Mono",
            "Liberation Mono",
        ];
        if let Some(output) = std::process::Command::new("reg")
            .args([
                "query",
                "HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Fonts",
            ])
            .output()
            .ok()
            .filter(|o| o.status.success())
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                for mono in &known_mono {
                    if line.to_lowercase().contains(&mono.to_lowercase())
                        && !fonts.iter().any(|f: &String| f.eq_ignore_ascii_case(mono))
                    {
                        fonts.push(mono.to_string());
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let known_macos_mono = [
            "Menlo",
            "Monaco",
            "SF Mono",
            "Courier New",
            "JetBrains Mono",
            "Fira Code",
        ];
        if let Some(output) = std::process::Command::new("fc-list")
            .args(["--format=%{family}\n", ":spacing=mono"])
            .output()
            .ok()
            .filter(|o| o.status.success())
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                for family in line.split(',') {
                    let trimmed = family.trim();
                    if !trimmed.is_empty()
                        && !fonts
                            .iter()
                            .any(|f: &String| f.eq_ignore_ascii_case(trimmed))
                    {
                        fonts.push(trimmed.to_string());
                    }
                }
            }
        }
        for mono in &known_macos_mono {
            if !fonts.iter().any(|f: &String| f.eq_ignore_ascii_case(mono)) {
                fonts.push(mono.to_string());
            }
        }
    }

    let default_candidates = [
        "JetBrains Mono",
        "Fira Code",
        "Cascadia Code",
        "Ubuntu Mono",
        "DejaVu Sans Mono",
        "Liberation Mono",
        "Consolas",
        "Menlo",
        "Monaco",
        "Courier New",
        "monospace",
    ];

    if fonts.is_empty() {
        for candidate in &default_candidates {
            fonts.push(candidate.to_string());
        }
    } else if !fonts.iter().any(|f| f == "monospace") {
        fonts.push("monospace".to_string());
    }

    fonts.sort_by(|a, b| {
        if a == "monospace" {
            std::cmp::Ordering::Greater
        } else if b == "monospace" {
            std::cmp::Ordering::Less
        } else {
            a.to_lowercase().cmp(&b.to_lowercase())
        }
    });

    fonts
}

pub fn get_default_monospace_font(installed: &[String]) -> String {
    let preference = [
        "JetBrains Mono",
        "Fira Code",
        "Cascadia Code",
        "Cascadia Mono",
        "Ubuntu Mono",
        "Ubuntu Sans Mono",
        "DejaVu Sans Mono",
        "Liberation Mono",
        "Consolas",
        "Menlo",
        "Monaco",
        "Courier New",
    ];
    for pref in &preference {
        if installed.iter().any(|f| f.eq_ignore_ascii_case(pref)) {
            return pref.to_string();
        }
    }

    #[cfg(target_os = "windows")]
    return "Cascadia Code".to_string();
    #[cfg(target_os = "macos")]
    return "Menlo".to_string();
    #[cfg(target_os = "linux")]
    return "Ubuntu Mono".to_string();
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    return "monospace".to_string();
}

#[tauri::command]
pub fn list_system_fonts() -> Result<SystemFontInfo, String> {
    tracing::debug!("list_system_fonts");
    let fonts = detect_system_monospace_fonts();
    let default_font = get_default_monospace_font(&fonts);
    Ok(SystemFontInfo {
        fonts,
        default_font,
    })
}
