use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

use russh_sftp::client::SftpSession;

use super::progress::{TransferProgress, TransferRequest, TransferStatus};

const CHUNK_SIZE: usize = 32768;

pub struct TransferEngine {
    active_transfers: Arc<Mutex<HashMap<String, TransferProgress>>>,
}

impl TransferEngine {
    pub fn new() -> Self {
        Self {
            active_transfers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_progress(&self, transfer_id: &str) -> Option<TransferProgress> {
        let transfers = self.active_transfers.lock().await;
        transfers.get(transfer_id).cloned()
    }

    pub async fn list_progress(&self) -> Vec<TransferProgress> {
        let transfers = self.active_transfers.lock().await;
        transfers.values().cloned().collect()
    }

    pub async fn cancel(&self, transfer_id: &str) -> bool {
        let mut transfers = self.active_transfers.lock().await;
        if let Some(p) = transfers.get_mut(transfer_id)
            && (p.status == TransferStatus::InProgress || p.status == TransferStatus::Queued)
        {
            p.status = TransferStatus::Cancelled;
            return true;
        }
        false
    }

    pub async fn remove(&self, transfer_id: &str) {
        let mut transfers = self.active_transfers.lock().await;
        transfers.remove(transfer_id);
    }

    pub async fn clear_completed(&self) {
        let mut transfers = self.active_transfers.lock().await;
        transfers.retain(|_, p| {
            p.status == TransferStatus::InProgress || p.status == TransferStatus::Queued
        });
    }

    pub fn spawn_upload(
        &self,
        request: TransferRequest,
        sftp: Arc<Mutex<Option<SftpSession>>>,
        on_progress: impl Fn(&TransferProgress) + Send + Sync + 'static,
    ) {
        let transfers = self.active_transfers.clone();
        let transfer_id = request.id.clone();
        let source = request.source_path.clone();
        let dest = request.dest_path.clone();
        tracing::info!(
            transfer_id = %transfer_id,
            source = %source,
            dest = %dest,
            "Upload started"
        );
        tokio::spawn(async move {
            let progress = TransferProgress::new(request.id.clone(), request.file_size);
            {
                let mut t = transfers.lock().await;
                t.insert(request.id.clone(), progress);
            }

            let result = perform_upload(&request, &sftp, &transfers, &on_progress).await;

            let mut t = transfers.lock().await;
            if let Some(p) = t.get_mut(&request.id) {
                match result {
                    Ok(()) => {
                        p.status = TransferStatus::Completed;
                        p.bytes_transferred = p.total_bytes;
                        tracing::info!(
                            transfer_id = %p.id,
                            bytes = p.total_bytes,
                            "Upload completed"
                        );
                        on_progress(p);
                    }
                    Err(e) => {
                        if p.status != TransferStatus::Cancelled {
                            p.status = TransferStatus::Failed;
                            p.error = Some(e.clone());
                            tracing::error!(
                                transfer_id = %p.id,
                                error = %e,
                                "Upload failed"
                            );
                            on_progress(p);
                        } else {
                            tracing::warn!(
                                transfer_id = %p.id,
                                "Upload cancelled"
                            );
                        }
                    }
                }
            }
        });
    }

    pub fn spawn_download(
        &self,
        request: TransferRequest,
        sftp: Arc<Mutex<Option<SftpSession>>>,
        on_progress: impl Fn(&TransferProgress) + Send + Sync + 'static,
    ) {
        let transfers = self.active_transfers.clone();
        let transfer_id = request.id.clone();
        let source = request.source_path.clone();
        let dest = request.dest_path.clone();
        tracing::info!(
            transfer_id = %transfer_id,
            source = %source,
            dest = %dest,
            "Download started"
        );
        tokio::spawn(async move {
            let progress = TransferProgress::new(request.id.clone(), request.file_size);
            {
                let mut t = transfers.lock().await;
                t.insert(request.id.clone(), progress);
            }

            let result = perform_download(&request, &sftp, &transfers, &on_progress).await;

            let mut t = transfers.lock().await;
            if let Some(p) = t.get_mut(&request.id) {
                match result {
                    Ok(()) => {
                        p.status = TransferStatus::Completed;
                        p.bytes_transferred = p.total_bytes;
                        tracing::info!(
                            transfer_id = %p.id,
                            bytes = p.total_bytes,
                            "Download completed"
                        );
                        on_progress(p);
                    }
                    Err(e) => {
                        if p.status != TransferStatus::Cancelled {
                            p.status = TransferStatus::Failed;
                            p.error = Some(e.clone());
                            tracing::error!(
                                transfer_id = %p.id,
                                error = %e,
                                "Download failed"
                            );
                            on_progress(p);
                        } else {
                            tracing::warn!(
                                transfer_id = %p.id,
                                "Download cancelled"
                            );
                        }
                    }
                }
            }
        });
    }
}

async fn perform_upload(
    request: &TransferRequest,
    sftp: &Arc<Mutex<Option<SftpSession>>>,
    transfers: &Arc<Mutex<HashMap<String, TransferProgress>>>,
    on_progress: &(dyn Fn(&TransferProgress) + Send + Sync),
) -> Result<(), String> {
    let mut local_file = tokio::fs::File::open(&request.source_path)
        .await
        .map_err(|e| format!("Failed to open local file: {}", e))?;

    let file_size = local_file
        .metadata()
        .await
        .map_err(|e| format!("Failed to read file metadata: {}", e))?
        .len();

    {
        let mut t = transfers.lock().await;
        if let Some(p) = t.get_mut(&request.id) {
            p.total_bytes = file_size;
            p.status = TransferStatus::InProgress;
            on_progress(p);
        }
    }

    let mut remote_file = {
        let guard = sftp.lock().await;
        let sftp_session = guard
            .as_ref()
            .ok_or_else(|| "SFTP session closed".to_string())?;
        tracing::debug!(
            transfer_id = %request.id,
            path = %request.dest_path,
            "Creating remote file for upload"
        );
        sftp_session
            .create(&request.dest_path)
            .await
            .map_err(|e| format!("Failed to create remote file: {}", e))?
    };

    let mut buf = vec![0u8; CHUNK_SIZE];
    let mut bytes_written: u64 = 0;
    let start = Instant::now();

    loop {
        {
            let t = transfers.lock().await;
            if let Some(p) = t.get(&request.id)
                && p.status == TransferStatus::Cancelled
            {
                return Ok(());
            }
        }

        let n = local_file
            .read(&mut buf)
            .await
            .map_err(|e| format!("Read error: {}", e))?;
        if n == 0 {
            break;
        }

        tokio::io::AsyncWriteExt::write_all(&mut remote_file, &buf[..n])
            .await
            .map_err(|e| format!("SFTP write error: {}", e))?;

        bytes_written += n as u64;

        let elapsed = start.elapsed().as_secs_f64();
        let speed = if elapsed > 0.0 {
            bytes_written as f64 / elapsed
        } else {
            0.0
        };

        {
            let mut t = transfers.lock().await;
            if let Some(p) = t.get_mut(&request.id) {
                p.bytes_transferred = bytes_written;
                p.speed_bps = speed;
                on_progress(p);
            }
        }
    }

    tokio::io::AsyncWriteExt::flush(&mut remote_file)
        .await
        .map_err(|e| format!("Flush error: {}", e))?;

    {
        use tokio::io::AsyncWriteExt;
        if let Err(e) = remote_file.shutdown().await {
            tracing::warn!(
                transfer_id = %request.id,
                error = %e,
                "Remote file shutdown error during upload"
            );
        }
    }

    Ok(())
}

async fn perform_download(
    request: &TransferRequest,
    sftp: &Arc<Mutex<Option<SftpSession>>>,
    transfers: &Arc<Mutex<HashMap<String, TransferProgress>>>,
    on_progress: &(dyn Fn(&TransferProgress) + Send + Sync),
) -> Result<(), String> {
    if let Some(parent) = std::path::Path::new(&request.dest_path).parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Failed to create parent dir: {}", e))?;
    }

    {
        let mut t = transfers.lock().await;
        if let Some(p) = t.get_mut(&request.id) {
            p.status = TransferStatus::InProgress;
            on_progress(p);
        }
    }

    let mut remote_file = {
        let guard = sftp.lock().await;
        let sftp_session = guard
            .as_ref()
            .ok_or_else(|| "SFTP session closed".to_string())?;
        tracing::debug!(
            transfer_id = %request.id,
            path = %request.source_path,
            "Opening remote file for streaming download"
        );
        sftp_session
            .open(&request.source_path)
            .await
            .map_err(|e| format!("SFTP open error: {}", e))?
    };

    let file_size = remote_file.metadata().await.map(|m| m.len()).unwrap_or(0);

    {
        let mut t = transfers.lock().await;
        if let Some(p) = t.get_mut(&request.id) {
            p.total_bytes = file_size;
        }
    }

    let mut local_file = tokio::fs::File::create(&request.dest_path)
        .await
        .map_err(|e| format!("Failed to create local file: {}", e))?;

    let mut buf = vec![0u8; CHUNK_SIZE];
    let mut written: u64 = 0;
    let start = Instant::now();

    loop {
        {
            let t = transfers.lock().await;
            if let Some(p) = t.get(&request.id)
                && p.status == TransferStatus::Cancelled
            {
                return Ok(());
            }
        }

        let n = remote_file
            .read(&mut buf)
            .await
            .map_err(|e| format!("SFTP read error: {}", e))?;
        if n == 0 {
            break;
        }

        local_file
            .write_all(&buf[..n])
            .await
            .map_err(|e| format!("Write error: {}", e))?;
        written += n as u64;

        let elapsed = start.elapsed().as_secs_f64();
        let speed = if elapsed > 0.0 {
            written as f64 / elapsed
        } else {
            0.0
        };

        {
            let mut t = transfers.lock().await;
            if let Some(p) = t.get_mut(&request.id) {
                p.bytes_transferred = written;
                p.speed_bps = speed;
                on_progress(p);
            }
        }
    }

    local_file
        .flush()
        .await
        .map_err(|e| format!("Flush error: {}", e))?;

    if file_size > 0 && written != file_size {
        tracing::warn!(
            transfer_id = %request.id,
            expected = file_size,
            actual = written,
            "Download size mismatch"
        );
    }

    Ok(())
}

impl Default for TransferEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_new() {
        let engine = TransferEngine::new();
        assert!(engine.list_progress().await.is_empty());
    }

