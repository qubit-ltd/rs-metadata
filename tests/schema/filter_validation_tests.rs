// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Schema validation tests for expression-first filters.

use qubit_datatype::DataType;
use qubit_metadata::{
    FilterExpression, MetadataError, MetadataFilter, MetadataSchema, UnknownFilterFieldPolicy,
};

/// Builds a root filter suitable for schema validation.
fn filter(expression: FilterExpression) -> MetadataFilter {
    MetadataFilter::builder()
        .expression(expression)
        .build()
        .unwrap()
}

#[test]
fn test_schema_accepts_compatible_filter_expression() {
    let schema = MetadataSchema::builder()
        .required("score", DataType::Int64)
        .build()
        .unwrap();
    let expression = FilterExpression::builder()
        .ge("score", 10_i64)
        .build()
        .unwrap();

    assert!(schema.validate_filter(&filter(expression)).is_ok());
}

#[test]
fn test_schema_rejects_incompatible_filter_value() {
    let schema = MetadataSchema::builder()
        .required("score", DataType::Int64)
        .build()
        .unwrap();
    let expression = FilterExpression::builder()
        .eq("score", "ten")
        .build()
        .unwrap();
    let error = schema.validate_filter(&filter(expression)).unwrap_err();

    assert!(matches!(
        error.into_issues().as_slice(),
        [MetadataError::InvalidFilterOperator { operator: "eq", .. }]
    ));
}

#[test]
fn test_filter_builder_rejects_nan_numeric_filter_value() {
    let error = FilterExpression::builder()
        .eq("score", f64::NAN)
        .build()
        .unwrap_err();

    assert!(matches!(
        error,
        MetadataError::InvalidFilterOperand {
            operator: "eq",
            data_type: DataType::Float64,
            ..
        }
    ));
}

#[test]
fn test_schema_accepts_string_range_filter() {
    let schema = MetadataSchema::builder()
        .required("name", DataType::String)
        .build()
        .unwrap();
    let expression = FilterExpression::builder()
        .ge("name", "alice")
        .lt("name", "zebra")
        .build()
        .unwrap();

    assert!(schema.validate_filter(&filter(expression)).is_ok());
}

#[test]
fn test_schema_rejects_range_filter_for_non_range_field() {
    let schema = MetadataSchema::builder()
        .required("enabled", DataType::Bool)
        .build()
        .unwrap();
    let expression = FilterExpression::builder()
        .gt("enabled", false)
        .build()
        .unwrap();
    let error = schema.validate_filter(&filter(expression)).unwrap_err();

    assert!(matches!(
        error.into_issues().as_slice(),
        [MetadataError::InvalidFilterOperator { operator: "gt", .. }]
    ));
}

#[test]
fn test_schema_honors_unknown_filter_field_policy() {
    let expression = FilterExpression::builder()
        .exists("dynamic")
        .build()
        .unwrap();
    let strict = MetadataSchema::builder().build().unwrap();
    let permissive = MetadataSchema::builder()
        .unknown_filter_field_policy(UnknownFilterFieldPolicy::AllowUnchecked)
        .build()
        .unwrap();

    assert!(strict.validate_filter(&filter(expression.clone())).is_err());
    assert!(permissive.validate_filter(&filter(expression)).is_ok());
}
