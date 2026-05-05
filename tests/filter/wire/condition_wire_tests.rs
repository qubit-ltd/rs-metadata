/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for the serialized condition wire representation.

use qubit_metadata::Condition;
use qubit_value::Value;
use serde_json::json;

#[test]
fn condition_wire_serializes_all_operator_tags() {
    let conditions = [
        (
            Condition::Equal {
                key: "status".to_string(),
                value: Value::String("active".to_string()),
            },
            "eq",
        ),
        (
            Condition::NotEqual {
                key: "status".to_string(),
                value: Value::String("inactive".to_string()),
            },
            "ne",
        ),
        (
            Condition::Less {
                key: "score".to_string(),
                value: Value::Int64(100),
            },
            "lt",
        ),
        (
            Condition::LessEqual {
                key: "score".to_string(),
                value: Value::Int64(42),
            },
            "le",
        ),
        (
            Condition::Greater {
                key: "score".to_string(),
                value: Value::Int64(10),
            },
            "gt",
        ),
        (
            Condition::GreaterEqual {
                key: "score".to_string(),
                value: Value::Int64(42),
            },
            "ge",
        ),
        (
            Condition::In {
                key: "tag".to_string(),
                values: vec![Value::String("rust".to_string())],
            },
            "in",
        ),
        (
            Condition::NotIn {
                key: "tag".to_string(),
                values: vec![Value::String("java".to_string())],
            },
            "not_in",
        ),
        (
            Condition::Exists {
                key: "status".to_string(),
            },
            "exists",
        ),
        (
            Condition::NotExists {
                key: "missing".to_string(),
            },
            "not_exists",
        ),
    ];

    for (condition, op) in conditions {
        let encoded = serde_json::to_value(&condition).expect("condition should serialize");
        assert_eq!(encoded.get("op"), Some(&json!(op)));
        let decoded: Condition =
            serde_json::from_value(encoded).expect("condition should deserialize");
        assert_eq!(decoded, condition);
    }
}
