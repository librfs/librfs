// src/metadata/model.rs
// SPDX-License-Identifier: AGPL-3.0
// Copyright (c) 2025 Canmi

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlockInfo {
    pub xxh3: u128,
    pub index: u32,
}

// Represents the full metadata for a single file, stored in its {cid}.json file.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileMetadata {
    pub filename: String, // The original filename is now stored here.
    pub size: u64,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    // BTreeMap ensures blocks are ordered by their sequence number.
    pub blocks: BTreeMap<u64, BlockInfo>,
}

// Represents a file's entry within a directory's metadata.json.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub cid: String,
    pub size: u64,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

// Represents a directory's entry within its parent's metadata.json.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryInfo {
    pub cid: String,
    pub size: u64,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

// An enum representing either a File or a Directory in a listing.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Entry {
    File(FileEntry),
    Directory(DirectoryInfo),
}

// Represents the content of a metadata.json file, mapping names to entries.
pub type DirectoryListing = HashMap<String, Entry>;
