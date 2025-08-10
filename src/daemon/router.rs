// src/daemon/router.rs
// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (c) 2025 Canmi

use crate::test::file::post_test_block_storage_handler;
use axum::{
    routing::{get, post},
    Router,
};

pub fn create_router() -> Router {
    Router::new()
        .route("/", get(get_root_handler))
        .route("/test/file/block/storage", post(post_test_block_storage_handler))
}

async fn get_root_handler() -> &'static str {
    "PONG"
}