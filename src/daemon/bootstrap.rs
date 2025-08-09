// src/daemon/bootstrap.rs
// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (c) 2025 Canmi

use crate::daemon::{router, unixsock};
use rfs_utils::{log, LogLevel};

const SOCKET_PATH: &str = "/opt/rfs/rfsd/rfsd.sock";

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let listener = unixsock::bind(SOCKET_PATH).await?;
    let app = router::create_router();
    log(LogLevel::Info, &format!("Server listening on {}", SOCKET_PATH));
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}