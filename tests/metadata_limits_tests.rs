// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for metadata-domain and JSON profile limits.

#![cfg(feature = "json")]

use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonResource;
use qubit_metadata::MetadataLimits;

#[test]
fn test_metadata_limits_default_exposes_domain_profile() {
    let limits = MetadataLimits::default();
    let decode = limits.json_decode();
    let encode = limits.json_encode();
    assert_eq!(limits.max_metadata_entries(), 4_096);
    assert_eq!(limits.max_schema_fields(), 4_096);
    assert_eq!(limits.max_key_bytes(), 256);
    assert_eq!(decode.max_input_bytes(), Some(1_048_576));
    assert_eq!(encode.max_output_bytes(), Some(1_048_576));
    assert_eq!(decode.value_limits().max_depth(), Some(64));
    assert_eq!(decode.value_limits().max_nodes(), Some(100_000));
    assert_eq!(decode.value_limits().max_sequence_items(), Some(4_096));
    assert_eq!(decode.value_limits().max_map_entries(), Some(4_096));
    assert_eq!(decode.value_limits().max_key_bytes(), Some(256 * 1024));
    assert_eq!(decode.value_limits().max_string_bytes(), Some(256 * 1024));
    assert_eq!(decode.value_limits().max_number_bytes(), Some(4_096));
    assert_eq!(decode.value_limits().max_payload_bytes(), Some(1_048_576));
    assert_eq!(encode.value_limits(), decode.value_limits());
}

#[test]
fn test_metadata_limits_replace_json_profile() {
    let limits = MetadataLimits::builder()
        .json_decode(
            JsonDecodeLimits::builder()
                .input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, 8))
                .build(),
        )
        .build()
        .expect("profile should build");
    assert_eq!(limits.json_decode().max_input_bytes(), Some(8));
    assert_eq!(
        limits.json_decode().input_bytes_limit().unwrap().resource(),
        &JsonResource::InputBytes
    );

    let limits = MetadataLimits::builder()
        .json_encode(
            JsonEncodeLimits::builder()
                .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 8))
                .build(),
        )
        .max_metadata_entries(4_096)
        .max_schema_fields(4_096)
        .max_key_bytes(256)
        .build()
        .expect("profile should build");
    assert_eq!(limits.json_encode().max_output_bytes(), Some(8));
    assert_eq!(limits.max_metadata_entries(), 4_096);
    assert_eq!(limits.max_schema_fields(), 4_096);
    assert_eq!(limits.max_key_bytes(), 256);
}

#[test]
fn test_metadata_limits_reject_domain_values_above_hard_maxima() {
    assert!(MetadataLimits::builder().max_metadata_entries(4_097).build().is_err());
    assert!(MetadataLimits::builder().max_schema_fields(4_097).build().is_err());
    assert!(MetadataLimits::builder().max_key_bytes(257).build().is_err());
}

#[test]
fn test_metadata_limits_builder_applies_every_profile_and_domain_override() {
    let decode = JsonDecodeLimits::builder()
        .input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, 9))
        .build();
    let encode = JsonEncodeLimits::builder()
        .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 11))
        .build();
    let limits = MetadataLimits::builder()
        .json_decode(decode)
        .json_encode(encode)
        .max_metadata_entries(12)
        .max_schema_fields(13)
        .max_key_bytes(14)
        .build()
        .expect("profile should build");

    assert_eq!(limits.json_decode().max_input_bytes(), Some(9));
    assert_eq!(limits.json_encode().max_output_bytes(), Some(11));
    assert_eq!(limits.max_metadata_entries(), 12);
    assert_eq!(limits.max_schema_fields(), 13);
    assert_eq!(limits.max_key_bytes(), 14);
}

#[test]
fn test_metadata_limit_validation_accepts_hard_boundaries() {
    let limits = MetadataLimits::builder()
        .max_metadata_entries(4_096)
        .max_schema_fields(4_096)
        .max_key_bytes(256)
        .build()
        .expect("hard boundaries must be accepted");
    assert_eq!(limits.max_key_bytes(), 256);
}

#[test]
fn test_metadata_limit_validation_rejects_each_value_above_boundary() {
    for (name, build) in [
        (
            "metadata entries",
            MetadataLimits::builder().max_metadata_entries(4_097).build(),
        ),
        (
            "schema fields",
            MetadataLimits::builder().max_schema_fields(4_097).build(),
        ),
        ("key bytes", MetadataLimits::builder().max_key_bytes(257).build()),
    ] {
        assert!(build.is_err(), "{name} above its cap must fail");
    }
}

#[test]
fn test_validate_rejects_each_invalid_domain_limit_without_build() {
    assert!(
        MetadataLimits::debug_only_invalid_domain_limits(4_097, 4_096, 256)
            .validate()
            .is_err()
    );
    assert!(
        MetadataLimits::debug_only_invalid_domain_limits(4_096, 4_097, 256)
            .validate()
            .is_err()
    );
    assert!(
        MetadataLimits::debug_only_invalid_domain_limits(4_096, 4_096, 257)
            .validate()
            .is_err()
    );
}

#[test]
fn test_metadata_limits_build_rejects_invalid_domain_configuration() {
    let error = MetadataLimits::builder()
        .max_metadata_entries(4_097)
        .build()
        .expect_err("invalid domain limits must fail during configuration");

    assert!(error.to_string().contains("metadata entries limit"));
}
