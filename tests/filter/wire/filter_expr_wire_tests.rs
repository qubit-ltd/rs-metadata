/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for the serialized filter expression wire representation.

use crate::test_support::sample;
use qubit_metadata::MetadataFilter;
use serde_json::json;

#[test]
fn filter_expr_wire_serializes_nested_expression_tree() {
    let filter = MetadataFilter::builder()
        .eq("status", "active")
        .and_ge("score", 10_i64)
        .or_not(|group| group.exists("tag").and_eq("tag", "java"))
        .build()
        .expect("nested filter should build");

    let encoded = serde_json::to_value(&filter).expect("filter should serialize");
    assert_eq!(
        encoded["expr"],
        json!({
            "type": "or",
            "children": [
                {
                    "type": "and",
                    "children": [
                        {
                            "type": "condition",
                            "condition": {
                                "op": "eq",
                                "key": "status",
                                "value": { "String": "active" }
                            }
                        },
                        {
                            "type": "condition",
                            "condition": {
                                "op": "ge",
                                "key": "score",
                                "value": { "Int64": 10 }
                            }
                        }
                    ]
                },
                {
                    "type": "not",
                    "expr": {
                        "type": "and",
                        "children": [
                            {
                                "type": "condition",
                                "condition": {
                                    "op": "exists",
                                    "key": "tag"
                                }
                            },
                            {
                                "type": "condition",
                                "condition": {
                                    "op": "eq",
                                    "key": "tag",
                                    "value": { "String": "java" }
                                }
                            }
                        ]
                    }
                }
            ]
        })
    );

    let decoded: MetadataFilter =
        serde_json::from_value(encoded).expect("filter should deserialize");
    assert_eq!(decoded, filter);
    assert!(decoded.matches(&sample()));
}

#[test]
fn filter_expr_wire_rejects_empty_groups() {
    let error = serde_json::from_value::<MetadataFilter>(json!({
        "version": 1,
        "expr": {
            "type": "or",
            "children": []
        }
    }))
    .expect_err("empty OR group should be rejected")
    .to_string();

    assert!(error.contains("empty 'or' filter group is not allowed"));
}
