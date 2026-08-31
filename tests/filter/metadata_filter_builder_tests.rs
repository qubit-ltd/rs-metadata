// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Unit tests for [`qubit_metadata::MetadataFilterBuilder`] default behavior.
#[cfg(feature = "schema")]
use qubit_datatype::DataType;
use qubit_datatype::NumericComparisonPolicy;
use qubit_metadata::FilterExpression;
use qubit_metadata::FilterExpressionView;
use qubit_metadata::FilterLimitKind;
use qubit_metadata::FilterLimits;
use qubit_metadata::FilterMatchOptions;
use qubit_metadata::MetadataError;
use qubit_metadata::MetadataFilter;
use qubit_metadata::MetadataFilterBuilder;
#[cfg(feature = "schema")]
use qubit_metadata::MetadataSchema;

#[test]
fn test_builder_requires_expression() {
    let error = MetadataFilter::builder()
        .build()
        .expect_err("a filter must have an expression");

    assert!(matches!(error, MetadataError::MissingFilterExpression));
}

#[test]
fn test_builder_uses_last_option_and_limit_values() {
    let expression = FilterExpression::builder()
        .exists("status")
        .build()
        .expect("expression should build");
    let first_options = FilterMatchOptions::builder().build();
    let final_options = FilterMatchOptions::builder()
        .numeric_comparison_policy(NumericComparisonPolicy::Approximate)
        .build();
    let first_limits = FilterLimits::builder().max_nodes(2).build().unwrap();
    let final_limits = FilterLimits::builder().max_nodes(3).build().unwrap();
    let filter = MetadataFilter::builder()
        .expression(expression)
        .options(first_options)
        .options(final_options)
        .limits(first_limits)
        .limits(final_limits)
        .build()
        .expect("filter should build");

    assert_eq!(filter.options(), final_options);
    assert_eq!(filter.limits(), final_limits);
}

#[test]
fn test_default_builder_builds_a_filter() {
    let expression = FilterExpression::builder()
        .exists("status")
        .build()
        .expect("expression should build");
    let filter = MetadataFilterBuilder::default()
        .expression(expression)
        .build()
        .expect("default builder should build a filter");

    assert!(matches!(filter.expression().view(), FilterExpressionView::Condition(_)));
}

#[test]
#[cfg(feature = "schema")]
fn test_build_checked_accepts_a_schema_compatible_filter() {
    let schema = MetadataSchema::builder()
        .required("status", DataType::String)
        .build()
        .expect("schema should build");
    let expression = FilterExpression::builder()
        .eq("status", "ready")
        .build()
        .expect("expression should build");

    let filter = MetadataFilter::builder()
        .expression(expression)
        .build_checked(&schema)
        .expect("schema-compatible filter should build");

    assert!(matches!(filter.expression().view(), FilterExpressionView::Condition(_)));
}

#[test]
fn test_builder_rejects_expression_over_its_configured_limits() {
    let expression = (0..2)
        .fold(FilterExpression::builder(), |builder, index| {
            builder.exists(&format!("field-{index}"))
        })
        .build()
        .expect("expression should build");
    let error = MetadataFilter::builder()
        .expression(expression)
        .limits(FilterLimits::builder().max_nodes(1).build().unwrap())
        .build()
        .expect_err("the configured node limit should reject the expression");

    assert_eq!(
        error,
        MetadataError::FilterLimitExceeded {
            kind: FilterLimitKind::Nodes,
            value: 2,
            maximum: 1,
        }
    );
}

#[test]
#[cfg(feature = "schema")]
fn test_build_checked_rejects_schema_incompatible_filter() {
    let schema = MetadataSchema::builder()
        .required("status", DataType::Bool)
        .build()
        .expect("schema should build");
    let expression = FilterExpression::builder()
        .eq("status", "ready")
        .build()
        .expect("expression should build");

    let error = MetadataFilter::builder()
        .expression(expression)
        .build_checked(&schema)
        .expect_err("schema-incompatible filter should be rejected");

    assert!(!error.issues().is_empty());
}

#[test]
#[cfg(feature = "schema")]
fn test_build_checked_propagates_missing_expression() {
    let schema = MetadataSchema::builder().build().expect("schema should build");

    let error = MetadataFilter::builder()
        .build_checked(&schema)
        .expect_err("missing expression should be reported");

    assert!(matches!(error.issues(), [MetadataError::MissingFilterExpression]));
}
