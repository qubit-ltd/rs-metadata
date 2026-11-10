// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Fuzz-target access to the small decoding profiles.

#[allow(unexpected_cfgs)]
#[path = "../../tests/support/decode_profiles.rs"]
mod shared;

use qubit_metadata::FilterLimits;
pub use shared::small_metadata_limits;

/// Builds the deliberately small filter expression profile used by fuzzing.
#[allow(dead_code)]
pub fn small_filter_limits() -> FilterLimits {
    FilterLimits::builder()
        .max_depth(4)
        .max_nodes(16)
        .max_set_values(4)
        .max_key_bytes(16)
        .build()
        .expect("fixed small filter profile is valid")
}
