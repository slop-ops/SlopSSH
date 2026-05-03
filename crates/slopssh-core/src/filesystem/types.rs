use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileType {
    File,
    Directory,
    Symlink,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttributes {
    pub size: u64,
    pub file_type: FileType,
    pub modified: Option<i64>,
    pub permissions: Option<u32>,
    pub uid: Option<u32>,
    pub gid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub attributes: FileAttributes,
}

#[derive(Debug, thiserror::Error)]
pub enum FileSystemError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    #[error("Not a directory: {0}")]
    NotDirectory(String),
    #[error("IO error: {0}")]
    IoError(String),
    #[error("SFTP error: {0}")]
    SftpError(String),
    #[error("{0}")]
    Other(String),
}

impl From<std::io::Error> for FileSystemError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => FileSystemError::NotFound(e.to_string()),
            std::io::ErrorKind::PermissionDenied => {
                FileSystemError::PermissionDenied(e.to_string())
            }
            std::io::ErrorKind::AlreadyExists => FileSystemError::AlreadyExists(e.to_string()),
            _ => FileSystemError::IoError(e.to_string()),
        }
    }
}

#[async_trait::async_trait]
pub trait FileSystem: Send + Sync {
    async fn list_dir(&self, path: &str) -> Result<Vec<DirEntry>, FileSystemError>;
    async fn stat(&self, path: &str) -> Result<FileAttributes, FileSystemError>;
    async fn mkdir(&self, path: &str) -> Result<(), FileSystemError>;
    async fn remove(&self, path: &str) -> Result<(), FileSystemError>;
    async fn rename(&self, from: &str, to: &str) -> Result<(), FileSystemError>;
    async fn read_file(&self, path: &str) -> Result<Vec<u8>, FileSystemError>;
    async fn write_file(&self, path: &str, data: &[u8]) -> Result<(), FileSystemError>;
    async fn exists(&self, path: &str) -> Result<bool, FileSystemError>;
    fn home_dir(&self) -> String;
    fn separator(&self) -> char;
}
