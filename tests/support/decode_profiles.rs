// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Small JSON decoding profiles shared by packaged regressions and fuzz
//! targets.

use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueLimits;
#[cfg(feature = "filter")]
use qubit_metadata::FilterLimits;
use qubit_metadata::MetadataLimits;

/// Builds the deliberately small metadata and generic JSON profile used by
/// fuzzing.
pub fn small_metadata_limits() -> MetadataLimits {
    let value = JsonValueLimits::builder()
        .max_depth(16)
        .max_nodes(64)
        .max_sequence_items(8)
        .max_map_entries(8)
        .max_key_bytes(32)
        .string_bytes_limit(ResourceLimit::new(JsonResource::StringBytes, 64))
        .number_bytes_limit(ResourceLimit::new(JsonResource::NumberBytes, 16))
        .payload_bytes_limit(ResourceLimit::new(JsonResource::PayloadBytes, 512))
        .build();
    let decode = JsonDecodeLimits::builder()
        .input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, 1024))
        .value_limits(value)
        .build();
    MetadataLimits::builder()
        .json_decode(decode)
        .max_metadata_entries(4)
        .max_schema_fields(4)
        .max_key_bytes(16)
        .build()
        .expect("fixed small metadata profile is valid")
}

/// Builds the deliberately small filter expression profile used by fuzzing.
#[cfg(feature = "filter")]
pub fn small_filter_limits() -> FilterLimits {
    FilterLimits::builder()
        .max_depth(4)
        .max_nodes(16)
        .max_set_values(4)
        .max_key_bytes(16)
        .build()
        .expect("fixed small filter profile is valid")
}
