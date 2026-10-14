use crate::error::{AppError, Result};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::net::TcpListener;
use std::path::{Path, PathBuf};

/// Single Instance Manager replicating Python SingleInstanceManager (lines 40-230 of attendance_v8.py)
pub struct SingleInstanceManager {
    listener: Option<TcpListener>,
    lock_file_path: Option<PathBuf>,
    lock_file: Option<File>,
    port: u16,
}

impl SingleInstanceManager {
    pub fn new(port: u16) -> Self {
        Self {
            listener: None,
            lock_file_path: None,
            lock_file: None,
            port,
        }
    }

    pub fn with_lock_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.lock_file_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Acquire exclusive lock via both TCP binding and file PID locking
    pub fn acquire_lock(&mut self) -> Result<()> {
        // 1. TCP socket lock (primary cross-platform mechanism)
        match TcpListener::bind(("127.0.0.1", self.port)) {
            Ok(listener) => {
                tracing::info!("Single instance socket lock successfully bound to 127.0.0.1:{}", self.port);
                self.listener = Some(listener);
            }
            Err(e) => {
                return Err(AppError::LockError(format!(
                    "Another instance of Attendance System is already running on port {}: {}",
                    self.port, e
                )));
            }
        }

        // 2. Lock file with process PID (matching Python create_lock_file)
        if let Some(ref path) = self.lock_file_path {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            match OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)
            {
                Ok(mut file) => {
                    let pid = std::process::id();
                    let info = format!("PID: {}\nStarted: {}\n", pid, chrono::Utc::now().to_rfc3339());
                    let _ = file.write_all(info.as_bytes());
                    self.lock_file = Some(file);
                    tracing::info!("Single instance lock file written to {:?}", path);
                }
                Err(e) => {
                    tracing::warn!("Could not create lock file {:?}: {}", path, e);
                }
            }
        }

        Ok(())
    }

    /// Explicit cleanup on shutdown
    pub fn release_lock(&mut self) {
        if let Some(ref path) = self.lock_file_path {
            let _ = std::fs::remove_file(path);
        }
        self.listener = None;
        self.lock_file = None;
        tracing::info!("Single instance lock released.");
    }
}

impl Drop for SingleInstanceManager {
    fn drop(&mut self) {
        self.release_lock();
    }
}
