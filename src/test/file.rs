// src/test/file.rs
// SPDX-License-Identifier: AGPL-3.0
// Copyright (c) 2025 Canmi

use crate::block::ingest;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
pub struct TestBlockStorageRequest {
    pub path: String,
    pub pool: u64,
}

// Axum handler for testing the storage process.
pub async fn post_test_block_storage_handler(
    Json(payload): Json<TestBlockStorageRequest>,
) -> impl IntoResponse {
    if !Path::new(&payload.path).exists() {
        return (
            StatusCode::NOT_FOUND,
            format!("File not found: {}", payload.path),
        );
    }

    match ingest::process_file(&payload.path, payload.pool).await {
        Ok(_) => (
            StatusCode::OK,
            format!("Successfully processed file: {}", payload.path),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to process file: {}", e),
        ),
    }
}
