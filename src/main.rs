// src/main.rs
// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025 Canmi

mod block;
mod common;
mod daemon;
mod test;

use rfs_ess::load_config;
use rfs_pool::load_and_mount_pools;
use rfs_utils::{log, set_log_level, LogLevel};

const CONFIG_PATH: &str = "/opt/rfs/rfsd/config.toml";
const POOL_CONFIG_PATH: &str = "/opt/rfs/rfsd/pool.toml";

#[tokio::main]
async fn main() {
    // 1. Load main config into a local variable.
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

    // 2. Set log level from the config object.
    set_log_level(config.common.log_level);
    log(LogLevel::Info, "Configuration loaded, logger initialized.");
    log(LogLevel::Debug, "Debug mode logger initialized.");

    // 3. Load and validate storage pools.
    if let Err(e) = load_and_mount_pools(POOL_CONFIG_PATH).await {
        log(LogLevel::Error, &e.to_string());
        if matches!(e, rfs_pool::PoolError::MustConfigure(_)) {
            std::process::exit(0);
        } else {
            std::process::exit(1);
        }
    }

    // 4. Start the daemon, passing the config as a parameter.
    if let Err(e) = daemon::bootstrap::run(&config).await {
        log(LogLevel::Error, &format!("Daemon process failed: {}", e));
        std::process::exit(1);
    }
}
