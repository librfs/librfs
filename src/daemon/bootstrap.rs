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

    // shutdown signal handler
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // shutdown
    log(LogLevel::Info, "Server has shut down. Cleaning up resources.");
    if let Err(e) = tokio::fs::remove_file(SOCKET_PATH).await {
        log(
            LogLevel::Error,
            &format!("Failed to remove socket file on shutdown: {}", e),
        );
    } else {
        log(
            LogLevel::Info,
            &format!("Successfully removed socket: {}", SOCKET_PATH),
        );
    }

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            println!();
            log(LogLevel::Info, "Received Ctrl+C signal.");
        },
        _ = terminate => log(LogLevel::Info, "Received terminate signal."),
    }
}