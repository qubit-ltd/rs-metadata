/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Tests for metadata schema data types.

use qubit_common::DataType;
use qubit_metadata::{Metadata, MetadataError, MetadataFilter, MetadataSchema, UnknownFieldPolicy};

#[test]
fn schema_builder_defines_required_and_optional_fields() {
    let schema = MetadataSchema::builder()
        .required("id", DataType::String)
        .optional("score", DataType::Int64)
        .unknown_field_policy(UnknownFieldPolicy::Allow)
        .build();

    assert_eq!(schema.field_type("id"), Some(DataType::String));
    assert_eq!(schema.field_type("score"), Some(DataType::Int64));
    assert_eq!(schema.field_type("missing"), None);
    assert_eq!(schema.unknown_field_policy(), UnknownFieldPolicy::Allow);
    assert!(schema.field("id").unwrap().is_required());
    assert!(!schema.field("score").unwrap().is_required());
}

#[test]
fn schema_validate_accepts_matching_metadata() {
    let schema = MetadataSchema::builder()
        .required("id", DataType::String)
        .optional("score", DataType::Int64)
        .build();
    let meta = Metadata::new().with("id", "doc-1").with("score", 42_i64);

    assert_eq!(schema.validate(&meta), Ok(()));
}

#[test]
fn schema_validate_reports_missing_required_field() {
    let schema = MetadataSchema::builder()
        .required("id", DataType::String)
        .build();
    let meta = Metadata::new();

    let error = schema.validate(&meta).unwrap_err();
    assert_eq!(
        error,
        MetadataError::MissingRequiredField {
            key: "id".to_string(),
            expected: DataType::String,
        }
    );
}

