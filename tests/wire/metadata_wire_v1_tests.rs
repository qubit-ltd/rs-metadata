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
fn test_metadata_deserialization_enforces_hard_map_limits() {
    let mut metadata = Metadata::new();
    for index in 0..=4_096 {
        let key = format!("key-{index}");
        metadata.set(&key, index as i64);
    }
    let encoded = serde_json::to_vec(&metadata)
        .expect("metadata with many entries should serialize");

    assert!(serde_json::from_slice::<Metadata>(&encoded).is_err());

    let long_key = "k".repeat(257);
    let metadata = Metadata::new().with(&long_key, 1_i64);
    let encoded = serde_json::to_vec(&metadata)
        .expect("metadata with a long key should serialize");

    assert!(serde_json::from_slice::<Metadata>(&encoded).is_err());

    let malformed_after_limit = format!(
        r#"{{"version":1,"values":{{"{long_key}":{{"invalid":}}}}}}"#
    );
    let error = serde_json::from_slice::<Metadata>(malformed_after_limit.as_bytes())
        .expect_err("the hard entry limit should reject before the value");
    assert!(error.to_string().contains("256 bytes"));
}
