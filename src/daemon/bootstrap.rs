// src/daemon/bootstrap.rs
// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (c) 2025 Canmi

use crate::daemon::{router, unixsock};
use rfs_ess::Config; // Import the Config struct
use rfs_utils::{log, LogLevel};

// run now takes the configuration as a parameter
pub async fn run(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    log(LogLevel::Info, "Starting rfsd daemon...");

    // Read socket path directly from the passed-in config
    let socket_path = config.rfsd.unix_socket.clone();

    log(LogLevel::Debug, "Calling unixsock::bind...");
    let listener = unixsock::bind(&socket_path).await?;
    log(LogLevel::Debug, "unixsock::bind call returned successfully.");

    let app = router::create_router();
    log(LogLevel::Debug, "Router created.");

    log(LogLevel::Info, &format!("Server listening on {}", socket_path));

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    log(LogLevel::Info, "Server has shut down. Cleaning up resources.");
    if let Err(e) = tokio::fs::remove_file(&socket_path).await {
        log(
            LogLevel::Error,
            &format!("Failed to remove socket file on shutdown: {}", e),
        );
    } else {
        log(
            LogLevel::Info,
            &format!("Successfully removed socket: {}", socket_path),
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

    log(LogLevel::Info, "Initiating graceful shutdown.");
}