// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the strict Metadata v1 envelope.

use qubit_metadata::Metadata;
use serde_json::json;

#[test]
fn test_metadata_rejects_unknown_field_and_unsupported_version() {
    let unknown = r#"{"version":1,"values":{},"extra":true}"#;
    let version = r#"{"version":2,"values":{}}"#;

    assert!(serde_json::from_str::<Metadata>(unknown).is_err());
    assert!(serde_json::from_str::<Metadata>(version).is_err());
}

#[test]
fn test_metadata_v1_round_trip() {
    let metadata = Metadata::new().with("name", "alice");
    let encoded =
        serde_json::to_string(&metadata).expect("metadata should serialize");
    let decoded =
        serde_json::from_str(&encoded).expect("metadata should deserialize");

    assert_eq!(metadata, decoded);
}

/// Verifies the metadata envelope owns the version while values use V1
/// payloads.
#[test]
fn test_metadata_v1_embeds_unversioned_value_payloads() {
    let metadata = Metadata::new().with("port", 8080_i32);
    let encoded =
        serde_json::to_value(metadata).expect("metadata should serialize");

    assert_eq!(encoded["version"], json!(1));
    assert_eq!(
        encoded["values"]["port"],
        json!({"scalar": {"int32": 8080}})
    );
}

#[test]
#[cfg(feature = "json")]
fn test_metadata_v1_user_guide_example_decodes() {
    const INPUT: &[u8] =
        br#"{"version":1,"values":{"tenant_id":{"scalar":{"string":"acme"}}}}"#;

    let metadata = Metadata::decode_json_slice(INPUT)
        .expect("the user-guide V1 example should decode");

    assert_eq!(metadata.get_str("tenant_id"), Some("acme"));
}

#[test]
fn test_metadata_deserialization_enforces_hard_map_limits() {
    let mut metadata = Metadata::new();
    for index in 0..=4_096 {
        let key = format!("key-{index}");
        metadata.set(&key, index as i64);
    }
    assert!(serde_json::to_vec(&metadata).is_err());

    let long_key = "k".repeat(257);
    let metadata = Metadata::new().with(&long_key, 1_i64);
    assert!(serde_json::to_vec(&metadata).is_err());

    let malformed_after_limit =
        format!(r#"{{"version":1,"values":{{"{long_key}":{{"invalid":}}}}}}"#);
    let error =
        serde_json::from_slice::<Metadata>(malformed_after_limit.as_bytes())
            .expect_err("the hard entry limit should reject before the value");
    assert!(error.to_string().contains("256 bytes"));
}

#[test]
#[cfg(feature = "json")]
fn test_metadata_json_decoder_exercises_seed_error_paths() {
    let wrong_values_type = br#"{"version":1,"values":[]}"#;

    assert!(Metadata::decode_json_slice(wrong_values_type).is_err());

    let invalid_value = br#"{"version":1,"values":{"name":{"invalid":true}}}"#;
    assert!(Metadata::decode_json_slice(invalid_value).is_err());
}

#[test]
#[cfg(feature = "json")]
fn test_metadata_json_decoder_rejects_unsupported_version() {
    let unsupported_version = br#"{"version":2,"values":{}}"#;

    Metadata::decode_json_slice(unsupported_version).expect_err(
        "the bounded JSON decoder must reject unsupported versions",
    );
}

#[test]
#[cfg(feature = "json")]
fn test_metadata_json_decoder_rejects_missing_and_unknown_fields() {
    let missing_version = br#"{"values":{}}"#;
    let missing_values = br#"{"version":1}"#;
    let unknown_field = br#"{"version":1,"values":{},"extra":true}"#;

    for input in [
        missing_version.as_slice(),
        missing_values.as_slice(),
        unknown_field.as_slice(),
    ] {
        assert!(Metadata::decode_json_slice(input).is_err());
    }
}
