// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Schema validation tests for expression-first filters.

use qubit_datatype::DataType;
use qubit_metadata::FilterExpression;
use qubit_metadata::MetadataError;
use qubit_metadata::MetadataFilter;
use qubit_metadata::MetadataSchema;
use qubit_metadata::UnknownFilterFieldPolicy;

/// Builds a root filter suitable for schema validation.
fn filter(expression: FilterExpression) -> MetadataFilter {
    MetadataFilter::builder().expression(expression).build().unwrap()
}

#[test]
fn test_schema_accepts_compatible_filter_expression() {
    let schema = MetadataSchema::builder()
        .required("score", DataType::Int64)
        .build()
        .unwrap();
    let expression = FilterExpression::builder().ge("score", 10_i64).build().unwrap();

    assert!(schema.validate_filter(&filter(expression)).is_ok());
}

#[test]
fn test_schema_rejects_incompatible_filter_value() {
    let schema = MetadataSchema::builder()
        .required("score", DataType::Int64)
        .build()
        .unwrap();
    let expression = FilterExpression::builder().eq("score", "ten").build().unwrap();
    let error = schema.validate_filter(&filter(expression)).unwrap_err();

    assert!(matches!(
        error.into_issues().as_slice(),
        [MetadataError::InvalidFilterOperator { operator: "eq", .. }]
    ));
}

#[test]
fn test_filter_builder_rejects_nan_numeric_filter_value() {
    let error = FilterExpression::builder().eq("score", f64::NAN).build().unwrap_err();

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
    let expression = FilterExpression::builder().gt("enabled", false).build().unwrap();
    let error = schema.validate_filter(&filter(expression)).unwrap_err();

    assert!(matches!(
        error.into_issues().as_slice(),
        [MetadataError::InvalidFilterOperator { operator: "gt", .. }]
    ));
}

#[test]
fn test_schema_honors_unknown_filter_field_policy() {
    let expression = FilterExpression::builder().exists("dynamic").build().unwrap();
    let strict = MetadataSchema::builder().build().unwrap();
    let permissive = MetadataSchema::builder()
        .unknown_filter_field_policy(UnknownFilterFieldPolicy::AllowUnchecked)
        .build()
        .unwrap();

    assert!(strict.validate_filter(&filter(expression.clone())).is_err());
    assert!(permissive.validate_filter(&filter(expression)).is_ok());
}

#[test]
fn test_schema_filter_validation_covers_all_operator_families_and_set_errors() {
    let numeric = MetadataSchema::builder()
        .required("score", DataType::Int64)
        .build()
        .expect("schema should build");
    let invalid = [
        FilterExpression::builder().ne("score", "bad").build(),
        FilterExpression::builder().lt("score", "bad").build(),
        FilterExpression::builder().le("score", "bad").build(),
        FilterExpression::builder().gt("score", "bad").build(),
        FilterExpression::builder().ge("score", "bad").build(),
        FilterExpression::builder().in_set("score", ["bad"]).build(),
        FilterExpression::builder().not_in_set("score", ["bad"]).build(),
    ];
    for expression in invalid {
        let error = numeric.validate_filter(&filter(expression.expect("filter expression should build")));
        assert!(error.is_err(), "incompatible operator must be rejected");
    }

    let bool_schema = MetadataSchema::builder()
        .required("enabled", DataType::Bool)
        .build()
        .expect("schema should build");
    let error = bool_schema
        .validate_filter(&filter(
            FilterExpression::builder()
                .eq("enabled", 1_i64)
                .build()
                .expect("expression should build"),
        ))
        .expect_err("wrong scalar type must be rejected");
    assert!(matches!(
        error.into_issues().as_slice(),
        [MetadataError::InvalidFilterOperator { operator: "eq", .. }]
    ));
}

#[test]
fn test_schema_filter_validation_aggregates_unknown_and_incompatible_conditions() {
    let schema = MetadataSchema::builder()
        .required("enabled", DataType::Bool)
        .required("score", DataType::Int64)
        .build()
        .expect("schema should build");
    let expression = FilterExpression::builder()
        .eq("enabled", 1_i64)
        .exists("missing")
        .gt("score", "bad")
        .build()
        .expect("expression should build");
    let issues = schema
        .validate_filter(&filter(expression))
        .expect_err("all incompatible conditions should be reported")
        .into_issues();
    assert_eq!(issues.len(), 3);
    assert!(matches!(
        issues[0],
        MetadataError::InvalidFilterOperator { operator: "eq", .. }
    ));
    assert!(matches!(issues[1], MetadataError::UnknownFilterField { ref key } if key == "missing"));
    assert!(matches!(
        issues[2],
        MetadataError::InvalidFilterOperator { operator: "gt", .. }
    ));
}
