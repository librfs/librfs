// src/metadata/manager.rs
// SPDX-License-Identifier: AGPL-3.0
// Copyright (c) 2025 Canmi

use crate::metadata::error::MetadataError;
use crate::metadata::lock::FileLock;
use crate::metadata::model::{
    DirectoryInfo, DirectoryListing, Entry, FileEntry, FileMetadata,
};
use crate::metadata::path_utils;
use chrono::Utc;
use futures::future::BoxFuture;
use std::path::{Path, PathBuf};
use tokio::fs;

const METADATA_DIR: &str = "metadata";
const LISTING_FILE: &str = "metadata.json";

// New function to read the contents of a directory.
pub async fn list_directory(
    pool_root: &str,
    rfs_dir_path: &str,
) -> Result<DirectoryListing, MetadataError> {
    let dir_components = path_utils::validate_and_split_path(rfs_dir_path)?;
    let target_dir_path = resolve_dir_path(pool_root, &dir_components).await?;
    let listing = read_listing(&target_dir_path).await?;
    Ok(listing)
}


// Creates a file with its associated metadata and triggers a recursive update.
pub async fn create_file(
    pool_root: &str,
    rfs_dir_path: &str,
    filename: &str,
    file_metadata: FileMetadata,
) -> Result<(), MetadataError> {
    // Resolve the target directory's physical path.
    let mut dir_components = path_utils::validate_and_split_path(rfs_dir_path)?;
    let target_dir_path = resolve_dir_path(pool_root, &dir_components).await?;
    let listing_path = target_dir_path.join(LISTING_FILE);

    // Acquire a lock on the directory's metadata file.
    let _lock = FileLock::acquire(&listing_path).await?;
    let mut listing = read_listing(&target_dir_path).await?;

    if listing.contains_key(filename) {
        return Err(MetadataError::EntryAlreadyExists(filename.to_string()));
    }

    // Create the new file's metadata and entry.
    let new_cid = path_utils::generate_cid();
    write_file_block_map(&target_dir_path, &new_cid, &file_metadata).await?;

    let new_entry = Entry::File(FileEntry {
        cid: new_cid,
        size: file_metadata.size,
        created_at: file_metadata.created_at,
        modified_at: file_metadata.modified_at,
    });
    listing.insert(filename.to_string(), new_entry);
    write_listing(&target_dir_path, &listing).await?;

    // Propagate the size and timestamp changes up the directory tree.
    propagate_update(pool_root, &mut dir_components, file_metadata.size as i64).await?;

    Ok(())
}

// Recursively updates the size and modification time of parent directories.
fn propagate_update<'a>(
    pool_root: &'a str,
    dir_components: &'a mut [String],
    size_delta: i64,
) -> BoxFuture<'a, Result<(), MetadataError>> {
    Box::pin(async move {
        if dir_components.is_empty() {
            return Ok(()); // Reached the root of the pool.
        }

        if let Some((child_name, parent_components)) = dir_components.split_last_mut() {
            let parent_path = resolve_dir_path(pool_root, parent_components).await?;
            let parent_listing_path = parent_path.join(LISTING_FILE);

            let _lock = FileLock::acquire(&parent_listing_path).await?;
            let mut parent_listing = read_listing(&parent_path).await?;

            // Find the entry for the child directory and update it.
            if let Some(Entry::Directory(dir_info)) = parent_listing.get_mut(child_name) {
                dir_info.size = (dir_info.size as i64 + size_delta) as u64;
                dir_info.modified_at = Utc::now();
            }

            write_listing(&parent_path, &parent_listing).await?;

            // Recurse to the next level up.
            propagate_update(pool_root, parent_components, size_delta).await?;
        }

        Ok(())
    })
}

// Resolves a virtual path to its physical metadata directory.
async fn resolve_dir_path(
    pool_root: &str,
    rfs_dir_components: &[String],
) -> Result<PathBuf, MetadataError> {
    let mut current_path = Path::new(pool_root).join(METADATA_DIR);
    fs::create_dir_all(&current_path).await?;

    for component in rfs_dir_components {
        let listing_path = current_path.join(LISTING_FILE);
        let _lock = FileLock::acquire(&listing_path).await?;
        let mut listing = read_listing(&current_path).await?;

        let entry_info = match listing.get(component) {
            Some(Entry::Directory(info)) => info.clone(),
            Some(Entry::File(_)) => return Err(MetadataError::NotADirectory(component.clone())),
            None => {
                let now = Utc::now();
                let new_dir_info = DirectoryInfo {
                    cid: path_utils::generate_cid(),
                    size: 0,
                    created_at: now,
                    modified_at: now,
                };
                listing.insert(component.clone(), Entry::Directory(new_dir_info.clone()));
                write_listing(&current_path, &listing).await?;
                new_dir_info
            }
        };
        current_path.push(entry_info.cid);
        fs::create_dir_all(&current_path).await?;
    }
    Ok(current_path)
}

// Reads and parses a metadata.json file.
async fn read_listing(dir_path: &Path) -> Result<DirectoryListing, MetadataError> {
    let listing_path = dir_path.join(LISTING_FILE);
    if !listing_path.exists() {
        return Ok(DirectoryListing::new());
    }
    let content = fs::read(listing_path).await?;
    Ok(serde_json::from_slice(&content)?)
}

// Writes a DirectoryListing to a metadata.json file.
async fn write_listing(
    dir_path: &Path,
    listing: &DirectoryListing,
) -> Result<(), MetadataError> {
    let content = serde_json::to_vec_pretty(listing)?;
    fs::write(dir_path.join(LISTING_FILE), content).await?;
    Ok(())
}

// Writes the detailed FileMetadata (block map) to its {cid}.json file.
async fn write_file_block_map(
    dir_path: &Path,
    cid: &str,
    metadata: &FileMetadata,
) -> Result<(), MetadataError> {
    let content = serde_json::to_vec_pretty(metadata)?;
    let meta_path = dir_path.join(format!("{}.json", cid));
fs::write(meta_path, &content).await?;
    Ok(())
}
