// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the strict MetadataSchema v1 envelope.

#[cfg(feature = "json")]
use qubit_budget::ResourceLimit;
#[cfg(feature = "json")]
use qubit_budget::json::JsonResource;
use qubit_datatype::DataType;
use qubit_metadata::MetadataSchema;
#[cfg(feature = "json")]
use qubit_metadata::{
    MetadataLimits,
    default_json_decode_limits,
};

#[test]
fn test_metadata_schema_rejects_unknown_field_and_unsupported_version() {
    let schema = MetadataSchema::default();
    let mut unknown =
        serde_json::to_value(&schema).expect("schema should serialize");
    unknown["extra"] = serde_json::json!(true);
    let mut version =
        serde_json::to_value(&schema).expect("schema should serialize");
    version["version"] = serde_json::json!(2);

    assert!(serde_json::from_value::<MetadataSchema>(unknown).is_err());
    assert!(serde_json::from_value::<MetadataSchema>(version).is_err());
}

#[test]
fn test_metadata_schema_v1_round_trip() {
    let schema = MetadataSchema::default();
    let encoded =
        serde_json::to_string(&schema).expect("schema should serialize");
    let decoded =
        serde_json::from_str(&encoded).expect("schema should deserialize");

    assert_eq!(schema, decoded);
}

#[test]
fn test_metadata_schema_serialization_enforces_hard_key_limit() {
    let long_key = "k".repeat(257);
    let schema = MetadataSchema::builder()
        .optional(&long_key, DataType::String)
        .build()
        .expect("schema should build");

    assert!(serde_json::to_vec(&schema).is_err());
}

#[test]
fn test_metadata_schema_serialization_enforces_hard_entry_limit() {
    let mut builder = MetadataSchema::builder();
    for index in 0..=4_096 {
        builder = builder.optional(&format!("key-{index}"), DataType::String);
    }
    let schema = builder.build().expect("schema should build");

    assert!(serde_json::to_vec(&schema).is_err());
}

#[test]
#[cfg(feature = "json")]
fn test_metadata_schema_json_decoder_rejects_unsupported_version() {
    let input = serde_json::json!({
        "version": 2,
        "fields": {},
        "unknown_metadata_field_policy": "reject",
        "unknown_filter_field_policy": "reject",
    });
    let input = serde_json::to_vec(&input).expect("input should serialize");

    assert!(MetadataSchema::decode_json_slice(&input).is_err());
}

#[test]
#[cfg(feature = "json")]
fn test_metadata_schema_json_decoder_exercises_seed_error_paths() {
    let duplicate_policy = br#"{
        "version": 1,
        "fields": {},
        "unknown_metadata_field_policy": "reject",
        "unknown_metadata_field_policy": "allow",
        "unknown_filter_field_policy": "reject"
    }"#;
    assert!(MetadataSchema::decode_json_slice(duplicate_policy).is_err());

    let duplicate_filter_policy = br#"{
        "version": 1,
        "fields": {},
        "unknown_metadata_field_policy": "reject",
        "unknown_filter_field_policy": "reject",
        "unknown_filter_field_policy": "allow_unchecked"
    }"#;
    assert!(
        MetadataSchema::decode_json_slice(duplicate_filter_policy).is_err()
    );

    let wrong_fields_type = br#"{
        "version": 1,
        "fields": [],
        "unknown_metadata_field_policy": "reject",
        "unknown_filter_field_policy": "reject"
    }"#;
    assert!(MetadataSchema::decode_json_slice(wrong_fields_type).is_err());

    let invalid_field = br#"{
        "version": 1,
        "fields": {"name": {"invalid": true}},
        "unknown_metadata_field_policy": "reject",
        "unknown_filter_field_policy": "reject"
    }"#;
    assert!(MetadataSchema::decode_json_slice(invalid_field).is_err());
}

#[test]
#[cfg(feature = "json")]
fn test_metadata_schema_json_decoder_rejects_missing_and_unknown_fields() {
    let missing_version = br#"{
        "fields": {},
        "unknown_metadata_field_policy": "reject",
        "unknown_filter_field_policy": "reject"
    }"#;
    let missing_fields = br#"{
        "version": 1,
        "unknown_metadata_field_policy": "reject",
        "unknown_filter_field_policy": "reject"
    }"#;
    let missing_metadata_policy = br#"{
        "version": 1,
        "fields": {},
        "unknown_filter_field_policy": "reject"
    }"#;
    let missing_filter_policy = br#"{
        "version": 1,
        "fields": {},
        "unknown_metadata_field_policy": "reject"
    }"#;
    let unknown_field = br#"{
        "version": 1,
        "fields": {},
        "unknown_metadata_field_policy": "reject",
        "unknown_filter_field_policy": "reject",
        "extra": true
    }"#;

    for input in [
        missing_version.as_slice(),
        missing_fields.as_slice(),
        missing_metadata_policy.as_slice(),
        missing_filter_policy.as_slice(),
        unknown_field.as_slice(),
    ] {
        assert!(MetadataSchema::decode_json_slice(input).is_err());
    }
}

#[test]
#[cfg(feature = "json")]
fn test_metadata_schema_json_decoder_maps_syntax_and_budget_errors() {
    assert!(MetadataSchema::decode_json_slice(b"!").is_err());

    let limits = MetadataLimits::default().with_json_decode(
        default_json_decode_limits().with_input_bytes_limit(
            ResourceLimit::new(JsonResource::InputBytes, 1),
        ),
    );
    assert!(
        MetadataSchema::decode_json_slice_with_limits(b"{}", limits,).is_err()
    );
}
