//! System text-editor detection and invocation.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

/// Describes a detected text editor on the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorInfo {
    /// Human-readable editor name.
    pub name: String,
    /// Command used to launch the editor.
    pub command: String,
    /// Absolute path to the editor binary, if found.
    pub path: Option<String>,
}

/// Scans the system for known text editors and returns those found on `$PATH`.
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

/// Resolves the best available editor command.
///
/// If `configured` is non-empty, tries it first (as a command name or absolute
/// path). Otherwise falls back to `code`, `nvim`, `vim`, `nano`, and finally
/// `vi`.
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

/// Opens `file_path` in the resolved editor as a detached process.
pub fn open_in_editor(editor_cmd: &str, file_path: &str) -> std::io::Result<()> {
    let resolved = resolve_editor(editor_cmd);
    Command::new(&resolved).arg(file_path).spawn()?;
    Ok(())
}
