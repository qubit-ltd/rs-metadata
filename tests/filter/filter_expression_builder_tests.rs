// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::FilterExpressionBuilder`].

use crate::support::test_support::sample;
use qubit_metadata::{FilterExpression, FilterExpressionView, MetadataFilter};

#[test]
fn builder_creates_nested_boolean_expression() {
    let expression = FilterExpression::builder()
        .eq("status", "active")
        .gt("score", 40_i64)
        .or_group(|group| group.eq("tag", "rust").exists("owner"))
        .build()
        .expect("expression should build");
    let filter = MetadataFilter::builder()
        .expression(expression)
        .build()
        .expect("filter should build");

    assert!(filter.matches(&sample()));
    assert!(matches!(
        filter.expression().view(),
        FilterExpressionView::Or(_)
    ));
}

#[test]
fn builder_rejects_empty_groups() {
    let result = FilterExpression::builder().and_group(|group| group).build();

    assert!(result.is_err());
}

#[test]
fn expression_owns_boolean_composition() {
    let active = FilterExpression::builder()
        .eq("status", "active")
        .build()
        .expect("expression should build");
    let high_score = FilterExpression::builder()
        .gt("score", 40_i64)
        .build()
        .expect("expression should build");
    let expression = active
        .try_and(high_score)
        .expect("AND composition should fit hard limits")
        .try_not()
        .expect("NOT composition should fit hard limits");

    assert!(matches!(
        FilterExpression::match_all().view(),
        FilterExpressionView::True
    ));
    assert!(matches!(
        FilterExpression::match_none().view(),
        FilterExpressionView::False
    ));
    assert!(matches!(expression.view(), FilterExpressionView::Not(_)));
}
