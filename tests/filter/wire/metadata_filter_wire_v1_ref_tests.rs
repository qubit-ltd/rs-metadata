// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Borrowed metadata-filter wire tests.

use qubit_metadata::{FilterExpression, MetadataFilter};

#[test]
fn test_metadata_filter_serialization_keeps_wire_version_and_expression() {
    let expression = FilterExpression::builder()
        .exists("owner")
        .build()
        .expect("expression should build");
    let filter = MetadataFilter::builder()
        .expression(expression)
        .build()
        .expect("filter should build");

    let encoded = serde_json::to_value(&filter).expect("filter should serialize");

    assert_eq!(encoded["version"], 1);
    assert_eq!(encoded["expression"]["kind"], "exists");
    assert_eq!(encoded["expression"]["key"], "owner");
}
