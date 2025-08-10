// src/block/rw.rs
// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (c) 2025 Canmi

use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Error, Debug)]
pub enum RwError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Path construction error for hash {0}")]
    PathError(String),
}

// Constructs the full path to a block's directory based on its XXH3 hash.
// The new structure is /blocks/{:2}/{:2}/{:2}/
fn get_block_dir(root_path: &str, xxh3: u128) -> PathBuf {
    let xxh3_hex = format!("{:032x}", xxh3);
    Path::new(root_path)
        .join("blocks")
        .join(&xxh3_hex[0..2])
        .join(&xxh3_hex[2..4])
        .join(&xxh3_hex[4..6])
}

// Writes a data block to the storage pool, using only XXH3 for pathing and naming.
//
// # Returns
// The collision index `n` of the `{xxh3}-{n}` file that was written or matched.
pub async fn write_block(
    root_path: &str,
    xxh3: u128,
    data: &[u8],
) -> Result<u32, RwError> {
    let block_dir = get_block_dir(root_path, xxh3);
    fs::create_dir_all(&block_dir).await?;

    // --- Phase 1: Discovery (Optimized) ---
    // Asynchronously read the directory to find all potential collision files at once.
    let mut max_n = 0;
    let mut matching_paths = Vec::new();

    // Use a match to gracefully handle cases where the directory might not be readable.
    match fs::read_dir(&block_dir).await {
        Ok(mut entries) => {
            let prefix = format!("{:032x}-", xxh3);
            while let Some(entry) = entries.next_entry().await? {
                if let Some(name_str) = entry.file_name().to_str() {
                    if name_str.starts_with(&prefix) {
                        if let Some(n_str) = name_str.strip_prefix(&prefix) {
                            if let Ok(n) = n_str.parse::<u32>() {
                                if n > max_n {
                                    max_n = n;
                                }
                                matching_paths.push((n, entry.path()));
                            }
                        }
                    }
                }
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // This case should ideally not be hit due to `create_dir_all`,
            // but we handle it defensively. It means no blocks exist yet.
        }
        Err(e) => return Err(e.into()),
    };

    // --- Phase 2: Comparison ---
    // Now, iterate through the discovered files and compare their content.
    for (n, path) in matching_paths {
        let existing_data = fs::read(path).await?;
        if existing_data == data {
            // Found an exact match. Return its index.
            return Ok(n);
        }
    }

    // --- Phase 3: Write New Block ---
    // If we've gone through all existing files and found no match,
    // we write a new file at the end of the chain.
    let new_index = max_n + 1;
    let new_block_filename = format!("{:032x}-{}", xxh3, new_index);
    let new_block_path = block_dir.join(new_block_filename);

    let mut file = fs::File::create(&new_block_path).await?;
    file.write_all(data).await?;

    Ok(new_index)
}

// Reads a data block from the storage pool.
pub async fn read_block(
    root_path: &str,
    xxh3: u128,
    collision_index: u32,
) -> Result<Vec<u8>, RwError> {
    let block_dir = get_block_dir(root_path, xxh3);
    let block_filename = format!("{:032x}-{}", xxh3, collision_index);
    let block_path = block_dir.join(block_filename);
    let data = fs::read(block_path).await?;
    Ok(data)
}
