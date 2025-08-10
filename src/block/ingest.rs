// src/block/ingest.rs
// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (c) 2025 Canmi

use crate::block::{digest, store};
use crate::common;
use crate::metadata::{
    error::MetadataError, manager, model::{BlockInfo, FileMetadata}, path_utils,
};
use chrono::Utc;
use rfs_utils::{log, LogLevel};
use std::collections::BTreeMap;
use thiserror::Error;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;

const BUFFER_SIZE: usize = 64 * 1024 * 1024; // 64 MB
const CHUNK_SIZE: usize = 128 * 1024; // 128KB

#[derive(Error, Debug)]
pub enum IngestError {
    #[error("Pool with ID {0} not found.")]
    PoolNotFound(u64),
    #[error("I/O error during file processing: {0}")]
    Io(#[from] std::io::Error),
    #[error("Block storage error: {0}")]
    Store(#[from] store::RwError),
    #[error("Metadata error: {0}")]
    Metadata(#[from] MetadataError),
}

// Ingests a file from the OS into the RFS.
// This process is now: 1. Process all blocks. 2. Create all metadata in one go.
pub async fn ingest_file(
    os_file_path: &str,
    rfs_dir_path: &str,
    filename: &str,
    pool_id: u64,
) -> Result<(), IngestError> {
    // 1. Validate paths and get the pool root.
    path_utils::validate_component(filename)?;
    let pool_root_path = common::pool::get_pool_path_by_id(pool_id)
        .ok_or(IngestError::PoolNotFound(pool_id))?;

    // 2. Set up the async block processing pipeline to gather block info.
    let (full_buf_tx, mut full_buf_rx) = mpsc::channel::<(Vec<u8>, usize)>(2);
    let (empty_buf_tx, mut empty_buf_rx) = mpsc::channel::<Vec<u8>>(2);
    empty_buf_tx.send(vec![0; BUFFER_SIZE]).await.unwrap();
    empty_buf_tx.send(vec![0; BUFFER_SIZE]).await.unwrap();

    let os_file_path_owned = os_file_path.to_string();
    let reader_handle = tokio::spawn(async move {
        let mut file = match tokio::fs::File::open(&os_file_path_owned).await {
            Ok(f) => f,
            Err(_) => return,
        };
        while let Some(mut buffer) = empty_buf_rx.recv().await {
            match file.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => if full_buf_tx.send((buffer, n)).await.is_err() { break; },
                Err(_) => break,
            }
        }
    });

    // 3. Process all chunks and build the complete FileMetadata object in memory.
    let mut blocks = BTreeMap::new();
    let mut total_size: u64 = 0;
    let mut chunk_sequence: u64 = 0;

    while let Some((buffer, bytes_in_buffer)) = full_buf_rx.recv().await {
        for chunk_data in buffer[..bytes_in_buffer].chunks(CHUNK_SIZE) {
            if chunk_data.is_empty() { continue; }
            let xxh3_hash = digest::calculate_xxh3_128(chunk_data);
            let collision_index = store::write_block(&pool_root_path, xxh3_hash, chunk_data).await?;
            blocks.insert(chunk_sequence, BlockInfo { xxh3: xxh3_hash, index: collision_index });
            total_size += chunk_data.len() as u64;
            chunk_sequence += 1;
        }
        let _ = empty_buf_tx.send(buffer).await;
    }
    reader_handle.await.unwrap();

    let now = Utc::now();
    let final_file_metadata = FileMetadata {
        filename: filename.to_string(), // Populate the new filename field.
        size: total_size,
        created_at: now,
        modified_at: now,
        blocks,
    };

    // 4. Call the metadata manager to create the file entry atomically.
    manager::create_file(&pool_root_path, rfs_dir_path, filename, final_file_metadata).await?;
    let final_rfs_path = format!("{}/{}", rfs_dir_path.trim_end_matches('/'), filename);
    log(LogLevel::Info, &format!("Successfully ingested '{}' into rfs at '{}'", os_file_path, final_rfs_path));

    Ok(())
}
