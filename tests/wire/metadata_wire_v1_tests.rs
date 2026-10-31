// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the strict Metadata v1 envelope.

use qubit_metadata::Metadata;
use serde_json::from_slice;
use serde_json::from_str;
use serde_json::json;
use serde_json::to_string;
use serde_json::to_value;
use serde_json::to_vec;

#[test]
fn test_metadata_rejects_unknown_field_and_unsupported_version() {
    let unknown = r#"{"version":1,"values":{},"extra":true}"#;
    let version = r#"{"version":2,"values":{}}"#;

    assert!(
        from_str::<Metadata>(unknown).is_err(),
        "the V1 envelope should reject unknown fields"
    );
    assert!(
        from_str::<Metadata>(version).is_err(),
        "the V1 envelope should reject unsupported versions"
    );
}

#[test]
fn test_metadata_v1_round_trip() {
    let metadata = Metadata::new().with("name", "alice");
    let encoded = to_string(&metadata).expect("metadata should serialize");
    let decoded = from_str(&encoded).expect("metadata should deserialize");

    assert_eq!(metadata, decoded);
}

/// Verifies the metadata envelope owns the version while values use V1
/// payloads.
#[test]
fn test_metadata_v1_embeds_unversioned_value_payloads() {
    let metadata = Metadata::new().with("port", 8080_i32);
    let encoded = to_value(metadata).expect("metadata should serialize");

    assert_eq!(encoded["version"], json!(1), "the envelope should own version 1");
    assert_eq!(
        encoded["values"]["port"],
        json!({"scalar": {"int32": 8080}}),
        "the values map should contain an unversioned V1 payload"
    );
}

#[test]
#[cfg(feature = "json")]
fn test_metadata_v1_user_guide_example_decodes() {
    const INPUT: &[u8] = br#"{"version":1,"values":{"tenant_id":{"scalar":{"string":"acme"}}}}"#;

    let metadata = Metadata::decode_json_slice(INPUT).expect("the user-guide V1 example should decode");

    assert_eq!(
        metadata.get_ref::<str>("tenant_id"),
        Ok("acme"),
        "the user-guide tenant ID should decode"
    );
}

#[test]
fn test_metadata_deserialization_enforces_hard_map_limits() {
    let mut metadata = Metadata::new();
    for index in 0..=4_096 {
        let key = format!("key-{index}");
        metadata.set(&key, index as i64);
    }
    assert!(
        to_vec(&metadata).is_err(),
        "serialization should reject more than 4096 entries"
    );

    let long_key = "k".repeat(257);
    let metadata = Metadata::new().with(&long_key, 1_i64);
    assert!(
        to_vec(&metadata).is_err(),
        "serialization should reject keys longer than 256 bytes"
    );

    let malformed_after_limit = format!(r#"{{"version":1,"values":{{"{long_key}":{{"invalid":}}}}}}"#);
    let error = from_slice::<Metadata>(malformed_after_limit.as_bytes())
        .expect_err("the hard entry limit should reject before the value");
    assert!(
        error.to_string().contains("256 bytes"),
        "the entry-limit error should explain the 256-byte key limit"
    );
}

#[test]
fn test_metadata_deserialization_enforces_hard_entry_boundary() {
    let template =
        to_value(Metadata::new().with("sample", 1_i64)).expect("template should serialize")["values"]["sample"].clone();
    for count in [4_096, 4_097] {
        let values: serde_json::Map<String, serde_json::Value> = (0..count)
            .map(|index| (format!("key-{index:04}"), template.clone()))
            .collect();
        let input =
            to_string(&json!({"version": 1, "values": values})).expect("raw JSON should serialize beyond model limits");
        let decoded = from_str::<Metadata>(&input);
        if count == 4_096 {
            let metadata = decoded.expect("4096 entries should decode");
            assert_eq!(metadata.len(), count, "all entries should survive");
            assert_eq!(metadata.get::<i64>("key-4095"), Ok(1));
        } else {
            let error = decoded.expect_err("4097 entries should exceed the hard limit");
            assert!(error.to_string().contains("4096 entries"), "wrong limit error: {error}");
        }
    }
}

#[test]
#[cfg(feature = "json")]
fn test_metadata_json_decoder_exercises_seed_error_paths() {
    let wrong_values_type = br#"{"version":1,"values":[]}"#;

    assert!(
        Metadata::decode_json_slice(wrong_values_type).is_err(),
        "the JSON decoder should reject a non-object values field"
    );

    let invalid_value = br#"{"version":1,"values":{"name":{"invalid":true}}}"#;
    assert!(
        Metadata::decode_json_slice(invalid_value).is_err(),
        "the JSON decoder should reject an invalid metadata value"
    );
}

#[test]
#[cfg(feature = "json")]
fn test_metadata_json_decoder_rejects_unsupported_version() {
    let unsupported_version = br#"{"version":2,"values":{}}"#;

    let _ = Metadata::decode_json_slice(unsupported_version)
        .expect_err("the bounded JSON decoder must reject unsupported versions");
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
        assert!(
            Metadata::decode_json_slice(input).is_err(),
            "the JSON decoder should reject missing required fields and unknown fields"
        );
    }
}
