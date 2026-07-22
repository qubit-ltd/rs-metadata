// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Metadata-filter serde tests.

use qubit_metadata::{FilterExpression, MetadataFilter};

#[test]
fn test_metadata_filter_serde_round_trip() {
    let expression = FilterExpression::builder()
        .exists("status")
        .build()
        .unwrap();
    let filter = MetadataFilter::builder()
        .expression(expression)
        .build()
        .unwrap();
    let encoded = serde_json::to_string(&filter).unwrap();
    let decoded: MetadataFilter = serde_json::from_str(&encoded).unwrap();

    assert_eq!(decoded, filter);
}
