// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
#![cfg(feature = "json")]

use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonResource;
use qubit_metadata::MetadataLimits;
use qubit_metadata::default_json_decode_limits;
use qubit_metadata::default_json_encode_limits;

#[test]
fn test_readme_directional_limits_example() {
    let decode = default_json_decode_limits()
        .into_builder()
        .input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, 64 * 1024))
        .build();
    let encode = default_json_encode_limits()
        .into_builder()
        .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 128 * 1024))
        .build();
    let limits = MetadataLimits::builder()
        .json_decode(decode)
        .json_encode(encode)
        .build()
        .expect("limits should build");
    assert_eq!(limits.json_decode().max_input_bytes(), Some(64 * 1024));
    assert_eq!(limits.json_encode().max_output_bytes(), Some(128 * 1024));
}

#[test]
fn test_user_guide_domain_limits_example() {
    let limits = MetadataLimits::builder()
        .max_metadata_entries(128)
        .max_key_bytes(128)
        .build()
        .expect("limits should build");
    assert_eq!(limits.max_metadata_entries(), 128);
    assert_eq!(limits.max_key_bytes(), 128);
}
