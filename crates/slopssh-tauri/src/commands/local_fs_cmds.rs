use slopssh_core::filesystem::local::LocalFileSystem;
use slopssh_core::filesystem::types::{FileSystem, FileType};

#[tauri::command]
pub async fn local_get_home() -> Result<String, String> {
    Ok(LocalFileSystem::get_home_dir())
}

#[tauri::command]
pub async fn local_get_drives() -> Result<Vec<String>, String> {
    Ok(LocalFileSystem::get_root_drives())
}

#[tauri::command]
pub async fn local_list_dir(path: Option<String>) -> Result<serde_json::Value, String> {
    let target_path = path.unwrap_or_else(LocalFileSystem::get_home_dir);
    let fs = LocalFileSystem::new();
    let entries = fs
        .list_dir(&target_path)
        .await
        .map_err(|e| format!("Failed to list directory '{}': {}", target_path, e))?;

    let json_entries: Vec<serde_json::Value> = entries
        .into_iter()
        .map(|entry| {
            serde_json::json!({
                "name": entry.name,
                "path": entry.path,
                "isDir": entry.attributes.file_type == FileType::Directory,
                "isFile": entry.attributes.file_type == FileType::File,
                "isSymlink": entry.attributes.file_type == FileType::Symlink,
                "size": entry.attributes.size,
                "modified": entry.attributes.modified.map(|s| s * 1000),
            })
        })
        .collect();

    Ok(serde_json::Value::Array(json_entries))
}

#[tauri::command]
pub async fn local_mkdir(path: String) -> Result<(), String> {
    let fs = LocalFileSystem::new();
    fs.mkdir(&path)
        .await
        .map_err(|e| format!("Failed to create directory '{}': {}", path, e))
}

#[tauri::command]
pub async fn local_remove(path: String) -> Result<(), String> {
    let fs = LocalFileSystem::new();
    fs.remove(&path)
        .await
        .map_err(|e| format!("Failed to remove '{}': {}", path, e))
}

#[tauri::command]
pub async fn local_rename(from: String, to: String) -> Result<(), String> {
    let fs = LocalFileSystem::new();
    fs.rename(&from, &to)
        .await
        .map_err(|e| format!("Failed to rename '{}' to '{}': {}", from, to, e))
}
