// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for metadata-domain and JSON profile limits.

#![cfg(feature = "json")]

use qubit_budget::{
    ResourceLimit,
};
use qubit_json::JsonResource;
use qubit_metadata::{
    MetadataLimits,
    default_json_decode_limits,
};

#[test]
fn test_metadata_limits_default_exposes_domain_profile() {
    let limits = MetadataLimits::default();
    assert_eq!(limits.max_metadata_entries(), 4_096);
    assert_eq!(limits.max_schema_fields(), 4_096);
    assert_eq!(limits.max_key_bytes(), 256);
    assert_eq!(limits.json_decode().max_input_bytes(), Some(1_048_576));
    assert_eq!(limits.json_encode().max_output_bytes(), Some(1_048_576));
}

#[test]
fn test_metadata_limits_replace_json_profile() {
    let limits = MetadataLimits::default().with_json_decode(
        default_json_decode_limits().with_input_bytes_limit(
            ResourceLimit::new(JsonResource::InputBytes, 8),
        ),
    );
    assert_eq!(limits.json_decode().max_input_bytes(), Some(8));
    assert_eq!(
        limits.json_decode().input_bytes_limit().unwrap().resource(),
        &JsonResource::InputBytes
    );
}

#[test]
fn test_metadata_limits_reject_domain_values_above_hard_maxima() {
    assert!(
        MetadataLimits::default()
            .with_max_metadata_entries(4_097)
            .validate()
            .is_err()
    );
    assert!(
        MetadataLimits::default()
            .with_max_schema_fields(4_097)
            .validate()
            .is_err()
    );
    assert!(
        MetadataLimits::default()
            .with_max_key_bytes(257)
            .validate()
            .is_err()
    );
}
