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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_progress_new() {
        let p = TransferProgress::new("t1".to_string(), 1024);
        assert_eq!(p.id, "t1");
        assert_eq!(p.bytes_transferred, 0);
        assert_eq!(p.total_bytes, 1024);
        assert_eq!(p.status, TransferStatus::Queued);
        assert!(p.error.is_none());
        assert_eq!(p.speed_bps, 0.0);
    }

    #[test]
    fn test_percent_zero_total() {
        let p = TransferProgress::new("t1".to_string(), 0);
        assert_eq!(p.percent(), 100.0);
    }

    #[test]
    fn test_percent_half() {
        let mut p = TransferProgress::new("t1".to_string(), 1000);
        p.bytes_transferred = 500;
        assert_eq!(p.percent(), 50.0);
    }

    #[test]
    fn test_percent_full() {
        let mut p = TransferProgress::new("t1".to_string(), 1000);
        p.bytes_transferred = 1000;
        assert_eq!(p.percent(), 100.0);
    }

    #[test]
    fn test_percent_partial() {
        let mut p = TransferProgress::new("t1".to_string(), 200);
        p.bytes_transferred = 1;
        let pct = p.percent();
        assert!(pct > 0.0 && pct < 1.0);
    }

    #[test]
    fn test_transfer_status_equality() {
        assert_eq!(TransferStatus::Queued, TransferStatus::Queued);
        assert_ne!(TransferStatus::Queued, TransferStatus::InProgress);
        assert_ne!(TransferStatus::Failed, TransferStatus::Cancelled);
    }

    #[test]
    fn test_transfer_direction_equality() {
        assert_eq!(TransferDirection::Upload, TransferDirection::Upload);
        assert_eq!(TransferDirection::Download, TransferDirection::Download);
        assert_ne!(TransferDirection::Upload, TransferDirection::Download);
    }

    #[test]
    fn test_conflict_resolution_variants() {
        assert_eq!(ConflictResolution::Overwrite, ConflictResolution::Overwrite);
        assert_eq!(ConflictResolution::Skip, ConflictResolution::Skip);
        assert_eq!(ConflictResolution::Rename, ConflictResolution::Rename);
        assert_eq!(ConflictResolution::Prompt, ConflictResolution::Prompt);
        assert_ne!(ConflictResolution::Overwrite, ConflictResolution::Skip);
    }

    #[test]
    fn test_transfer_progress_serialize_deserialize() {
        let mut p = TransferProgress::new("t1".to_string(), 1024);
        p.bytes_transferred = 512;
        p.status = TransferStatus::InProgress;
        p.speed_bps = 1234.5;
        let json = serde_json::to_string(&p).unwrap();
        let parsed: TransferProgress = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "t1");
        assert_eq!(parsed.bytes_transferred, 512);
        assert_eq!(parsed.total_bytes, 1024);
        assert_eq!(parsed.status, TransferStatus::InProgress);
        assert_eq!(parsed.speed_bps, 1234.5);
    }

    #[test]
    fn test_transfer_status_serialize_deserialize() {
        for status in [
            TransferStatus::Queued,
            TransferStatus::InProgress,
            TransferStatus::Completed,
            TransferStatus::Failed,
            TransferStatus::Cancelled,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let parsed: TransferStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, status);
        }
    }

    #[test]
    fn test_transfer_direction_serialize_deserialize() {
        let json = serde_json::to_string(&TransferDirection::Upload).unwrap();
        assert!(json.contains("Upload"));
        let json = serde_json::to_string(&TransferDirection::Download).unwrap();
        assert!(json.contains("Download"));
    }

    #[test]
    fn test_transfer_progress_with_error() {
        let mut p = TransferProgress::new("t1".to_string(), 100);
        p.status = TransferStatus::Failed;
        p.error = Some("connection reset".to_string());
        assert_eq!(p.error.as_deref(), Some("connection reset"));
        assert_eq!(p.status, TransferStatus::Failed);
    }

    #[test]
    fn test_transfer_request_fields() {
        let req = TransferRequest {
            id: "r1".to_string(),
            session_id: "s1".to_string(),
            direction: TransferDirection::Download,
            source_path: "/remote/file.txt".to_string(),
            dest_path: "/local/file.txt".to_string(),
            file_size: 4096,
            conflict_resolution: ConflictResolution::Overwrite,
        };
        assert_eq!(req.id, "r1");
        assert_eq!(req.session_id, "s1");
        assert_eq!(req.direction, TransferDirection::Download);
        assert_eq!(req.file_size, 4096);
    }
}
