// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Behavioral tests for private filter-expression nodes.

use qubit_metadata::{FilterExpressionView, MetadataFilter};

#[test]
fn test_filter_expression_node_flattens_chained_and_children() {
    let filter = MetadataFilter::builder()
        .exists("first")
        .and_exists("second")
        .and_exists("third")
        .build()
        .expect("filter should build");

    assert!(matches!(
        filter.expression().view(),
        FilterExpressionView::And(children) if children.len() == 3
    ));
}
