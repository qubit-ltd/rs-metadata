// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Metadata-filter wire tests.

use qubit_metadata::{FilterExpression, FilterLimits, MetadataFilter};
use serde_json::json;

#[test]
fn test_metadata_filter_wire_contains_expression() {
    let expression = FilterExpression::builder()
        .exists("status")
        .build()
        .unwrap();
    let filter = MetadataFilter::builder()
        .expression(expression)
        .build()
        .unwrap();
    let encoded = serde_json::to_value(filter).unwrap();

    assert_eq!(encoded["version"], json!(4));
    assert_eq!(encoded["expression"]["kind"], json!("exists"));
    assert_eq!(
        encoded["limits"]["max_depth"],
        json!(FilterLimits::MAX.max_depth())
    );
}
