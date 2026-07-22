// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::FilterExpressionView`].

use qubit_metadata::{FilterExpression, FilterExpressionView};

#[test]
fn test_expression_view_preserves_boolean_structure() {
    let expression = FilterExpression::builder()
        .eq("status", "active")
        .or_group(|group| group.eq("tag", "rust").exists("owner"))
        .build()
        .expect("expression should build");

    assert!(matches!(expression.view(), FilterExpressionView::Or(_)));
}