    #[tokio::test]
    async fn test_engine_default() {
        let engine = TransferEngine::default();
        assert!(engine.list_progress().await.is_empty());
    }

    #[tokio::test]
    async fn test_get_progress_nonexistent() {
        let engine = TransferEngine::new();
        assert!(engine.get_progress("does-not-exist").await.is_none());
    }

    #[tokio::test]
    async fn test_list_progress_empty() {
        let engine = TransferEngine::new();
        let list = engine.list_progress().await;
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn test_cancel_nonexistent() {
        let engine = TransferEngine::new();
        assert!(!engine.cancel("does-not-exist").await);
    }

    #[tokio::test]
    async fn test_remove_nonexistent() {
        let engine = TransferEngine::new();
        engine.remove("does-not-exist").await;
        assert!(engine.list_progress().await.is_empty());
    }

    #[tokio::test]
    async fn test_clear_completed_empty() {
        let engine = TransferEngine::new();
        engine.clear_completed().await;
        assert!(engine.list_progress().await.is_empty());
    }

    #[tokio::test]
    async fn test_cancel_queued_transfer() {
        let engine = TransferEngine::new();
        let progress = TransferProgress::new("t1".to_string(), 100);
        engine
            .active_transfers
            .lock()
            .await
            .insert("t1".to_string(), progress);

        assert!(engine.cancel("t1").await);
        let p = engine.get_progress("t1").await.unwrap();
        assert_eq!(p.status, TransferStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_cancel_in_progress_transfer() {
        let engine = TransferEngine::new();
        let mut progress = TransferProgress::new("t2".to_string(), 100);
        progress.status = TransferStatus::InProgress;
        engine
            .active_transfers
            .lock()
            .await
            .insert("t2".to_string(), progress);

        assert!(engine.cancel("t2").await);
        let p = engine.get_progress("t2").await.unwrap();
        assert_eq!(p.status, TransferStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_cancel_completed_transfer_fails() {
        let engine = TransferEngine::new();
        let mut progress = TransferProgress::new("t3".to_string(), 100);
        progress.status = TransferStatus::Completed;
        engine
            .active_transfers
            .lock()
            .await
            .insert("t3".to_string(), progress);

        assert!(!engine.cancel("t3").await);
    }

    #[tokio::test]
    async fn test_cancel_failed_transfer_fails() {
        let engine = TransferEngine::new();
        let mut progress = TransferProgress::new("t4".to_string(), 100);
        progress.status = TransferStatus::Failed;
        engine
            .active_transfers
            .lock()
            .await
            .insert("t4".to_string(), progress);

        assert!(!engine.cancel("t4").await);
    }

    #[tokio::test]
    async fn test_remove_existing() {
        let engine = TransferEngine::new();
        let progress = TransferProgress::new("t5".to_string(), 100);
        engine
            .active_transfers
            .lock()
            .await
            .insert("t5".to_string(), progress);
        assert!(engine.get_progress("t5").await.is_some());

        engine.remove("t5").await;
        assert!(engine.get_progress("t5").await.is_none());
    }

    #[tokio::test]
    async fn test_clear_completed_removes_finished() {
        let engine = TransferEngine::new();

        let mut completed = TransferProgress::new("c1".to_string(), 100);
        completed.status = TransferStatus::Completed;
        let mut failed = TransferProgress::new("f1".to_string(), 100);
        failed.status = TransferStatus::Failed;
        let mut cancelled = TransferProgress::new("x1".to_string(), 100);
        cancelled.status = TransferStatus::Cancelled;
        let queued = TransferProgress::new("q1".to_string(), 100);
        let mut in_progress = TransferProgress::new("p1".to_string(), 100);
        in_progress.status = TransferStatus::InProgress;

        let mut transfers = engine.active_transfers.lock().await;
        transfers.insert("c1".to_string(), completed);
        transfers.insert("f1".to_string(), failed);
        transfers.insert("x1".to_string(), cancelled);
        transfers.insert("q1".to_string(), queued);
        transfers.insert("p1".to_string(), in_progress);
        drop(transfers);

        engine.clear_completed().await;

        let list = engine.list_progress().await;
        assert_eq!(list.len(), 2);
        let ids: Vec<&str> = list.iter().map(|p| p.id.as_str()).collect();
        assert!(ids.contains(&"q1"));
        assert!(ids.contains(&"p1"));
    }

    #[tokio::test]
    async fn test_list_progress_multiple() {
        let engine = TransferEngine::new();

        let p1 = TransferProgress::new("a".to_string(), 100);
        let p2 = TransferProgress::new("b".to_string(), 200);

        let mut transfers = engine.active_transfers.lock().await;
        transfers.insert("a".to_string(), p1);
        transfers.insert("b".to_string(), p2);
        drop(transfers);

        let list = engine.list_progress().await;
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_get_progress_existing() {
        let engine = TransferEngine::new();
        let mut progress = TransferProgress::new("t6".to_string(), 500);
        progress.bytes_transferred = 250;
        progress.status = TransferStatus::InProgress;

        engine
            .active_transfers
            .lock()
            .await
            .insert("t6".to_string(), progress.clone());

        let fetched = engine.get_progress("t6").await.unwrap();
        assert_eq!(fetched.id, "t6");
        assert_eq!(fetched.bytes_transferred, 250);
        assert_eq!(fetched.status, TransferStatus::InProgress);
    }

    #[tokio::test]
    async fn test_cancel_already_cancelled_fails() {
        let engine = TransferEngine::new();
        let mut progress = TransferProgress::new("t7".to_string(), 100);
        progress.status = TransferStatus::Cancelled;
        engine
            .active_transfers
            .lock()
            .await
            .insert("t7".to_string(), progress);

        assert!(!engine.cancel("t7").await);
    }
}
