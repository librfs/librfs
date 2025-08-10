// src/block/mapping.rs
// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (c) 2025 Canmi

use xxhash_rust::xxh3::xxh3_128;

// Calculates the 128-bit XXH3 hash of a byte slice.
pub fn calculate_xxh3_128(data: &[u8]) -> u128 {
    xxh3_128(data)
}
