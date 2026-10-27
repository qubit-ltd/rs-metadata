// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for Boolean expression semantics.

use qubit_datatype::DataType;
use qubit_metadata::FilterExpression;
use qubit_metadata::FilterExpressionView;
use qubit_metadata::FilterLimitKind;
use qubit_metadata::FilterLimits;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataError;
use qubit_metadata::MetadataFilter;
use qubit_value::Value;

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
fn test_and_or_follow_three_valued_logic_for_missing_and_unset_values() {
    // Each row is (left, right, AND, OR); None represents Unknown.
    let cases = [
        (Some(true), Some(true), Some(true), Some(true)),
        (Some(true), Some(false), Some(false), Some(true)),
        (Some(true), None, None, Some(true)),
        (Some(false), Some(true), Some(false), Some(true)),
        (Some(false), Some(false), Some(false), Some(false)),
        (Some(false), None, Some(false), None),
        (None, Some(true), None, Some(true)),
        (None, Some(false), Some(false), None),
        (None, None, None, None),
    ];

    for unset_unknown in [false, true] {
        let unknown_mode = if unset_unknown { "unset" } else { "missing" };
        for (left_value, right_value, expected_and, expected_or) in cases {
            let mut metadata = Metadata::new();
            if let Some(value) = left_value {
                metadata = metadata.with("left", value);
            } else if unset_unknown {
                metadata = metadata.with("left", Value::Unset(DataType::Bool));
            }
            if let Some(value) = right_value {
                metadata = metadata.with("right", value);
            } else if unset_unknown {
                metadata = metadata.with("right", Value::Unset(DataType::Bool));
            }

            let left = FilterExpression::builder()
                .eq("left", true)
                .build()
                .expect("left predicate should build");
            let right = FilterExpression::builder()
                .eq("right", true)
                .build()
                .expect("right predicate should build");

            for (operator, expected, expression) in [
                (
                    "AND",
                    expected_and,
                    left.clone().try_and(right.clone()).expect("AND should build"),
                ),
                (
                    "OR",
                    expected_or,
                    left.clone().try_or(right.clone()).expect("OR should build"),
                ),
            ] {
                let filter = MetadataFilter::builder()
                    .expression(expression.clone())
                    .build()
                    .expect("combined filter should build");
                assert_eq!(
                    filter.matches(&metadata),
                    expected == Some(true),
                    "left={left_value:?}, right={right_value:?}, operator={operator}, unknown_mode={unknown_mode}"
                );

                let negated = expression.try_not().expect("NOT should build");
                let negated_filter = MetadataFilter::builder()
                    .expression(negated)
                    .build()
                    .expect("negated filter should build");
                assert_eq!(
                    negated_filter.matches(&metadata),
                    expected == Some(false),
                    "left={left_value:?}, right={right_value:?}, operator=NOT {operator}, unknown_mode={unknown_mode}"
                );
            }
        }
    }
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
