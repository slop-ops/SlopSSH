use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorInfo {
    pub name: String,
    pub command: String,
    pub path: Option<String>,
}

pub fn detect_editors() -> Vec<EditorInfo> {
    let editors = [
        ("VS Code", "code"),
        ("VS Code Insiders", "code-insiders"),
        ("Cursor", "cursor"),
        ("Vim", "vim"),
        ("Neovim", "nvim"),
        ("Nano", "nano"),
        ("Emacs", "emacs"),
        ("Micro", "micro"),
        ("Helix", "hx"),
        ("Sublime Text", "subl"),
        ("Atom", "atom"),
        ("Kate", "kate"),
        ("Gedit", "gedit"),
        ("Mousepad", "mousepad"),
    ];

    let mut found = Vec::new();
    for (name, cmd) in &editors {
        if let Some(path) = which_command(cmd) {
            found.push(EditorInfo {
                name: name.to_string(),
                command: cmd.to_string(),
                path: Some(path),
            });
        }
    }
    found
}

pub fn resolve_editor(configured: &str) -> String {
    if !configured.is_empty() {
        if let Some(path) = which_command(configured) {
            return path;
        }
        if Path::new(configured).exists() {
            return configured.to_string();
        }
    }

    let preferred = ["code", "nvim", "vim", "nano"];
    for cmd in &preferred {
        if let Some(path) = which_command(cmd) {
            return path;
        }
    }

    "vi".to_string()
}

fn which_command(name: &str) -> Option<String> {
    Command::new("which")
        .arg(name)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            String::from_utf8(o.stdout)
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
}

pub fn open_in_editor(editor_cmd: &str, file_path: &str) -> std::io::Result<()> {
    let resolved = resolve_editor(editor_cmd);
    Command::new(&resolved).arg(file_path).spawn()?;
    Ok(())
}