#[test]
fn schema_validate_reports_type_mismatch() {
    let schema = MetadataSchema::builder()
        .required("score", DataType::Int64)
        .build();
    let meta = Metadata::new().with("score", "high");

    match schema.validate(&meta).unwrap_err() {
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
fn schema_validate_reports_unknown_field_by_default() {
    let schema = MetadataSchema::builder()
        .required("id", DataType::String)
        .build();
    let meta = Metadata::new().with("id", "doc-1").with("extra", true);

    assert_eq!(
        schema.validate(&meta),
        Err(MetadataError::UnknownField {
            key: "extra".to_string(),
        })
    );
}

#[test]
fn schema_validate_can_allow_unknown_fields() {
    let schema = MetadataSchema::builder()
        .required("id", DataType::String)
        .unknown_field_policy(UnknownFieldPolicy::Allow)
        .build();
    let meta = Metadata::new().with("id", "doc-1").with("extra", true);

    assert_eq!(schema.validate(&meta), Ok(()));
}

#[test]
fn schema_validate_filter_accepts_numeric_literal_compatibility() {
    let schema = MetadataSchema::builder()
        .required("score", DataType::Int64)
        .build();
    let filter = MetadataFilter::builder()
        .ge("score", 10)
        .build_checked(&schema)
        .unwrap();
    let meta = Metadata::new().with("score", 42_i64);

    assert!(filter.matches(&meta));
}

#[test]
fn schema_validate_filter_accepts_big_number_fields() {
    let schema = MetadataSchema::builder()
        .required("amount", DataType::BigDecimal)
        .required("count", DataType::BigInteger)
        .build();

    let filter = MetadataFilter::builder()
        .ge("amount", bigdecimal::BigDecimal::from(10_i64))
        .and_eq("count", num_bigint::BigInt::from(3_i64))
        .build_checked(&schema)
        .unwrap();

    let meta = Metadata::new()
        .with("amount", bigdecimal::BigDecimal::from(11_i64))
        .with("count", num_bigint::BigInt::from(3_i64));
    assert!(filter.matches(&meta));
}

#[test]
fn schema_default_rejects_unknown_fields() {
    let schema = MetadataSchema::default();
    let meta = Metadata::new().with("extra", true);

    assert_eq!(
        schema.validate(&meta),
        Err(MetadataError::UnknownField {
            key: "extra".to_string(),
        })
    );
}

#[test]
fn schema_validate_filter_visits_all_condition_kinds() {
    let schema = MetadataSchema::builder()
        .required("score", DataType::Int64)
        .required("status", DataType::String)
        .optional("tag", DataType::String)
        .build();

    let filter = MetadataFilter::builder()
        .lt("score", 100_i64)
        .and_le("score", 100_i64)
        .and_gt("score", 1_i64)
        .and_in_set("status", ["active", "pending"])
        .and_not_in_set("tag", ["archived"])
        .and_exists("status")
        .and_not_exists("tag")
        .not()
        .build_checked(&schema)
        .unwrap();

    schema.validate_filter(&filter).unwrap();
    schema.validate_filter(&MetadataFilter::none()).unwrap();
}

#[test]
fn schema_validate_filter_accepts_not_equal_condition() {
    let schema = MetadataSchema::builder()
        .required("status", DataType::String)
        .build();

    MetadataFilter::builder()
        .ne("status", "inactive")
        .build_checked(&schema)
        .unwrap();
}

#[test]
fn schema_validate_filter_rejects_incompatible_value_predicate() {
    let schema = MetadataSchema::builder()
        .required("status", DataType::String)
        .build();
    let error = MetadataFilter::builder()
        .eq("status", 1_i64)
        .build_checked(&schema)
        .unwrap_err();

    match error {
        MetadataError::InvalidFilterOperator {
            key,
            operator,
            data_type,
            message,
        } => {
            assert_eq!(key, "status");
            assert_eq!(operator, "eq");
            assert_eq!(data_type, DataType::String);
            assert!(message.contains("not compatible"));
        }
        other => panic!("expected InvalidFilterOperator, got {other:?}"),
    }
}

#[test]
fn schema_validate_filter_rejects_unknown_value_predicate_field() {
    let schema = MetadataSchema::builder()
        .required("status", DataType::String)
        .build();
    let error = MetadataFilter::builder()
        .eq("missing", "active")
        .build_checked(&schema)
        .unwrap_err();

    assert_eq!(
        error,
        MetadataError::UnknownFilterField {
            key: "missing".to_string(),
        }
    );
}

#[test]
fn schema_validate_filter_rejects_incompatible_not_in_value() {
    let schema = MetadataSchema::builder()
        .required("status", DataType::String)
        .build();
    let error = MetadataFilter::builder()
        .not_in_set("status", [1_i64])
        .build_checked(&schema)
        .unwrap_err();

    match error {
        MetadataError::InvalidFilterOperator {
            key,
            operator,
            data_type,
            message,
        } => {
            assert_eq!(key, "status");
            assert_eq!(operator, "in_set");
            assert_eq!(data_type, DataType::String);
            assert!(message.contains("not compatible"));
        }
        other => panic!("expected InvalidFilterOperator, got {other:?}"),
    }
}

#[test]
fn schema_validate_filter_rejects_unknown_exists_field() {
    let schema = MetadataSchema::builder()
        .required("status", DataType::String)
        .build();
    let error = MetadataFilter::builder()
        .exists("missing")
        .build_checked(&schema)
        .unwrap_err();

    assert_eq!(
        error,
        MetadataError::UnknownFilterField {
            key: "missing".to_string(),
        }
    );
}

#[test]
fn schema_validate_filter_rejects_incompatible_range_value() {
    let schema = MetadataSchema::builder()
        .required("status", DataType::String)
        .build();
    let error = MetadataFilter::builder()
        .gt("status", 1_i64)
        .build_checked(&schema)
        .unwrap_err();

    match error {
        MetadataError::InvalidFilterOperator {
            key,
            operator,
            data_type,
            message,
        } => {
            assert_eq!(key, "status");
            assert_eq!(operator, "gt");
            assert_eq!(data_type, DataType::String);
            assert!(message.contains("not compatible"));
        }
        other => panic!("expected InvalidFilterOperator, got {other:?}"),
    }
}

#[test]
fn schema_validate_filter_reports_unknown_field() {
    let schema = MetadataSchema::builder()
        .required("score", DataType::Int64)
        .build();
    let error = MetadataFilter::builder()
        .ge("unknown", 10_i64)
        .build_checked(&schema)
        .unwrap_err();

    assert_eq!(
        error,
        MetadataError::UnknownFilterField {
            key: "unknown".to_string(),
        }
    );
}

#[test]
fn schema_validate_filter_rejects_range_on_bool() {
    let schema = MetadataSchema::builder()
        .required("active", DataType::Bool)
        .build();
    let error = MetadataFilter::builder()
        .gt("active", true)
        .build_checked(&schema)
        .unwrap_err();

    match error {
        MetadataError::InvalidFilterOperator {
            key,
            operator,
            data_type,
            ..
        } => {
            assert_eq!(key, "active");
            assert_eq!(operator, "gt");
            assert_eq!(data_type, DataType::Bool);
        }
        other => panic!("expected InvalidFilterOperator, got {other:?}"),
    }
}

#[test]
fn schema_fields_iterates_in_key_order() {
    let schema = MetadataSchema::builder()
        .optional("z", DataType::Bool)
        .optional("a", DataType::String)
        .build();
    let keys: Vec<&str> = schema.fields().map(|(key, _)| key).collect();

    assert_eq!(keys, vec!["a", "z"]);
}
