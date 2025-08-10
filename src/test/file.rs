// src/test/file.rs
// SPDX-License-Identifier: AGPL-3.0
// Copyright (c) 2025 Canmi

use crate::block::ingest;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
pub struct TestBlockStorageRequest {
    pub file: String,
    pub path: String, // This is now treated as the destination DIRECTORY
    pub pool: u64,
}

/// Axum handler for testing the storage process.
pub async fn post_test_block_storage_handler(
    Json(payload): Json<TestBlockStorageRequest>,
) -> impl IntoResponse {
    let os_path = Path::new(&payload.file);

    if !os_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            format!("Source file not found: {}", payload.file),
        );
    }

    // --- LOGIC CHANGE ---
    // Extract the filename from the source OS file path.
    let filename = match os_path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                format!("Could not determine filename from: {}", payload.file),
            );
        }
    };

    // The 'path' from the payload is now correctly treated as the destination directory.
    match ingest::ingest_file(&payload.file, &payload.path, filename, payload.pool).await {
        Ok(_) => {
            let final_path = Path::new(&payload.path).join(filename);
            (
                StatusCode::OK,
                format!("Successfully ingested file into RFS at: {}", final_path.display()),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to process file: {}", e),
        ),
    }
}
