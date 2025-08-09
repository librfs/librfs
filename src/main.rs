// src/main.rs
// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 Canmi

mod daemon;

use rfs_utils::{log, LogLevel};

#[tokio::main]
async fn main() {
    log(LogLevel::Info, "Starting rfsd daemon...");
    if let Err(e) = daemon::bootstrap::start().await {
        log(LogLevel::Error, &format!("Daemon failed to start: {}", e));
        std::process::exit(1);
    }
}