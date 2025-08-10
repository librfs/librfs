// src/common/pool.rs
// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025 Canmi

use rfs_pool::POOLS;

/// Gets the storage path for a given pool ID.
///
/// This function locks the global POOLS vector and searches for a pool
/// with the matching ID.
///
/// # Arguments
/// * `id` - The ID of the pool to find.
///
/// # Returns
/// An `Option<String>` containing the path if the pool is found, otherwise `None`.
pub fn get_pool_path_by_id(id: u64) -> Option<String> {
    let pools_guard = POOLS.lock().unwrap();
    pools_guard
        .iter()
        .find(|p| p.pool_id == id)
        .map(|p| p.path.clone())
}
