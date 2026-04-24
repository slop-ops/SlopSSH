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
                        on_progress(p);
                    }
                    Err(e) => {
                        if p.status != TransferStatus::Cancelled {
                            p.status = TransferStatus::Failed;
                            p.error = Some(e);
                            on_progress(p);
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
                        on_progress(p);
                    }
                    Err(e) => {
                        if p.status != TransferStatus::Cancelled {
                            p.status = TransferStatus::Failed;
                            p.error = Some(e);
                            on_progress(p);
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
    let mut file = tokio::fs::File::open(&request.source_path)
        .await
        .map_err(|e| format!("Failed to open local file: {}", e))?;

    let file_size = file
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

    let mut all_data = Vec::with_capacity(file_size as usize);
    let mut buf = vec![0u8; CHUNK_SIZE];
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

        let n = file
            .read(&mut buf)
            .await
            .map_err(|e| format!("Read error: {}", e))?;
        if n == 0 {
            break;
        }
        all_data.extend_from_slice(&buf[..n]);

        let elapsed = start.elapsed().as_secs_f64();
        let speed = if elapsed > 0.0 {
            all_data.len() as f64 / elapsed
        } else {
            0.0
        };

        {
            let mut t = transfers.lock().await;
            if let Some(p) = t.get_mut(&request.id) {
                p.bytes_transferred = all_data.len() as u64;
                p.speed_bps = speed;
                on_progress(p);
            }
        }
    }

    {
        let guard = sftp.lock().await;
        let sftp_session = guard
            .as_ref()
            .ok_or_else(|| "SFTP session closed".to_string())?;
        sftp_session
            .write(&request.dest_path, &all_data)
            .await
            .map_err(|e| format!("SFTP write error: {}", e))?;
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

    let remote_data = {
        let guard = sftp.lock().await;
        let sftp_session = guard
            .as_ref()
            .ok_or_else(|| "SFTP session closed".to_string())?;
        sftp_session
            .read(&request.source_path)
            .await
            .map_err(|e| format!("SFTP read error: {}", e))?
    };

    {
        let mut t = transfers.lock().await;
        if let Some(p) = t.get_mut(&request.id) {
            p.total_bytes = remote_data.len() as u64;
        }
    }

    let mut local_file = tokio::fs::File::create(&request.dest_path)
        .await
        .map_err(|e| format!("Failed to create local file: {}", e))?;

    let mut written: u64 = 0;
    let start = Instant::now();

    for chunk in remote_data.chunks(CHUNK_SIZE) {
        {
            let t = transfers.lock().await;
            if let Some(p) = t.get(&request.id)
                && p.status == TransferStatus::Cancelled
            {
                return Ok(());
            }
        }

        local_file
            .write_all(chunk)
            .await
            .map_err(|e| format!("Write error: {}", e))?;
        written += chunk.len() as u64;

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

    Ok(())
}

impl Default for TransferEngine {
    fn default() -> Self {
        Self::new()
    }
}
