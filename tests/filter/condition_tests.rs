// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Predicate evaluation tests through the public expression API.

use crate::support::test_support::sample;
use qubit_metadata::{
    FilterExpression,
    Metadata,
    MetadataFilter,
};

/// Builds a filter using default root configuration.
fn filter(expression: FilterExpression) -> MetadataFilter {
    MetadataFilter::builder()
        .expression(expression)
        .build()
        .expect("filter should build")
}

#[test]
fn test_comparison_predicates_match_concrete_values() {
    let expression = FilterExpression::builder()
        .eq("status", "active")
        .ne("status", "inactive")
        .gt("score", 10_i64)
        .ge("score", 42_i64)
        .lt("score", 100_i64)
        .le("score", 42_i64)
        .build()
        .expect("expression should build");

    assert!(filter(expression).matches(&sample()));
}

#[test]
fn test_membership_and_existence_predicates_match() {
    let expression = FilterExpression::builder()
        .in_set("tag", ["rust", "java"])
        .not_in_set("tag", ["python"])
        .exists("verified")
        .not_exists("missing")
        .build()
        .expect("expression should build");

    assert!(filter(expression).matches(&sample()));
}

#[test]
fn test_missing_comparison_stays_unknown_through_not() {
    let expression = FilterExpression::builder()
        .eq("missing", "value")
        .not()
        .build()
        .expect("expression should build");

    assert!(!filter(expression).matches(&Metadata::new()));
}

#[test]
fn test_not_equal_is_not_equivalent_to_negated_equal_for_missing_value() {
    let not_equal = FilterExpression::builder()
        .ne("missing", "value")
        .build()
        .expect("expression should build");
    let negated_equal = FilterExpression::builder()
        .eq("missing", "value")
        .not()
        .build()
        .expect("expression should build");
    let metadata = Metadata::new();

    assert!(!filter(not_equal).matches(&metadata));
    assert!(!filter(negated_equal).matches(&metadata));
}
