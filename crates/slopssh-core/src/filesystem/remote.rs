use std::sync::Arc;
use std::time::UNIX_EPOCH;

use russh_sftp::client::SftpSession;
use tokio::sync::Mutex;

use super::types::{DirEntry, FileAttributes, FileSystem, FileSystemError, FileType};

pub struct RemoteFileSystem {
    sftp: Arc<Mutex<Option<SftpSession>>>,
}

impl RemoteFileSystem {
    pub fn new(sftp: SftpSession) -> Self {
        Self {
            sftp: Arc::new(Mutex::new(Some(sftp))),
        }
    }

    pub async fn close(&self) {
        let mut guard = self.sftp.lock().await;
        if let Some(sftp) = guard.take() {
            let _ = sftp.close().await;
        }
    }
}

fn convert_file_type(ft: &russh_sftp::protocol::FileType) -> FileType {
    if ft.is_dir() {
        FileType::Directory
    } else if ft.is_symlink() {
        FileType::Symlink
    } else if ft.is_file() {
        FileType::File
    } else {
        FileType::Other
    }
}

fn convert_metadata(meta: &russh_sftp::client::fs::Metadata) -> FileAttributes {
    let modified = meta.modified().ok().and_then(|t| {
        t.duration_since(UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs() as i64)
    });
    FileAttributes {
        size: meta.len(),
        file_type: convert_file_type(&meta.file_type()),
        modified,
        permissions: meta.permissions,
        uid: meta.uid,
        gid: meta.gid,
    }
}

fn convert_sftp_error(e: russh_sftp::client::error::Error) -> FileSystemError {
    match &e {
        russh_sftp::client::error::Error::Status(status) => match status.status_code {
            russh_sftp::protocol::StatusCode::NoSuchFile => {
                FileSystemError::NotFound(status.error_message.clone())
            }
            russh_sftp::protocol::StatusCode::PermissionDenied => {
                FileSystemError::PermissionDenied(status.error_message.clone())
            }
            _ => FileSystemError::SftpError(e.to_string()),
        },
        _ => FileSystemError::SftpError(e.to_string()),
    }
}

#[async_trait::async_trait]
impl FileSystem for RemoteFileSystem {
    async fn list_dir(&self, path: &str) -> Result<Vec<DirEntry>, FileSystemError> {
        let guard = self.sftp.lock().await;
        let sftp = guard
            .as_ref()
            .ok_or_else(|| FileSystemError::SftpError("SFTP session closed".to_string()))?;

        let read_dir = sftp.read_dir(path).await.map_err(convert_sftp_error)?;

        let mut entries = Vec::new();
        for entry in read_dir {
            let name = entry.file_name();
            let meta = entry.metadata();

            let entry_path = if path == "/" {
                format!("/{}", name)
            } else {
                format!("{}/{}", path, name)
            };

            entries.push(DirEntry {
                name,
                path: entry_path,
                attributes: convert_metadata(&meta),
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
        let guard = self.sftp.lock().await;
        let sftp = guard
            .as_ref()
            .ok_or_else(|| FileSystemError::SftpError("SFTP session closed".to_string()))?;

        let meta = sftp.metadata(path).await.map_err(convert_sftp_error)?;

        Ok(convert_metadata(&meta))
    }

    async fn mkdir(&self, path: &str) -> Result<(), FileSystemError> {
        let guard = self.sftp.lock().await;
        let sftp = guard
            .as_ref()
            .ok_or_else(|| FileSystemError::SftpError("SFTP session closed".to_string()))?;

        sftp.create_dir(path).await.map_err(convert_sftp_error)
    }

    async fn remove(&self, path: &str) -> Result<(), FileSystemError> {
        let guard = self.sftp.lock().await;
        let sftp = guard
            .as_ref()
            .ok_or_else(|| FileSystemError::SftpError("SFTP session closed".to_string()))?;

        let meta = sftp.metadata(path).await.map_err(convert_sftp_error)?;

        if meta.is_dir() {
            sftp.remove_dir(path).await.map_err(convert_sftp_error)
        } else {
            sftp.remove_file(path).await.map_err(convert_sftp_error)
        }
    }

    async fn rename(&self, from: &str, to: &str) -> Result<(), FileSystemError> {
        let guard = self.sftp.lock().await;
        let sftp = guard
            .as_ref()
            .ok_or_else(|| FileSystemError::SftpError("SFTP session closed".to_string()))?;

        sftp.rename(from, to).await.map_err(convert_sftp_error)
    }

    async fn read_file(&self, path: &str) -> Result<Vec<u8>, FileSystemError> {
        let guard = self.sftp.lock().await;
        let sftp = guard
            .as_ref()
            .ok_or_else(|| FileSystemError::SftpError("SFTP session closed".to_string()))?;

        sftp.read(path).await.map_err(convert_sftp_error)
    }

    async fn write_file(&self, path: &str, data: &[u8]) -> Result<(), FileSystemError> {
        let guard = self.sftp.lock().await;
        let sftp = guard
            .as_ref()
            .ok_or_else(|| FileSystemError::SftpError("SFTP session closed".to_string()))?;

        sftp.write(path, data).await.map_err(convert_sftp_error)
    }

    async fn exists(&self, path: &str) -> Result<bool, FileSystemError> {
        let guard = self.sftp.lock().await;
        let sftp = guard
            .as_ref()
            .ok_or_else(|| FileSystemError::SftpError("SFTP session closed".to_string()))?;

        sftp.try_exists(path).await.map_err(convert_sftp_error)
    }

    fn home_dir(&self) -> String {
        "/".to_string()
    }

    fn separator(&self) -> char {
        '/'
    }
}
