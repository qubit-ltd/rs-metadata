// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for duplicate-key rejection by strict wire maps.

#[cfg(feature = "schema")]
use qubit_datatype::DataType;
use qubit_metadata::Metadata;
#[cfg(feature = "schema")]
use qubit_metadata::{MetadataField, MetadataSchema};

#[test]
fn test_metadata_rejects_duplicate_value_key() {
    let encoded = r#"
        {
            "version": 1,
            "values": {
                "name": { "scalar": { "string": "alice" } },
                "name": { "scalar": { "string": "bob" } }
            }
        }
    "#;

    let error = serde_json::from_str::<Metadata>(encoded).unwrap_err();
    assert!(error.to_string().contains("duplicate map key 'name'"));
}

#[test]
fn test_metadata_rejects_duplicate_key_before_decoding_duplicate_value() {
    let encoded = r#"
        {
            "version": 1,
            "values": {
                "name": { "scalar": { "string": "alice" } },
                "name": { "invalid": }
            }
        }
    "#;

    let error = serde_json::from_str::<Metadata>(encoded).unwrap_err();
    assert!(error.to_string().contains("duplicate map key 'name'"));
}

#[test]
#[cfg(feature = "schema")]
fn test_metadata_schema_rejects_duplicate_field_key() {
    let field =
        serde_json::to_string(&MetadataField::new(DataType::String, true))
            .expect("field should serialize");
    let encoded = format!(
        r#"
            {{
                "version": 1,
                "fields": {{ "name": {field}, "name": {field} }},
                "unknown_metadata_field_policy": "reject",
                "unknown_filter_field_policy": "reject"
            }}
        "#
    );

    let error = serde_json::from_str::<MetadataSchema>(&encoded).unwrap_err();
    assert!(error.to_string().contains("duplicate map key 'name'"));
}
