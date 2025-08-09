// src/daemon/router.rs
// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (c) 2025 Canmi

use axum::{routing::get, Router};

pub fn create_router() -> Router {
    Router::new().route("/", get(get_root_handler))
}

async fn get_root_handler() -> &'static str {
    "PONG"
}