// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the serialized condition wire representation.

use qubit_metadata::Condition;
use qubit_value::{MultiValues, Value};
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

#[test]
fn condition_wire_round_trips_finite_float_value() {
    let condition = Condition::Equal {
        key: "score".to_string(),
        value: Value::Float64(1.5),
    };

    let encoded = serde_json::to_value(&condition).unwrap();
    assert_eq!(
        encoded,
        json!({"op": "eq", "key": "score", "value": {"Float64": 1.5}}),
    );
    assert_eq!(
        serde_json::from_value::<Condition>(encoded).unwrap(),
        condition,
    );
}

#[test]
fn condition_wire_round_trips_wide_integer_values() {
    for (condition, expected) in [
        (
            Condition::Equal {
                key: "signed".to_string(),
                value: Value::Int128(i128::MIN),
            },
            json!({
                "op": "eq",
                "key": "signed",
                "value": {"Int128": i128::MIN.to_string()},
            }),
        ),
        (
            Condition::Equal {
                key: "unsigned".to_string(),
                value: Value::UInt128(u128::MAX),
            },
            json!({
                "op": "eq",
                "key": "unsigned",
                "value": {"UInt128": u128::MAX.to_string()},
            }),
        ),
    ] {
        let encoded = serde_json::to_value(&condition).unwrap();
        assert_eq!(encoded, expected);
        assert_eq!(
            serde_json::from_value::<Condition>(encoded).unwrap(),
            condition,
        );
    }
}

#[test]
fn metadata_wire_rejects_non_finite_float_values() {
    for value in [
        Value::Float32(f32::NAN),
        Value::Float64(f64::INFINITY),
        Value::Float64(f64::NEG_INFINITY),
    ] {
        let condition = Condition::Equal {
            key: "score".to_string(),
            value,
        };
        assert!(serde_json::to_value(condition).is_err());
    }

    assert!(serde_json::to_value(MultiValues::Float64(vec![1.0, f64::INFINITY,])).is_err());
}
