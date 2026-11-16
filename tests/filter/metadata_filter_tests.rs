// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::MetadataFilter`].

use qubit_datatype::NumericComparisonPolicy;
use qubit_metadata::FilterExpression;
use qubit_metadata::FilterLimits;
use qubit_metadata::FilterMatchOptions;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataFilter;
#[cfg(feature = "json")]
use qubit_metadata::default_json_encode_limits;

use crate::support::test_support::sample;

#[test]
fn test_all_and_none_match_as_constants() {
    let all = MetadataFilter::all();
    let none = MetadataFilter::none();

    assert!(all.matches(&Metadata::new()));
    assert!(all.matches(&sample()));
    assert!(!none.matches(&Metadata::new()));
    assert!(!none.matches(&sample()));
    assert_eq!(all.options(), FilterMatchOptions::default());
    assert_eq!(all.limits(), FilterLimits::MAX);
    assert_eq!(none.options(), FilterMatchOptions::default());
    assert_eq!(none.limits(), FilterLimits::MAX);
}

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
fn test_filter_distinguishes_exact_and_approximate_numeric_policies() {
    let mut metadata = Metadata::new();
    metadata.set("number", 9_007_199_254_740_993_i64);
    let expression = FilterExpression::builder()
        .eq("number", 9_007_199_254_740_992_f64)
        .build()
        .expect("expression should build");
    let exact = MetadataFilter::builder()
        .expression(expression.clone())
        .build()
        .expect("filter should build");
    let options = FilterMatchOptions::builder()
        .numeric_comparison_policy(NumericComparisonPolicy::Approximate)
        .build();
    let approximate = MetadataFilter::builder()
        .expression(expression)
        .options(options)
        .build()
        .expect("filter should build");

    assert!(!exact.matches(&metadata));
    assert!(approximate.matches(&metadata));
}

#[cfg(feature = "json")]
#[test]
fn test_filter_json_encode_and_decode_surface() {
    let filter = MetadataFilter::all();
    let encoded = filter.to_json_vec().expect("filter should encode");
    let decoded = MetadataFilter::decode_json_slice(&encoded).expect("filter should decode");
    assert_eq!(decoded, filter);

    filter
        .to_json_vec_with_limits(default_json_encode_limits())
        .expect("filter should encode with limits");

    let mut buffer = Vec::new();
    filter.to_json_writer(&mut buffer).expect("filter should write");
    filter
        .to_json_writer_with_limits(&mut buffer, default_json_encode_limits())
        .expect("filter should write with limits");
}
