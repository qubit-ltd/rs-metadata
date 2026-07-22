// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Unit tests for [`qubit_metadata::MetadataFilter`] (match semantics and
//! builder DSL).
use crate::support::test_support::sample;
use qubit_datatype::NumericComparisonPolicy;
use qubit_metadata::{FilterLimits, FilterMatchOptions, Metadata, MetadataError, MetadataFilter};

#[test]
fn test_and_predicates_all_match() {
    let f = MetadataFilter::builder()
        .eq("status", "active")
        .and_ge("score", 10_i64)
        .and_exists("verified");
    assert!(f.build().unwrap().matches(&sample()));
}

#[test]
fn test_and_predicates_one_fails() {
    let f = MetadataFilter::builder()
        .eq("status", "active")
        .and_gt("score", 100_i64);
    assert!(!f.build().unwrap().matches(&sample()));
}

#[test]
fn test_or_predicates_one_matches() {
    let f = MetadataFilter::builder()
        .eq("status", "inactive")
        .or_eq("status", "active");
    assert!(f.build().unwrap().matches(&sample()));
}

#[test]
fn test_or_predicates_all_fail() {
    let f = MetadataFilter::builder()
        .eq("status", "inactive")
        .or_eq("status", "pending");
    assert!(!f.build().unwrap().matches(&sample()));
}

#[test]
fn test_not_inverts_expression_result() {
    let yes = MetadataFilter::builder().eq("status", "active").not();
    let no = MetadataFilter::builder().eq("status", "inactive").not();
    assert!(!yes.build().unwrap().matches(&sample()));
    assert!(no.build().unwrap().matches(&sample()));
}

#[test]
fn test_empty_filter_matches_anything() {
    let f = MetadataFilter::builder().build().unwrap();
    assert!(f.matches(&sample()));
    assert!(f.matches(&Metadata::new()));
}

#[test]
fn test_negated_empty_filter_matches_nothing() {
    let f = MetadataFilter::builder().not();
    assert!(!f.clone().build().unwrap().matches(&sample()));
    assert!(!f.build().unwrap().matches(&Metadata::new()));
}

#[test]
fn test_group_composition_works() {
    // status == active AND (score >= 80 OR tag == rust)
    let f = MetadataFilter::builder()
        .eq("status", "active")
        .and(|g| g.ge("score", 80_i64).or_eq("tag", "rust"));
    assert!(f.build().unwrap().matches(&sample()));
}

#[test]
fn test_negated_group_composition_works() {
    // status == active AND NOT (score >= 80 OR tag == java)
    let f = MetadataFilter::builder()
        .eq("status", "active")
        .and_not(|g| g.ge("score", 80_i64).or_eq("tag", "java"));
    assert!(f.build().unwrap().matches(&sample()));
}

#[test]
fn test_missing_values_remain_unknown_through_group_negation() {
    let metadata = Metadata::new();
    let and_not = MetadataFilter::builder()
        .and_not(|group| group.eq("missing", "value"))
        .build()
        .unwrap();
    let or_not = MetadataFilter::builder()
        .eq("also_missing", "value")
        .or_not(|group| group.in_set("missing", ["value"]))
        .build()
        .unwrap();

    assert!(!and_not.matches(&metadata));
    assert!(!or_not.matches(&metadata));
}

#[test]
fn test_numeric_comparison_policy_can_be_configured_on_filter() {
    let mut m = Metadata::new();
    m.set("n", 9_007_199_254_740_993_i64);

    let exact = MetadataFilter::builder().eq("n", 9_007_199_254_740_992_f64);
    assert!(!exact.clone().build().unwrap().matches(&m));

    let approximate = exact
        .clone()
        .numeric_comparison_policy(NumericComparisonPolicy::Approximate);
    assert!(approximate.build().unwrap().matches(&m));
}

#[test]
fn test_options_round_trip_works() {
    let options = FilterMatchOptions::new()
        .with_numeric_comparison_policy(NumericComparisonPolicy::Approximate);
    let f = MetadataFilter::builder()
        .eq("status", "active")
        .with_options(options)
        .build()
        .unwrap();
    assert_eq!(f.options(), options);
}

#[test]
fn test_filter_constructors_and_option_setters_work() {
    let options = FilterMatchOptions::new()
        .with_numeric_comparison_policy(NumericComparisonPolicy::Approximate);

    assert!(MetadataFilter::all().matches(&sample()));
    assert!(!MetadataFilter::none().matches(&sample()));
    assert!((!MetadataFilter::none()).matches(&sample()));

    let missing = MetadataFilter::builder()
        .ne("missing", "x")
        .build()
        .unwrap();
    assert!(!missing.matches(&sample()));

    let approximate = MetadataFilter::builder()
        .gt("score", 0.5_f64)
        .build()
        .unwrap()
        .with_numeric_comparison_policy(NumericComparisonPolicy::Approximate)
        .with_options(options);
    assert_eq!(approximate.options(), options);
}

