// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for metadata schema data types.

use qubit_datatype::DataType;
use qubit_metadata::{
    Metadata,
    MetadataError,
    MetadataSchema,
    MetadataValidationError,
    UnknownFilterFieldPolicy,
    UnknownMetadataFieldPolicy,
};
use qubit_value::Value;
use serde_json::json;

fn single_issue(error: MetadataValidationError) -> MetadataError {
    let mut issues = error.into_issues();
    assert_eq!(issues.len(), 1);
    issues.remove(0)
}

#[test]
fn test_schema_builder_defines_required_and_optional_fields() {
    let schema = MetadataSchema::builder()
        .required("id", DataType::String)
        .optional("score", DataType::Int64)
        .unknown_metadata_field_policy(UnknownMetadataFieldPolicy::Allow)
        .unknown_filter_field_policy(UnknownFilterFieldPolicy::AllowUnchecked)
        .build()
        .unwrap();

    assert_eq!(schema.field_type("id"), Some(DataType::String));
    assert_eq!(schema.field_type("score"), Some(DataType::Int64));
    assert_eq!(schema.field_type("missing"), None);
    assert_eq!(
        schema.unknown_metadata_field_policy(),
        UnknownMetadataFieldPolicy::Allow
    );
    assert_eq!(
        schema.unknown_filter_field_policy(),
        UnknownFilterFieldPolicy::AllowUnchecked
    );
    assert!(schema.field("id").unwrap().is_required());
    assert!(!schema.field("score").unwrap().is_required());
}

#[test]
fn test_schema_validate_accepts_matching_metadata() {
    let schema = MetadataSchema::builder()
        .required("id", DataType::String)
        .optional("score", DataType::Int64)
        .build()
        .expect("schema should build");
    let meta = Metadata::new().with("id", "doc-1").with("score", 42_i64);

    assert_eq!(schema.validate(&meta), Ok(()));
}

#[test]
fn test_schema_validate_reports_missing_required_field() {
    let schema = MetadataSchema::builder()
        .required("id", DataType::String)
        .build()
        .expect("schema should build");
    let meta = Metadata::new();

    let error = single_issue(schema.validate(&meta).unwrap_err());
    assert_eq!(
        error,
        MetadataError::MissingRequiredField {
            key: "id".to_string(),
            expected: DataType::String,
        }
    );
}

#[test]
fn test_schema_validate_treats_unset_required_field_as_missing() {
    let schema = MetadataSchema::builder()
        .required("id", DataType::String)
        .build()
        .expect("schema should build");
    let metadata =
        Metadata::new().with_raw("id", Value::Unset(DataType::String));

    assert_eq!(
        schema.validate(&metadata),
        Err(MetadataValidationError::from_issue(
            MetadataError::MissingRequiredField {
                key: "id".to_string(),
                expected: DataType::String,
            },
        ))
    );
}

#[test]
fn test_schema_validate_accepts_unset_optional_field() {
    let schema = MetadataSchema::builder()
        .optional("score", DataType::Int64)
        .build()
        .expect("schema should build");
    let metadata =
        Metadata::new().with_raw("score", Value::Unset(DataType::Int64));

    assert_eq!(schema.validate(&metadata), Ok(()));
}

#[test]
fn test_schema_validate_reports_type_mismatch() {
    let schema = MetadataSchema::builder()
        .required("score", DataType::Int64)
        .build()
        .expect("schema should build");
    let meta = Metadata::new().with("score", "high");

    match single_issue(schema.validate(&meta).unwrap_err()) {
        MetadataError::TypeMismatch {
            key,
            expected,
            actual,
            ..
        } => {
            assert_eq!(key, "score");
            assert_eq!(expected, DataType::Int64);
            assert_eq!(actual, DataType::String);
        }
        other => panic!("expected TypeMismatch, got {other:?}"),
    }
}

#[test]
fn test_schema_validate_reports_unknown_field_by_default() {
    let schema = MetadataSchema::builder()
        .required("id", DataType::String)
        .build()
        .expect("schema should build");
    let meta = Metadata::new().with("id", "doc-1").with("extra", true);

    assert_eq!(
        schema.validate(&meta),
        Err(MetadataValidationError::from_issue(
            MetadataError::UnknownField {
                key: "extra".to_string(),
            },
        ))
    );
}

#[test]
fn test_schema_validate_collects_all_metadata_issues() {
    let schema = MetadataSchema::builder()
        .required("id", DataType::String)
        .required("score", DataType::Int64)
        .build()
        .expect("schema should build");
    let meta = Metadata::new().with("score", "high").with("extra", true);

    let error = schema.validate(&meta).unwrap_err();
    let issues = error.issues();

    assert_eq!(issues.len(), 3);
    assert!(matches!(
        &issues[0],
        MetadataError::MissingRequiredField { key, .. } if key == "id"
    ));
    assert!(matches!(
        &issues[1],
        MetadataError::UnknownField { key } if key == "extra"
    ));
    assert!(matches!(
        &issues[2],
        MetadataError::TypeMismatch { key, .. } if key == "score"
    ));
}

#[test]
fn test_schema_validate_can_allow_unknown_fields() {
    let schema = MetadataSchema::builder()
        .required("id", DataType::String)
        .unknown_metadata_field_policy(UnknownMetadataFieldPolicy::Allow)
        .build()
        .expect("schema should build");
    let meta = Metadata::new().with("id", "doc-1").with("extra", true);

    assert_eq!(schema.validate(&meta), Ok(()));
}

#[test]
fn test_schema_metadata_and_filter_unknown_field_policies_are_independent() {
    let schema = MetadataSchema::builder()
        .unknown_metadata_field_policy(UnknownMetadataFieldPolicy::Allow)
        .build()
        .expect("schema should build");

    assert_eq!(
        schema.validate(&Metadata::new().with("dynamic", true)),
        Ok(())
    );
    let filter = qubit_metadata::MetadataFilter::builder()
        .exists("dynamic")
        .build()
        .expect("filter should build");
    assert!(matches!(
        schema.validate_filter(&filter),
        Err(error) if error.issues() == [MetadataError::UnknownFilterField {
            key: "dynamic".to_string(),
        }]
    ));
}

#[test]
fn test_schema_deserialize_rejects_unknown_struct_fields() {
    let error = serde_json::from_value::<MetadataSchema>(json!({
        "fields": {},
        "unknown_metadata_field_policy": "reject",
        "unknown_filter_field_policy": "reject",
        "unexpected": true,
    }))
    .expect_err("schema must reject unknown serialized fields")
    .to_string();

    assert!(error.contains("unknown field `unexpected`"));
}

#[test]
fn test_schema_default_rejects_unknown_fields() {
    let schema = MetadataSchema::default();
    let meta = Metadata::new().with("extra", true);

    assert_eq!(
        schema.validate(&meta),
        Err(MetadataValidationError::from_issue(
            MetadataError::UnknownField {
                key: "extra".to_string(),
            },
        ))
    );
}

#[test]
fn test_schema_fields_iterates_in_key_order() {
    let schema = MetadataSchema::builder()
        .optional("z", DataType::Bool)
        .optional("a", DataType::String)
        .build()
        .expect("schema should build");
    let keys: Vec<&str> = schema.fields().map(|(key, _)| key).collect();

    assert_eq!(keys, vec!["a", "z"]);
}
