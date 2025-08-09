// src/daemon/unixsock.rs
// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (c) 2025 Canmi

use rfs_utils::{log, LogLevel};
use std::path::Path;
use tokio::net::UnixListener;

pub async fn bind(path_str: &str) -> std::io::Result<UnixListener> {
    let path = Path::new(path_str);

    if tokio::fs::try_exists(path).await? {
        log(
            LogLevel::Warn,
            &format!("Socket path {} already exists. Removing.", path.display()),
        );
        tokio::fs::remove_file(path).await?;
    }

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            log(
                LogLevel::Info,
                &format!("Creating parent directory: {}", parent.display()),
            );
            tokio::fs::create_dir_all(parent).await?;
        }
    }

    log(LogLevel::Info, &format!("Binding to unix socket {}", path_str));
    let listener = UnixListener::bind(path)?;
    log(LogLevel::Info, "Successfully bound to unix socket.");

    Ok(listener)
}