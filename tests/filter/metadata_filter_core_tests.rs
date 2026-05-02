/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Unit tests for [`qubit_metadata::MetadataFilter`] (match semantics and builder DSL).
use crate::test_support::sample;
use qubit_metadata::{
    FilterMatchOptions, Metadata, MetadataError, MetadataFilter, MissingKeyPolicy,
    NumberComparisonPolicy,
};

#[test]
fn and_predicates_all_match() {
    let f = MetadataFilter::builder()
        .eq("status", "active")
        .and_ge("score", 10_i64)
        .and_exists("verified");
    assert!(f.build().unwrap().matches(&sample()));
}

#[test]
fn and_predicates_one_fails() {
    let f = MetadataFilter::builder()
        .eq("status", "active")
        .and_gt("score", 100_i64);
    assert!(!f.build().unwrap().matches(&sample()));
}

#[test]
fn or_predicates_one_matches() {
    let f = MetadataFilter::builder()
        .eq("status", "inactive")
        .or_eq("status", "active");
    assert!(f.build().unwrap().matches(&sample()));
}

#[test]
fn or_predicates_all_fail() {
    let f = MetadataFilter::builder()
        .eq("status", "inactive")
        .or_eq("status", "pending");
    assert!(!f.build().unwrap().matches(&sample()));
}

#[test]
fn not_inverts_expression_result() {
    let yes = MetadataFilter::builder().eq("status", "active").not();
    let no = MetadataFilter::builder().eq("status", "inactive").not();
    assert!(!yes.build().unwrap().matches(&sample()));
    assert!(no.build().unwrap().matches(&sample()));
}

#[test]
fn empty_filter_matches_anything() {
    let f = MetadataFilter::builder().build().unwrap();
    assert!(f.matches(&sample()));
    assert!(f.matches(&Metadata::new()));
}

#[test]
fn negated_empty_filter_matches_nothing() {
    let f = MetadataFilter::builder().not();
    assert!(!f.clone().build().unwrap().matches(&sample()));
    assert!(!f.build().unwrap().matches(&Metadata::new()));
}

#[test]
fn group_composition_works() {
    // status == active AND (score >= 80 OR tag == rust)
    let f = MetadataFilter::builder()
        .eq("status", "active")
        .and(|g| g.ge("score", 80_i64).or_eq("tag", "rust"));
    assert!(f.build().unwrap().matches(&sample()));
}

#[test]
fn negated_group_composition_works() {
    // status == active AND NOT (score >= 80 OR tag == java)
    let f = MetadataFilter::builder()
        .eq("status", "active")
        .and_not(|g| g.ge("score", 80_i64).or_eq("tag", "java"));
    assert!(f.build().unwrap().matches(&sample()));
}

#[test]
fn missing_key_policy_can_be_configured_on_filter() {
    let f = MetadataFilter::builder()
        .ne("missing", "x")
        .missing_key_policy(MissingKeyPolicy::NoMatch);
    assert!(!f.build().unwrap().matches(&sample()));
}

#[test]
fn number_comparison_policy_can_be_configured_on_filter() {
    let mut m = Metadata::new();
    m.set("n", 9_007_199_254_740_993_i64);

    let conservative = MetadataFilter::builder().gt("n", 0.5_f64);
    assert!(!conservative.clone().build().unwrap().matches(&m));

    let approximate = conservative
        .clone()
        .number_comparison_policy(NumberComparisonPolicy::Approximate);
    assert!(approximate.build().unwrap().matches(&m));
}

#[test]
fn options_round_trip_works() {
    let options = FilterMatchOptions {
        missing_key_policy: MissingKeyPolicy::NoMatch,
        number_comparison_policy: NumberComparisonPolicy::Approximate,
    };
    let f = MetadataFilter::builder()
        .eq("status", "active")
        .with_options(options)
        .build()
        .unwrap();
    assert_eq!(f.options(), options);
}

#[test]
fn filter_constructors_and_option_setters_work() {
    let options = FilterMatchOptions {
        missing_key_policy: MissingKeyPolicy::NoMatch,
        number_comparison_policy: NumberComparisonPolicy::Approximate,
    };

    assert!(MetadataFilter::all().matches(&sample()));
    assert!(!MetadataFilter::none().matches(&sample()));
    assert!((!MetadataFilter::none()).matches(&sample()));

    let strict = MetadataFilter::builder()
        .ne("missing", "x")
        .build()
        .unwrap()
        .with_missing_key_policy(MissingKeyPolicy::NoMatch);
    assert!(!strict.matches(&sample()));

    let approximate = MetadataFilter::builder()
        .gt("score", 0.5_f64)
        .build()
        .unwrap()
        .with_number_comparison_policy(NumberComparisonPolicy::Approximate)
        .with_options(options);
    assert_eq!(approximate.options(), options);
}

#[test]
fn or_operator_methods_cover_each_predicate() {
    let meta = sample();

    assert!(
        MetadataFilter::builder()
            .eq("status", "inactive")
            .or_ne("status", "inactive")
            .build()
            .unwrap()
            .matches(&meta)
    );
    assert!(
        MetadataFilter::builder()
            .eq("status", "inactive")
            .or_lt("score", 50_i64)
            .build()
            .unwrap()
            .matches(&meta)
    );
    assert!(
        MetadataFilter::builder()
            .eq("status", "inactive")
            .or_le("score", 42_i64)
            .build()
            .unwrap()
            .matches(&meta)
    );
    assert!(
        MetadataFilter::builder()
            .eq("status", "inactive")
            .or_gt("score", 40_i64)
            .build()
            .unwrap()
            .matches(&meta)
    );
    assert!(
        MetadataFilter::builder()
            .eq("status", "inactive")
            .or_ge("score", 42_i64)
            .build()
            .unwrap()
            .matches(&meta)
    );
    assert!(
        MetadataFilter::builder()
            .eq("status", "inactive")
            .or_in_set("status", ["active", "pending"])
            .build()
            .unwrap()
            .matches(&meta)
    );
    assert!(
        MetadataFilter::builder()
            .eq("status", "inactive")
            .or_not_in_set("status", ["pending"])
            .build()
            .unwrap()
            .matches(&meta)
    );
    assert!(
        MetadataFilter::builder()
            .eq("status", "inactive")
            .or_exists("verified")
            .build()
            .unwrap()
            .matches(&meta)
    );
    assert!(
        MetadataFilter::builder()
            .eq("status", "inactive")
            .or_not_exists("missing")
            .build()
            .unwrap()
            .matches(&meta)
    );
}

#[test]
fn builder_aliases_preserve_expected_identities() {
    let meta = sample();

    assert!(
        MetadataFilter::builder()
            .not_in_set("status", ["inactive"])
            .build()
            .unwrap()
            .matches(&meta)
    );
    assert!(
        MetadataFilter::builder()
            .or(|g| g.eq("status", "active"))
            .build()
            .unwrap()
            .matches(&meta)
    );
    assert!(
        MetadataFilter::builder()
            .not()
            .or_eq("status", "active")
            .build()
            .unwrap()
            .matches(&meta)
    );
}

#[test]
fn empty_group_build_returns_error() {
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
fn chained_or_expressions_are_flattened() {
    let filter = MetadataFilter::builder()
        .eq("status", "inactive")
        .or_eq("tag", "java")
        .or_eq("status", "active")
        .build()
        .unwrap();

    assert!(filter.matches(&sample()));
}
