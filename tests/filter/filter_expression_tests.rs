// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for Boolean expression semantics.

use qubit_metadata::FilterExpression;
use qubit_metadata::MetadataFilter;

use crate::support::test_support::sample;

#[test]
fn test_and_or_and_not_are_expression_operations() {
    let active = FilterExpression::builder()
        .eq("status", "active")
        .build()
        .expect("expression should build");
    let tagged = FilterExpression::builder()
        .eq("tag", "rust")
        .build()
        .expect("expression should build");
    let expression = active
        .try_and(tagged)
        .expect("AND should build")
        .try_not()
        .expect("NOT should build");
    let filter = MetadataFilter::builder()
        .expression(expression)
        .build()
        .expect("filter should build");

    assert!(!filter.matches(&sample()));
}
