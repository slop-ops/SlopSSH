use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use tokio::fs;
use tokio::io::AsyncWriteExt;

use super::types::{DirEntry, FileAttributes, FileSystem, FileSystemError, FileType};

pub struct LocalFileSystem {
    root: Option<PathBuf>,
}

impl LocalFileSystem {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn with_root(root: PathBuf) -> Self {
        Self { root: Some(root) }
    }

    fn resolve(&self, path: &str) -> PathBuf {
        let p = Path::new(path);
        if p.is_absolute() {
            p.to_path_buf()
        } else if let Some(root) = &self.root {
            root.join(path)
        } else {
            p.to_path_buf()
        }
    }
}

impl Default for LocalFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl FileSystem for LocalFileSystem {
    async fn list_dir(&self, path: &str) -> Result<Vec<DirEntry>, FileSystemError> {
        let full_path = self.resolve(path);
        let mut entries = Vec::new();
        let mut dir = fs::read_dir(&full_path)
            .await
            .map_err(FileSystemError::from)?;

        while let Some(entry) = dir.next_entry().await.map_err(FileSystemError::from)? {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "." || name == ".." {
                continue;
            }

            let metadata = match entry.metadata().await {
                Ok(m) => m,
                Err(_) => continue,
            };

            let file_type = if metadata.is_dir() {
                FileType::Directory
            } else if metadata.is_symlink() {
                FileType::Symlink
            } else {
                FileType::File
            };

            let modified = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64);

            let entry_path = full_path.join(&name).to_string_lossy().to_string();

            entries.push(DirEntry {
                name,
                path: entry_path,
                attributes: FileAttributes {
                    size: metadata.len(),
                    file_type,
                    modified,
                    permissions: None,
                    uid: None,
                    gid: None,
                },
            });
        }

        entries.sort_by(
            |a, b| match (&a.attributes.file_type, &b.attributes.file_type) {
                (FileType::Directory, FileType::File) => std::cmp::Ordering::Less,
                (FileType::File, FileType::Directory) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            },
        );

        Ok(entries)
    }

    async fn stat(&self, path: &str) -> Result<FileAttributes, FileSystemError> {
        let full_path = self.resolve(path);
        let metadata = fs::metadata(&full_path)
            .await
            .map_err(FileSystemError::from)?;

        let file_type = if metadata.is_dir() {
            FileType::Directory
        } else if metadata.is_symlink() {
            FileType::Symlink
        } else {
            FileType::File
        };

        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64);

        Ok(FileAttributes {
            size: metadata.len(),
            file_type,
            modified,
            permissions: None,
            uid: None,
            gid: None,
        })
    }

    async fn mkdir(&self, path: &str) -> Result<(), FileSystemError> {
        let full_path = self.resolve(path);
        fs::create_dir(&full_path)
            .await
            .map_err(FileSystemError::from)
    }

    async fn remove(&self, path: &str) -> Result<(), FileSystemError> {
        let full_path = self.resolve(path);
        let metadata = fs::metadata(&full_path)
            .await
            .map_err(FileSystemError::from)?;
        if metadata.is_dir() {
            fs::remove_dir(&full_path)
                .await
                .map_err(FileSystemError::from)
        } else {
            fs::remove_file(&full_path)
                .await
                .map_err(FileSystemError::from)
        }
    }

    async fn rename(&self, from: &str, to: &str) -> Result<(), FileSystemError> {
        let from_path = self.resolve(from);
        let to_path = self.resolve(to);
        fs::rename(&from_path, &to_path)
            .await
            .map_err(FileSystemError::from)
    }

    async fn read_file(&self, path: &str) -> Result<Vec<u8>, FileSystemError> {
        let full_path = self.resolve(path);
        fs::read(&full_path).await.map_err(FileSystemError::from)
    }

    async fn write_file(&self, path: &str, data: &[u8]) -> Result<(), FileSystemError> {
        let full_path = self.resolve(path);
        let mut file = fs::File::create(&full_path)
            .await
            .map_err(FileSystemError::from)?;
        file.write_all(data).await.map_err(FileSystemError::from)?;
        file.flush().await.map_err(FileSystemError::from)
    }

    async fn exists(&self, path: &str) -> Result<bool, FileSystemError> {
        let full_path = self.resolve(path);
        Ok(full_path.exists())
    }

    fn home_dir(&self) -> String {
        dirs::home_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".to_string())
    }

    fn separator(&self) -> char {
        std::path::MAIN_SEPARATOR
    }
}
