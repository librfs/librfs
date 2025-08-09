// src/main.rs
// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 Canmi

use rfs_utils::{log, LogLevel};

fn main() {
    log(LogLevel::Info, "This is info message");
    log(LogLevel::Debug, "This is debug message");
    log(LogLevel::Warn, "This is warn message");
    log(LogLevel::Error, "This is error message");
}
