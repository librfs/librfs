// src/main.rs
// SPDX-License-Identifier: AGPL-3.0
// Copyright (c) 2025 Canmi

mod daemon;

use rfs_ess::load_config;
use rfs_pool::{load_and_mount_pools, PoolError};
use rfs_utils::{log, set_log_level, LogLevel};

const CONFIG_PATH: &str = "/opt/rfs/rfsd/config.toml";
const POOL_CONFIG_PATH: &str = "/opt/rfs/rfsd/pool.toml";

#[tokio::main]
async fn main() {
    // Load main config into a local variable.
    let config = match load_config(CONFIG_PATH) {
        Ok(cfg) => cfg,
        Err(e) => {
            log(
                LogLevel::Error,
                &format!("Failed to load configuration: {}. Exiting.", e),
            );
            std::process::exit(1);
        }
    };

    // Set log level from the config object.
    set_log_level(config.common.log_level);
    log(LogLevel::Info, "Configuration loaded, logger initialized.");
    log(LogLevel::Debug, "Debug mode logger initialized.");

    // Load and validate storage pools.
    if let Err(e) = load_and_mount_pools(POOL_CONFIG_PATH).await {
        // Log the specific error message, which is now more informative.
        log(LogLevel::Error, &e.to_string());
        // Handle the specific case where the user needs to configure the new file.
        if matches!(e, PoolError::MustConfigure(_)) {
            // This is not a crash, but an intentional stop. Exit with 0.
            std::process::exit(0);
        } else {
            // For all other pool errors, exit with a failure code.
            std::process::exit(1);
        }
    }

    // Start the daemon, passing the config as a parameter.
    if let Err(e) = daemon::bootstrap::run(&config).await {
        log(LogLevel::Error, &format!("Daemon process failed: {}", e));
        std::process::exit(1);
    }
}