// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Expression wire tests through a root filter.

use qubit_metadata::{FilterExpression, MetadataFilter};

#[test]
fn test_malformed_expression_nodes_are_rejected() {
    for expression in [
        serde_json::json!({"kind": "and", "children": []}),
        serde_json::json!({"kind": "or", "children": [{"kind": "all"}]}),
        serde_json::json!({"kind": "exists", "key": "k", "extra": true}),
    ] {
        let mut envelope = serde_json::to_value(MetadataFilter::all()).unwrap();
        envelope["expression"] = expression;
        assert!(serde_json::from_value::<MetadataFilter>(envelope).is_err());
    }
}

#[test]
fn test_nested_expression_round_trips_through_filter_wire() {
    let expression = FilterExpression::builder()
        .eq("status", "active")
        .or_group(|group| group.exists("tag").not())
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
