// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Unit tests for [`qubit_metadata::MetadataFilterBuilder`] default behavior.
use qubit_metadata::{
    FilterExpression, FilterLimits, FilterMatchOptions, MetadataError, MetadataFilter,
};

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
        .numeric_comparison_policy(qubit_datatype::NumericComparisonPolicy::Approximate)
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
