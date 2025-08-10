// src/block/slicing.rs
// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (c) 2025 Canmi

use crate::block::{digest, store};
use crate::common;
use rfs_utils::{log, LogLevel};
use std::io;
use thiserror::Error;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;

const BUFFER_SIZE: usize = 64 * 1024 * 1024; // 64 MB
const CHUNK_SIZE: usize = 128 * 1024; // 128KB

#[derive(Error, Debug)]
pub enum SlicingError {
    #[error("Pool with ID {0} not found.")]
    PoolNotFound(u64),
    #[error("I/O error during file processing: {0}")]
    Io(#[from] io::Error),
    #[error("Block write error: {0}")]
    Rw(#[from] store::RwError),
}

// Reads a file using a double buffer, slices it, and stores each chunk.
pub async fn process_file(file_path: &str, pool_id: u64) -> Result<(), SlicingError> {
    let pool_root_path = common::pool::get_pool_path_by_id(pool_id)
        .ok_or(SlicingError::PoolNotFound(pool_id))?;

    // Channel for sending filled buffers from reader to processor
    let (full_buf_tx, mut full_buf_rx) = mpsc::channel::<(Vec<u8>, usize)>(2);
    // Channel for sending empty buffers from processor back to reader
    let (empty_buf_tx, mut empty_buf_rx) = mpsc::channel::<Vec<u8>>(2);

    // Provide two empty buffers to start the cycle.
    empty_buf_tx.send(vec![0; BUFFER_SIZE]).await.unwrap();
    empty_buf_tx.send(vec![0; BUFFER_SIZE]).await.unwrap();

    // --- Reader Task (Producer) ---
    let file_path_owned = file_path.to_string();
    let reader_handle = tokio::spawn(async move {
        let mut file = match tokio::fs::File::open(&file_path_owned).await {
            Ok(f) => f,
            Err(e) => {
                log(LogLevel::Error, &format!("Reader failed to open file '{}': {}", file_path_owned, e));
                return;
            }
        };

        while let Some(mut buffer) = empty_buf_rx.recv().await {
            match file.read(&mut buffer).await {
                Ok(0) => break, // End of file
                Ok(n) => {
                    // Successfully read `n` bytes. Send the buffer (with the number of bytes read) to the processor.
                    if full_buf_tx.send((buffer, n)).await.is_err() {
                        // Processor has shut down, so we can stop reading.
                        break;
                    }
                }
                Err(e) => {
                    log(LogLevel::Error, &format!("File read error in reader task: {}", e));
                    break;
                }
            }
        }
    });

    // --- Processor Task (Consumer) ---
    let mut chunk_index = 0;
    while let Some((buffer, bytes_in_buffer)) = full_buf_rx.recv().await {
        log(LogLevel::Debug, &format!("Processor received buffer with {} bytes.", bytes_in_buffer));

        for chunk_data in buffer[..bytes_in_buffer].chunks(CHUNK_SIZE) {
            if chunk_data.is_empty() { continue; }
            let xxh3_hash = digest::calculate_xxh3_128(chunk_data);

            log(
                LogLevel::Debug,
                &format!(
                    "Processing chunk {}: size={}, xxh3={:032x}",
                    chunk_index,
                    chunk_data.len(),
                    xxh3_hash
                ),
            );

            let _collision_index = store::write_block(&pool_root_path, xxh3_hash, chunk_data).await?;
            chunk_index += 1;
        }

        // Return the buffer to the reader so it can be refilled.
        let _ = empty_buf_tx.send(buffer).await;
    }

    let _ = reader_handle.await;

    log(LogLevel::Info, &format!("Finished processing all chunks for file '{}'.", file_path));
    Ok(())
}
