// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Serde and wire format tests for [`qubit_metadata::MetadataFilter`].
use crate::support::test_support::sample;
use qubit_metadata::{
    MetadataFilter,
    MissingKeyPolicy,
    NumberComparisonPolicy,
};
use serde_json::json;

#[test]
fn filter_serde_round_trip() {
    let f = MetadataFilter::builder()
        .eq("status", "active")
        .and_ge("score", 10_i64)
        .or_not(|g| g.exists("tag").and_eq("tag", "java"))
        .missing_key_policy(MissingKeyPolicy::NoMatch)
        .number_comparison_policy(NumberComparisonPolicy::Approximate)
        .build()
        .unwrap();

    let json = serde_json::to_string(&f).unwrap();
    let restored: MetadataFilter = serde_json::from_str(&json).unwrap();
    assert_eq!(f, restored);
    assert_eq!(f.matches(&sample()), restored.matches(&sample()));
}

#[test]
fn filter_serde_uses_versioned_wire_format() {
    let f = MetadataFilter::builder()
        .eq("status", "active")
        .and_ge("score", 10_i64)
        .build()
        .unwrap();

    let json = serde_json::to_value(&f).unwrap();
    assert_eq!(
        json,
        json!({
            "version": 2,
            "expr": {
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
            "options": {
                "missing_key_policy": "match",
                "number_comparison_policy": "conservative"
            }
        })
    );
}

#[test]
fn filter_serde_round_trips_all_condition_ops() {
    let f = MetadataFilter::builder()
        .eq("status", "active")
        .and_ne("status", "inactive")
        .and_lt("score", 100_i64)
        .and_le("score", 42_i64)
        .and_gt("score", 10_i64)
        .and_ge("score", 42_i64)
        .and_in_set("tag", ["rust", "go"])
        .and_not_in_set("status", ["archived"])
        .and_exists("verified")
        .and_not_exists("missing")
        .build()
        .unwrap();

    let json = serde_json::to_string(&f).unwrap();
    for op in [
        "\"op\":\"eq\"",
        "\"op\":\"ne\"",
        "\"op\":\"lt\"",
        "\"op\":\"le\"",
        "\"op\":\"gt\"",
        "\"op\":\"ge\"",
        "\"op\":\"in\"",
        "\"op\":\"not_in\"",
        "\"op\":\"exists\"",
        "\"op\":\"not_exists\"",
    ] {
        assert!(json.contains(op), "missing {op} in {json}");
    }

    let restored: MetadataFilter = serde_json::from_str(&json).unwrap();
    assert_eq!(f, restored);
    assert!(restored.matches(&sample()));
}

#[test]
fn filter_options_serde_uses_snake_case_and_rejects_pascal_case() {
    let filter = MetadataFilter::builder()
        .eq("status", "active")
        .missing_key_policy(MissingKeyPolicy::NoMatch)
        .number_comparison_policy(NumberComparisonPolicy::Approximate)
        .build()
        .unwrap();

    let json = serde_json::to_value(&filter).unwrap();
    assert_eq!(
        json["options"],
        json!({
            "missing_key_policy": "no_match",
            "number_comparison_policy": "approximate"
        })
    );

    let error = serde_json::from_value::<MetadataFilter>(json!({
        "version": 2,
        "expr": {
            "type": "condition",
            "condition": {
                "op": "exists",
                "key": "status"
            }
        },
        "options": {
            "missing_key_policy": "NoMatch",
            "number_comparison_policy": "Approximate"
        }
    }))
    .unwrap_err()
    .to_string();

    assert!(error.contains("unknown variant"));
}

#[test]
fn filter_serde_encodes_match_all_and_match_none() {
    assert_eq!(
        serde_json::to_value(MetadataFilter::all()).unwrap(),
        json!({
            "version": 2,
            "options": {
                "missing_key_policy": "match",
                "number_comparison_policy": "conservative"
            }
        })
    );
    assert_eq!(
        serde_json::to_value(MetadataFilter::none()).unwrap(),
        json!({
            "version": 2,
            "expr": {
                "type": "false"
            },
            "options": {
                "missing_key_policy": "match",
                "number_comparison_policy": "conservative"
            }
        })
    );
}

#[test]
fn filter_deserialize_accepts_missing_wire_version_as_current() {
    let f: MetadataFilter = serde_json::from_value(json!({
        "expr": {
            "type": "condition",
            "condition": {
                "op": "exists",
                "key": "status"
            }
        }
    }))
    .unwrap();

    assert!(f.matches(&sample()));
}

#[test]
fn filter_deserialize_rejects_legacy_private_expr_format() {
    let error = serde_json::from_value::<MetadataFilter>(json!({
        "expr": {
            "Or": [
                {
                    "And": [
                        {
                            "Condition": {
                                "Equal": {
                                    "key": "status",
                                    "value": { "String": "active" }
                                }
                            }
                        },
                        {
                            "Condition": {
                                "GreaterEqual": {
                                    "key": "score",
                                    "value": { "Int64": 10 }
                                }
                            }
                        }
                    ]
                },
                {
                    "Not": {
                        "Condition": {
                            "Exists": {
                                "key": "missing"
                            }
                        }
                    }
                },
                "False"
            ]
        },
        "options": {
            "missing_key_policy": "no_match",
            "number_comparison_policy": "conservative"
        }
    }))
    .unwrap_err()
    .to_string();

    assert!(error.contains("missing field"));
}

#[test]
fn filter_deserialize_rejects_unsupported_wire_version() {
    let error = serde_json::from_value::<MetadataFilter>(json!({
        "version": 3
    }))
    .unwrap_err()
    .to_string();

    assert!(error.contains("unsupported MetadataFilter wire format version 3"));
}

#[test]
fn filter_deserialize_rejects_v1_wire_version() {
    let error = serde_json::from_value::<MetadataFilter>(json!({
        "version": 1
    }))
    .unwrap_err()
    .to_string();

    assert!(error.contains("unsupported MetadataFilter wire format version 1"));
}

#[test]
fn filter_deserialize_rejects_empty_wire_group() {
    let error = serde_json::from_value::<MetadataFilter>(json!({
        "version": 2,
        "expr": {
            "type": "and",
            "children": []
        }
    }))
    .unwrap_err()
    .to_string();

    assert!(error.contains("empty 'and' filter group is not allowed"));
}
