// src/metadata/lock.rs
// SPDX-License-Identifier: AGPL-3.0
// Copyright (c) 2025 Canmi

use rfs_utils::{log, LogLevel};
use std::path::{Path, PathBuf};
use tokio::time::{sleep, Duration};

// A file-based lock guard. The lock is released when this struct is dropped.
pub struct FileLock {
    lock_path: PathBuf,
}

impl FileLock {
    // Acquires a lock on a target path by creating a `.lock` file.
    // It will wait indefinitely if the lock is already held.
    pub async fn acquire(target_path: &Path) -> std::io::Result<Self> {
        let lock_path = target_path.with_extension(
            target_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_owned()
                + ".lock",
        );

        let mut attempts = 0;
        while lock_path.exists() {
            if attempts == 0 {
                log(
                    LogLevel::Debug,
                    &format!("Waiting for lock on: {}", lock_path.display()),
                );
            }
            attempts += 1;
            sleep(Duration::from_millis(100)).await;
        }

        // Create the lock file to claim the lock.
        tokio::fs::File::create(&lock_path).await?;
        log(
            LogLevel::Debug,
            &format!("Acquired lock: {}", lock_path.display()),
        );

        Ok(FileLock { lock_path })
    }
}

// The Drop implementation ensures the lock file is removed when the guard
// goes out of scope, releasing the lock.
impl Drop for FileLock {
    fn drop(&mut self) {
        // This is a synchronous operation but is acceptable for this use case.
        if let Err(e) = std::fs::remove_file(&self.lock_path) {
            // We can't do much about an error here, but we should log it.
            eprintln!(
                "Failed to remove lock file {}: {}",
                self.lock_path.display(),
                e
            );
        } else {
            // Using eprintln here as our logger might not be available during a panic.
            eprintln!("Released lock: {}", self.lock_path.display());
        }
    }
}
