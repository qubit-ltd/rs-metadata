// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for expression-node normalization through public views.

use qubit_metadata::{FilterExpression, FilterExpressionView};

#[test]
fn test_filter_expression_node_flattens_chained_and_children() {
    let first = FilterExpression::builder().exists("a").build().unwrap();
    let second = FilterExpression::builder().exists("b").build().unwrap();
    let third = FilterExpression::builder().exists("c").build().unwrap();
    let expression = first.try_and(second).unwrap().try_and(third).unwrap();

    assert!(
        matches!(expression.view(), FilterExpressionView::And(children) if children.len() == 3)
    );
}
