// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::MetadataFilter`].

use crate::support::test_support::sample;
use qubit_datatype::NumericComparisonPolicy;
use qubit_metadata::{FilterExpression, FilterMatchOptions, Metadata, MetadataFilter};

#[test]
fn test_filter_matches_its_expression() {
    let expression = FilterExpression::builder()
        .eq("status", "active")
        .ge("score", 40_i64)
        .build()
        .expect("expression should build");
    let filter = MetadataFilter::builder()
        .expression(expression)
        .build()
        .expect("filter should build");

    assert!(filter.matches(&sample()));
}

#[test]
fn test_filter_uses_its_configured_options() {
    let mut metadata = Metadata::new();
    metadata.set("number", 9_007_199_254_740_993_i64);
    let expression = FilterExpression::builder()
        .eq("number", 9_007_199_254_740_992_f64)
        .build()
        .expect("expression should build");
    let options = FilterMatchOptions::builder()
        .numeric_comparison_policy(NumericComparisonPolicy::Approximate)
        .build();
    let filter = MetadataFilter::builder()
        .expression(expression)
        .options(options)
        .build()
        .expect("filter should build");

    assert!(filter.matches(&metadata));
}
