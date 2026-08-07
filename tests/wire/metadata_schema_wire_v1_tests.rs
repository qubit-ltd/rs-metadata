// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the strict MetadataSchema v1 envelope.

use qubit_metadata::MetadataSchema;

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
