// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Expression wire tests through a root filter.

use qubit_metadata::FilterExpression;
use qubit_metadata::MetadataFilter;

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
    let filter = MetadataFilter::builder().expression(expression).build().unwrap();
    let encoded = serde_json::to_string(&filter).unwrap();
    let decoded: MetadataFilter = serde_json::from_str(&encoded).unwrap();

    assert_eq!(decoded, filter);
}

#[test]
fn test_filter_wire_round_trips_every_condition_and_boolean_operator() {
    let expressions = [
        FilterExpression::builder().eq("k", 1_i64).build(),
        FilterExpression::builder().ne("k", 1_i64).build(),
        FilterExpression::builder().lt("k", 1_i64).build(),
        FilterExpression::builder().le("k", 1_i64).build(),
        FilterExpression::builder().gt("k", 1_i64).build(),
        FilterExpression::builder().ge("k", 1_i64).build(),
        FilterExpression::builder().in_set("k", [1_i64, 2]).build(),
        FilterExpression::builder().not_in_set("k", [1_i64, 2]).build(),
        FilterExpression::builder().exists("k").build(),
        FilterExpression::builder().not_exists("k").build(),
        FilterExpression::builder().exists("k").and_group(|group| group.not_exists("j")).build(),
        FilterExpression::builder().exists("k").or_group(|group| group.not_exists("j")).build(),
        FilterExpression::builder().exists("k").not().build(),
    ];

    for expression in expressions {
        let filter = MetadataFilter::builder()
            .expression(expression.expect("expression should build"))
            .build()
            .expect("filter should build");
        let encoded = serde_json::to_string(&filter).expect("wire encoding should succeed");
        let decoded: MetadataFilter = serde_json::from_str(&encoded).expect("wire decoding should succeed");
        assert_eq!(decoded, filter);
    }
}

#[test]
fn test_filter_wire_rejects_invalid_operand_shapes_and_group_members() {
    let invalid_expressions = [
        serde_json::json!({"kind": "eq", "key": "k", "value": {"map": {}}}),
        serde_json::json!({"kind": "ne", "key": "k", "value": {"sequence": []}}),
        serde_json::json!({"kind": "lt", "key": "k"}),
        serde_json::json!({"kind": "le", "key": "k", "value": null}),
        serde_json::json!({"kind": "gt", "key": "k", "value": {}}),
        serde_json::json!({"kind": "ge", "key": "k", "value": {"scalar": null}}),
        serde_json::json!({"kind": "in", "key": "k", "values": [{"map": {}}]}),
        serde_json::json!({"kind": "not_in", "key": "k", "values": [{"sequence": []}]}),
        serde_json::json!({"kind": "exists"}),
        serde_json::json!({"kind": "not_exists", "key": 1}),
        serde_json::json!({"kind": "and", "children": [null, {"kind": "all"}]}),
        serde_json::json!({"kind": "or", "children": [{"kind": "all"}, null]}),
        serde_json::json!({"kind": "not", "expression": null}),
        serde_json::json!({"kind": "unknown"}),
    ];

    for expression in invalid_expressions {
        let mut envelope = serde_json::to_value(MetadataFilter::all()).expect("envelope should encode");
        envelope["expression"] = expression;
        assert!(
            serde_json::from_value::<MetadataFilter>(envelope).is_err(),
            "invalid expression must be rejected"
        );
    }
}
