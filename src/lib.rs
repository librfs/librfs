// src/lib.rs
// SPDX-License-Identifier: AGPL-3.0
// Copyright (c) 2025 Canmi

pub mod metadata;

pub use metadata::error::MetadataError;
pub use metadata::manager::{create_file, list_directory};
pub use metadata::model;
