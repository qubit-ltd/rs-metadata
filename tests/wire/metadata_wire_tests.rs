// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
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
