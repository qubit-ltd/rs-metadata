// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for filter-expression composition.

use crate::support::test_support::sample;
use qubit_metadata::MetadataFilter;

#[test]
fn test_filter_expression_and_requires_all_children_to_match() {
    let matching = MetadataFilter::builder()
        .eq("status", "active")
        .and_ge("score", 40_i64)
        .build()
        .expect("AND filter should build");
    let non_matching = MetadataFilter::builder()
        .eq("status", "active")
        .and_ge("score", 100_i64)
        .build()
        .expect("AND filter should build");

    assert!(matching.matches(&sample()));
    assert!(!non_matching.matches(&sample()));
}

#[test]
fn test_filter_expression_or_matches_when_any_child_matches() {
    let filter = MetadataFilter::builder()
        .eq("status", "inactive")
        .or_ge("score", 40_i64)
        .build()
        .expect("OR filter should build");

    assert!(filter.matches(&sample()));
}

#[test]
fn test_filter_expression_not_negates_nested_expression() {
    let filter = MetadataFilter::builder()
        .eq("status", "inactive")
        .not()
        .build()
        .expect("NOT filter should build");

    assert!(filter.matches(&sample()));
    assert!(MetadataFilter::none().not().matches(&sample()));
    assert!(!MetadataFilter::all().not().matches(&sample()));
}
