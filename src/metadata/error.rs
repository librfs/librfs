// src/metadata/error.rs
// SPDX-License-Identifier: AGPL-3.0
// Copyright (c) 2025 Canmi

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MetadataError {
    // An invalid character or format was found in a path component.
    #[error("Invalid character or format in path component: '{0}'")]
    InvalidPathComponent(String),

    // A path component (e.g., between slashes) was empty.
    #[error("Path component cannot be empty")]
    EmptyPathComponent,

    // An entry with the given name already exists at the target path.
    #[error("An entry with the name '{0}' already exists at this path.")]
    EntryAlreadyExists(String),

    // Wraps standard I/O errors.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    // Wraps JSON processing errors.
    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    // The operation expected a directory but found a file.
    #[error("The specified path is a file, not a directory: {0}")]
    NotADirectory(String),
}
