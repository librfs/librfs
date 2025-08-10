// src/metadata/path_utils.rs
// SPDX-License-Identifier: AGPL-3.0
// Copyright (c) 2025 Canmi

use crate::metadata::error::MetadataError;
use once_cell::sync::Lazy;
use rand::Rng;
use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

const CID_LENGTH: usize = 5;
const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

// Regex for validating safe characters in file/directory names.
static SAFE_NAME_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[\p{L}\p{N}_\-\.\@\~\(\)\[\]]+$").unwrap());

// Generates a random, 5-character alphanumeric Content ID (CID).
pub fn generate_cid() -> String {
    // Get a thread-local random number generator.
    let mut rng = rand::rng();
    (0..CID_LENGTH)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

// Validates a single component of a path (a file or directory name).
pub fn validate_component(name: &str) -> Result<(), MetadataError> {
    if name.is_empty() {
        return Err(MetadataError::EmptyPathComponent);
    }
    // Check for allowed character set.
    if !SAFE_NAME_REGEX.is_match(name) {
        return Err(MetadataError::InvalidPathComponent(name.to_string()));
    }
    // Handle special cases for '.'.
    if name == "." || name == ".." || name.ends_with('.') || name.contains("..") {
        return Err(MetadataError::InvalidPathComponent(name.to_string()));
    }
    Ok(())
}

// Splits a virtual path into its components and validates each one.
// Returns a Vec of owned Strings to avoid lifetime issues.
pub fn validate_and_split_path(path: &str) -> Result<Vec<String>, MetadataError> {
    let components: Vec<String> = path
        .trim_matches('/')
        .graphemes(true)
        .collect::<String>()
        .split('/')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

    for component in &components {
        validate_component(component)?;
    }
    Ok(components)
}
