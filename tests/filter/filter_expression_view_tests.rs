// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::FilterExpressionView`].

use qubit_metadata::{FilterExpressionView, MetadataFilter};

#[test]
fn test_expression_view_preserves_boolean_structure() {
    let filter = MetadataFilter::builder()
        .eq("a", 1_i64)
        .or_not(|group| group.exists("b"))
        .build()
        .expect("filter should build");

    match filter.expression().view() {
        FilterExpressionView::Or(children) => {
            assert_eq!(children.len(), 2);
            assert!(matches!(
                children[0].view(),
                FilterExpressionView::Condition(_)
            ));
            assert!(matches!(children[1].view(), FilterExpressionView::Not(_)));
        }
        other => panic!("expected OR expression, got {other:?}"),
    }
}

#[test]
fn test_expression_view_exposes_constant_expressions() {
    assert!(matches!(
        MetadataFilter::all().expression().view(),
        FilterExpressionView::True
    ));
    assert!(matches!(
        MetadataFilter::none().expression().view(),
        FilterExpressionView::False
    ));
}
