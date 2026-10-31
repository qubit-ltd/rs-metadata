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
use qubit_budget::json::JsonDecodeLimits;
#[cfg(feature = "json")]
use qubit_budget::json::JsonResource;
use qubit_datatype::DataType;
#[cfg(feature = "json")]
use qubit_metadata::MetadataLimits;
use qubit_metadata::MetadataSchema;
use serde_json::from_str;
use serde_json::from_value;
use serde_json::json;
use serde_json::to_string;
use serde_json::to_value;
use serde_json::to_vec;

#[test]
fn test_metadata_schema_rejects_unknown_field_and_unsupported_version() {
    let schema = MetadataSchema::default();
    let mut unknown = to_value(&schema).expect("schema should serialize");
    unknown["extra"] = json!(true);
    let mut version = to_value(&schema).expect("schema should serialize");
    version["version"] = json!(2);

    assert!(
        from_value::<MetadataSchema>(unknown).is_err(),
        "unknown schema fields should be rejected"
    );
    assert!(
        from_value::<MetadataSchema>(version).is_err(),
        "unsupported schema versions should be rejected"
    );
}

#[test]
fn test_metadata_schema_v1_round_trip() {
    let schema = MetadataSchema::default();
    let encoded = to_string(&schema).expect("schema should serialize");
    let decoded = from_str(&encoded).expect("schema should deserialize");

    assert_eq!(schema, decoded);

    #[cfg(feature = "json")]
    assert_eq!(
        MetadataSchema::decode_json_slice(encoded.as_bytes()).expect("the bounded JSON schema should decode"),
        schema
    );
}

#[test]
fn test_metadata_schema_rejects_positional_envelope() {
    let input = r#"[1,{},"reject","reject"]"#;

    assert!(
        from_str::<MetadataSchema>(input).is_err(),
        "the strict schema envelope must reject positional input"
    );
    #[cfg(feature = "json")]
    assert!(
        MetadataSchema::decode_json_slice(input.as_bytes()).is_err(),
        "the bounded decoder must reject positional input"
    );
}

#[test]
fn test_metadata_schema_serialization_enforces_hard_key_limit() {
    let long_key = "k".repeat(257);
    let schema = MetadataSchema::builder()
        .optional(&long_key, DataType::String)
        .build()
        .expect("schema should build");

    assert!(
        to_vec(&schema).is_err(),
        "serialization should reject keys over the hard limit"
    );
}

#[test]
fn test_metadata_schema_serialization_enforces_hard_entry_limit() {
    let mut builder = MetadataSchema::builder();
    for index in 0..=4_096 {
        builder = builder.optional(&format!("key-{index}"), DataType::String);
    }
    let schema = builder.build().expect("schema should build");

    assert!(
        to_vec(&schema).is_err(),
        "serialization should reject entries over the hard limit"
    );
}

#[test]
#[cfg(feature = "json")]
fn test_metadata_schema_json_decoder_rejects_unsupported_version() {
    let input = json!({
        "version": 2,
        "fields": {},
        "unknown_metadata_field_policy": "reject",
        "unknown_filter_field_policy": "reject",
    });
    let input = to_vec(&input).expect("input should serialize");

    assert!(
        MetadataSchema::decode_json_slice(&input).is_err(),
        "the decoder should reject unsupported schema versions"
    );
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
    assert!(
        MetadataSchema::decode_json_slice(duplicate_policy).is_err(),
        "the decoder should reject duplicate metadata policies"
    );

    let duplicate_filter_policy = br#"{
        "version": 1,
        "fields": {},
        "unknown_metadata_field_policy": "reject",
        "unknown_filter_field_policy": "reject",
        "unknown_filter_field_policy": "allow_unchecked"
    }"#;
    assert!(
        MetadataSchema::decode_json_slice(duplicate_filter_policy).is_err(),
        "the decoder should reject duplicate filter policies"
    );

    let wrong_fields_type = br#"{
        "version": 1,
        "fields": [],
        "unknown_metadata_field_policy": "reject",
        "unknown_filter_field_policy": "reject"
    }"#;
    assert!(
        MetadataSchema::decode_json_slice(wrong_fields_type).is_err(),
        "the decoder should reject a non-object fields value"
    );

    let invalid_field = br#"{
        "version": 1,
        "fields": {"name": {"invalid": true}},
        "unknown_metadata_field_policy": "reject",
        "unknown_filter_field_policy": "reject"
    }"#;
    assert!(
        MetadataSchema::decode_json_slice(invalid_field).is_err(),
        "the decoder should reject invalid field definitions"
    );
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
        assert!(
            MetadataSchema::decode_json_slice(input).is_err(),
            "the decoder should reject missing or unknown envelope fields"
        );
    }
}

#[test]
#[cfg(feature = "json")]
fn test_metadata_schema_json_decoder_maps_syntax_and_budget_errors() {
    assert!(
        MetadataSchema::decode_json_slice(b"!").is_err(),
        "the decoder should reject malformed JSON syntax"
    );

    let limits = MetadataLimits::builder()
        .json_decode(
            JsonDecodeLimits::builder()
                .input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, 1))
                .build(),
        )
        .build()
        .expect("limits should build");
    assert!(
        MetadataSchema::decode_json_slice_with_limits(b"{}", limits).is_err(),
        "the decoder should reject input exceeding its byte budget"
    );
}
