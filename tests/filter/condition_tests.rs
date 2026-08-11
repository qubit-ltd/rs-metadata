// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Predicate evaluation tests through the public expression API.

use crate::support::test_support::sample;
use qubit_datatype::DataType;
use qubit_metadata::{
    FilterExpression,
    Metadata,
    MetadataFilter,
};
use qubit_value::Value;

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
fn test_empty_membership_sets_have_defined_concrete_value_semantics() {
    let in_empty = FilterExpression::builder()
        .in_set("tag", std::iter::empty::<&str>())
        .build()
        .expect("expression should build");
    let not_in_empty = FilterExpression::builder()
        .not_in_set("tag", std::iter::empty::<&str>())
        .build()
        .expect("expression should build");
    let metadata = Metadata::new().with("tag", "rust");

    assert!(!filter(in_empty).matches(&metadata));
    assert!(filter(not_in_empty).matches(&metadata));
}

#[test]
fn test_string_range_predicates_use_lexicographic_order() {
    let expression = FilterExpression::builder()
        .gt("name", "apple")
        .lt("name", "zebra")
        .build()
        .expect("expression should build");

    assert!(filter(expression).matches(&Metadata::new().with("name", "mango")));
}

#[test]
fn test_unset_value_is_absent_for_existence_and_unknown_for_comparison() {
    let exists = FilterExpression::builder()
        .exists("value")
        .build()
        .expect("expression should build");
    let not_exists = FilterExpression::builder()
        .not_exists("value")
        .build()
        .expect("expression should build");
    let negated_equal = FilterExpression::builder()
        .eq("value", "present")
        .not()
        .build()
        .expect("expression should build");
    let metadata =
        Metadata::new().with("value", Value::Unset(DataType::String));

    assert!(!filter(exists).matches(&metadata));
    assert!(filter(not_exists).matches(&metadata));
    assert!(!filter(negated_equal).matches(&metadata));
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
fn test_incomparable_concrete_value_stays_unknown_through_not() {
    let expression = FilterExpression::builder()
        .lt("age", 18_i64)
        .not()
        .build()
        .expect("expression should build");
    let metadata = Metadata::new().with("age", "not-a-number");

    assert!(!filter(expression).matches(&metadata));
}

#[test]
fn test_not_equal_incomparable_concrete_value_fails_closed() {
    let expression = FilterExpression::builder()
        .ne("age", 18_i64)
        .build()
        .expect("expression should build");
    let metadata = Metadata::new().with("age", "not-a-number");

    assert!(!filter(expression).matches(&metadata));
}

#[test]
fn test_membership_with_incomparable_candidates_stays_unknown() {
    let expression = FilterExpression::builder()
        .not_in_set("age", [18_i64, 21_i64])
        .build()
        .expect("expression should build");
    let metadata = Metadata::new().with("age", "not-a-number");

    assert!(!filter(expression).matches(&metadata));
}

#[test]
fn test_not_equal_and_negated_equal_both_fail_closed_for_missing_value() {
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

#[test]
fn test_condition_debug_identifies_operator_without_exposing_value() {
    let comparison = FilterExpression::builder()
        .gt("score", 42_i64)
        .build()
        .expect("expression should build");
    let existence = FilterExpression::builder()
        .exists("verified")
        .build()
        .expect("expression should build");

    let comparison_debug = format!("{comparison:?}");
    let existence_debug = format!("{existence:?}");

    assert_eq!(
        comparison_debug,
        "FilterExpression { node: Condition(Condition { operator: \"greater\", key: \"score\", value: <redacted> }) }"
    );
    assert_eq!(
        existence_debug,
        "FilterExpression { node: Condition(Condition { operator: \"exists\", key: \"verified\" }) }"
    );
}