#[test]
fn test_try_and_and_try_or_combine_filters_with_matching_options() {
    let active = MetadataFilter::builder()
        .eq("status", "active")
        .build()
        .expect("active filter should build");
    let score = MetadataFilter::builder()
        .ge("score", 40_i64)
        .build()
        .expect("score filter should build");

    assert!(active
        .clone()
        .try_and(score.clone())
        .unwrap()
        .matches(&sample()));
    assert!(active.try_or(score).unwrap().matches(&sample()));
}

#[test]
fn test_try_and_rejects_filters_with_different_options() {
    let exact = MetadataFilter::builder()
        .eq("score", 42_i64)
        .build()
        .expect("exact filter should build");
    let approximate = MetadataFilter::builder()
        .eq("score", 42_i64)
        .numeric_comparison_policy(NumericComparisonPolicy::Approximate)
        .build()
        .expect("approximate filter should build");

    assert!(matches!(
        exact.try_and(approximate),
        Err(MetadataError::IncompatibleFilterOptions { .. })
    ));
}

#[test]
fn test_validate_limits_rejects_excessive_key_length_and_node_count() {
    let key_limit = FilterLimits::new(8, 8, 8, 3);
    let key_error = MetadataFilter::builder()
        .exists("long")
        .build()
        .expect("filter should build")
        .validate_limits(key_limit)
        .expect_err("key exceeding a custom limit must be rejected");
    assert!(matches!(
        key_error,
        MetadataError::FilterLimitExceeded {
            limit: "key length",
            maximum: 3,
        }
    ));

    let node_limit = FilterLimits::new(8, 2, 8, 8);
    let node_error = MetadataFilter::builder()
        .exists("first")
        .and_exists("second")
        .build()
        .expect("filter should build")
        .validate_limits(node_limit)
        .expect_err("expression exceeding a custom node limit must be rejected");
    assert!(matches!(
        node_error,
        MetadataError::FilterLimitExceeded {
            limit: "node count",
            maximum: 2,
        }
    ));
}

#[test]
fn test_or_operator_methods_cover_each_predicate() {
    let meta = sample();

    assert!(MetadataFilter::builder()
        .eq("status", "inactive")
        .or_ne("status", "inactive")
        .build()
        .unwrap()
        .matches(&meta));
    assert!(MetadataFilter::builder()
        .eq("status", "inactive")
        .or_lt("score", 50_i64)
        .build()
        .unwrap()
        .matches(&meta));
    assert!(MetadataFilter::builder()
        .eq("status", "inactive")
        .or_le("score", 42_i64)
        .build()
        .unwrap()
        .matches(&meta));
    assert!(MetadataFilter::builder()
        .eq("status", "inactive")
        .or_gt("score", 40_i64)
        .build()
        .unwrap()
        .matches(&meta));
    assert!(MetadataFilter::builder()
        .eq("status", "inactive")
        .or_ge("score", 42_i64)
        .build()
        .unwrap()
        .matches(&meta));
    assert!(MetadataFilter::builder()
        .eq("status", "inactive")
        .or_in_set("status", ["active", "pending"])
        .build()
        .unwrap()
        .matches(&meta));
    assert!(MetadataFilter::builder()
        .eq("status", "inactive")
        .or_not_in_set("status", ["pending"])
        .build()
        .unwrap()
        .matches(&meta));
    assert!(MetadataFilter::builder()
        .eq("status", "inactive")
        .or_exists("verified")
        .build()
        .unwrap()
        .matches(&meta));
    assert!(MetadataFilter::builder()
        .eq("status", "inactive")
        .or_not_exists("missing")
        .build()
        .unwrap()
        .matches(&meta));
}

#[test]
fn test_builder_aliases_preserve_expected_identities() {
    let meta = sample();

    assert!(MetadataFilter::builder()
        .not_in_set("status", ["inactive"])
        .build()
        .unwrap()
        .matches(&meta));
    assert!(MetadataFilter::builder()
        .or(|g| g.eq("status", "active"))
        .build()
        .unwrap()
        .matches(&meta));
    assert!(MetadataFilter::builder()
        .not()
        .or_eq("status", "active")
        .build()
        .unwrap()
        .matches(&meta));
}

#[test]
fn test_empty_group_build_returns_error() {
    for (operator, error) in [
        (
            "and",
            MetadataFilter::builder().and(|g| g).build().unwrap_err(),
        ),
        (
            "or",
            MetadataFilter::builder().or(|g| g).build().unwrap_err(),
        ),
        (
            "and_not",
            MetadataFilter::builder()
                .and_not(|g| g)
                .build()
                .unwrap_err(),
        ),
        (
            "or_not",
            MetadataFilter::builder().or_not(|g| g).build().unwrap_err(),
        ),
        (
            "and",
            MetadataFilter::builder()
                .and(|g| g.not())
                .build()
                .unwrap_err(),
        ),
    ] {
        match error {
            MetadataError::InvalidFilterExpression { message } => {
                assert!(message.contains(&format!("empty '{operator}'")));
            }
            other => panic!("expected InvalidFilterExpression, got {other:?}"),
        }
    }
}

#[test]
fn test_chained_or_expressions_are_flattened() {
    let filter = MetadataFilter::builder()
        .eq("status", "inactive")
        .or_eq("tag", "java")
        .or_eq("status", "active")
        .build()
        .unwrap();

    assert!(filter.matches(&sample()));
}
