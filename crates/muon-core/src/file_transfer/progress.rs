use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransferDirection {
    Upload,
    Download,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransferStatus {
    Queued,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictResolution {
    Overwrite,
    Skip,
    Rename,
    Prompt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequest {
    pub id: String,
    pub session_id: String,
    pub direction: TransferDirection,
    pub source_path: String,
    pub dest_path: String,
    pub file_size: u64,
    pub conflict_resolution: ConflictResolution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferProgress {
    pub id: String,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub status: TransferStatus,
    pub error: Option<String>,
    pub speed_bps: f64,
}

impl TransferProgress {
    pub fn new(id: String, total_bytes: u64) -> Self {
        Self {
            id,
            bytes_transferred: 0,
            total_bytes,
            status: TransferStatus::Queued,
            error: None,
            speed_bps: 0.0,
        }
    }

    pub fn percent(&self) -> f64 {
        if self.total_bytes == 0 {
            return 100.0;
        }
        (self.bytes_transferred as f64 / self.total_bytes as f64) * 100.0
    }
}
