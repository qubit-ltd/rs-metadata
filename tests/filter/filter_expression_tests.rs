// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for Boolean expression semantics.

use qubit_metadata::FilterExpression;
use qubit_metadata::FilterExpressionView;
use qubit_metadata::FilterLimitKind;
use qubit_metadata::FilterLimits;
use qubit_metadata::MetadataError;
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

#[test]
fn test_alternating_groups_preserve_depth_limit_after_cached_validation() {
    let mut expression = FilterExpression::builder()
        .exists("key")
        .build()
        .expect("initial expression should build");
    for _ in 1..FilterLimits::MAX.max_depth() {
        let leaf = FilterExpression::builder()
            .exists("next")
            .build()
            .expect("leaf expression should build");
        expression = if matches!(expression.view(), FilterExpressionView::And(_)) {
            expression.try_or(leaf)
        } else {
            expression.try_and(leaf)
        }
        .expect("expression at the exact depth limit should build");
    }

    let leaf = FilterExpression::builder()
        .exists("overflow")
        .build()
        .expect("overflow leaf expression should build");
    let result = if matches!(expression.view(), FilterExpressionView::And(_)) {
        expression.try_or(leaf)
    } else {
        expression.try_and(leaf)
    };

    assert_eq!(
        result,
        Err(MetadataError::FilterLimitExceeded {
            kind: FilterLimitKind::Depth,
            value: FilterLimits::MAX.max_depth() + 1,
            maximum: FilterLimits::MAX.max_depth(),
        })
    );
}

#[test]
fn test_full_validation_reports_earlier_key_error_before_later_node_error() {
    let expression = FilterExpression::builder()
        .exists("too-long")
        .exists("ok")
        .build()
        .expect("expression should fit hard limits");
    let limits = FilterLimits::builder()
        .max_nodes(2)
        .max_key_bytes(1)
        .build()
        .expect("custom limits should build");

    let result = MetadataFilter::builder().expression(expression).limits(limits).build();

    assert_eq!(
        result,
        Err(MetadataError::FilterLimitExceeded {
            kind: FilterLimitKind::KeyBytes,
            value: "too-long".len(),
            maximum: 1,
        })
    );
}

#[test]
fn test_full_validation_reports_earlier_key_error_before_later_depth_error() {
    let first = FilterExpression::builder()
        .exists("too-long")
        .build()
        .expect("first expression should build");
    let later_nested = FilterExpression::builder()
        .exists("a")
        .exists("b")
        .build()
        .expect("nested expression should build");
    let expression = first
        .try_or(later_nested)
        .expect("combined expression should fit hard limits");
    let limits = FilterLimits::builder()
        .max_depth(2)
        .max_key_bytes(1)
        .build()
        .expect("custom limits should build");

    let result = MetadataFilter::builder().expression(expression).limits(limits).build();

    assert_eq!(
        result,
        Err(MetadataError::FilterLimitExceeded {
            kind: FilterLimitKind::KeyBytes,
            value: "too-long".len(),
            maximum: 1,
        })
    );
}
