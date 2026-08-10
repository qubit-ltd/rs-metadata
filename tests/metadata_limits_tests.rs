// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
//! Tests for metadata-domain and JSON profile limits.

#![cfg(feature = "json")]

use qubit_budget::JsonResource;
use qubit_metadata::{
    MetadataLimits,
    default_json_limits,
};

#[test]
fn test_metadata_limits_default_exposes_domain_profile() {
    let limits = MetadataLimits::default();
    assert_eq!(limits.max_metadata_entries(), 4_096);
    assert_eq!(limits.max_schema_fields(), 4_096);
    assert_eq!(limits.max_key_bytes(), 256);
    assert_eq!(limits.json().max_input_bytes(), Some(1_048_576));
}

#[test]
fn test_metadata_limits_replace_json_profile() {
    let limits = MetadataLimits::default()
        .with_json(default_json_limits().with_max_input_bytes(8));
    assert_eq!(limits.json().max_input_bytes(), Some(8));
    assert_eq!(
        limits.json().input_bytes_limit().unwrap().resource(),
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
