use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};

pub struct LocalTerminalSession {
    writer: Option<Box<dyn Write + Send>>,
    _reader: Option<Box<dyn Read + Send>>,
    reader_handle: Option<std::thread::JoinHandle<()>>,
    cancel: Arc<AtomicBool>,
    #[allow(dead_code)]
    master: Box<dyn MasterPty + Send>,
}

impl LocalTerminalSession {
    pub fn new(
        cols: u16,
        rows: u16,
        on_data: Box<dyn Fn(Vec<u8>) + Send + Sync>,
    ) -> Result<Self> {
        let pty_system = native_pty_system();

        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let cmd = CommandBuilder::new(shell);
        let _child = pair.slave.spawn_command(cmd)?;

        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let cancel = Arc::new(AtomicBool::new(false));
        let cancel_clone = cancel.clone();

        let reader_handle = std::thread::spawn(move || {
            let mut buf = vec![0u8; 8192];
            let mut reader = reader;
            loop {
                if cancel_clone.load(Ordering::Relaxed) {
                    break;
                }
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => on_data(buf[..n].to_vec()),
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            writer: Some(writer),
            _reader: None,
            reader_handle: Some(reader_handle),
            cancel,
            master: pair.master,
        })
    }

    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        if let Some(ref mut w) = self.writer {
            w.write_all(data)?;
            w.flush()?;
        }
        Ok(())
    }

    pub fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        Ok(())
    }
}

impl Drop for LocalTerminalSession {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
        self.writer.take();
        self._reader.take();
        if let Some(handle) = self.reader_handle.take() {
            let _ = handle.join();
        }
    }
}

pub struct LocalTerminalManager {
    sessions: HashMap<String, LocalTerminalSession>,
}

impl LocalTerminalManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn open(
        &mut self,
        session_id: &str,
        cols: u16,
        rows: u16,
        on_data: Box<dyn Fn(Vec<u8>) + Send + Sync>,
    ) -> Result<()> {
        let session = LocalTerminalSession::new(cols, rows, on_data)?;
        self.sessions.insert(session_id.to_string(), session);
        Ok(())
    }

    pub fn write(&mut self, session_id: &str, data: &[u8]) -> Result<()> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Local terminal session not found"))?;
        session.write(data)
    }

    pub fn resize(&self, session_id: &str, cols: u16, rows: u16) -> Result<()> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or_else(|| anyhow::anyhow!("Local terminal session not found"))?;
        session.resize(cols, rows)
    }

    pub fn close(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
    }

    pub fn close_all(&mut self) {
        self.sessions.clear();
    }
}

impl Default for LocalTerminalManager {
    fn default() -> Self {
        Self::new()
    }
}
