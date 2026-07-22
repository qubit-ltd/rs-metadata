// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Serde and wire format tests for [`qubit_metadata::MetadataFilter`].
use crate::support::test_support::sample;
use qubit_datatype::NumericComparisonPolicy;
use qubit_metadata::{FilterLimits, MetadataFilter};
use serde_json::json;

#[test]
fn test_filter_serde_round_trip() {
    let f = MetadataFilter::builder()
        .eq("status", "active")
        .and_ge("score", 10_i64)
        .or_not(|g| g.exists("tag").and_eq("tag", "java"))
        .numeric_comparison_policy(NumericComparisonPolicy::Approximate)
        .build()
        .unwrap();

    let json = serde_json::to_string(&f).unwrap();
    let restored: MetadataFilter = serde_json::from_str(&json).unwrap();
    assert_eq!(f, restored);
    assert_eq!(f.matches(&sample()), restored.matches(&sample()));
}

#[test]
fn test_filter_serde_uses_versioned_wire_format() {
    let f = MetadataFilter::builder()
        .eq("status", "active")
        .and_ge("score", 10_i64)
        .build()
        .unwrap();

    let json = serde_json::to_value(&f).unwrap();
    assert_eq!(
        json,
        json!({
            "version": 3,
            "expression": {
                "type": "and",
                "children": [
                    {
                        "type": "condition",
                        "condition": {
                            "op": "eq",
                            "key": "status",
                            "value": {
                                "version": 1,
                                "value": {"scalar": {"string": "active"}}
                            }
                        }
                    },
                    {
                        "type": "condition",
                        "condition": {
                            "op": "ge",
                            "key": "score",
                            "value": {
                                "version": 1,
                                "value": {"scalar": {"int64": 10}}
                            }
                        }
                    }
                ]
            },
            "options": {
                "numeric_comparison_policy": "exact"
            }
        })
    );
}

#[test]
fn test_filter_serde_round_trips_all_condition_ops() {
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
fn test_filter_options_serde_uses_snake_case_and_rejects_pascal_case() {
    let filter = MetadataFilter::builder()
        .eq("status", "active")
        .numeric_comparison_policy(NumericComparisonPolicy::Approximate)
        .build()
        .unwrap();

    let json = serde_json::to_value(&filter).unwrap();
    assert_eq!(
        json["options"],
        json!({
            "numeric_comparison_policy": "approximate"
        })
    );

    let error = serde_json::from_value::<MetadataFilter>(json!({
        "version": 3,
        "expression": {
            "type": "condition",
            "condition": {
                "op": "exists",
                "key": "status"
            }
        },
        "options": {
            "numeric_comparison_policy": "Approximate"
        }
    }))
    .unwrap_err()
    .to_string();

    assert!(error.contains("unknown variant"));
}

#[test]
fn test_filter_options_deserialize_rejects_unknown_fields() {
    let error = serde_json::from_value::<MetadataFilter>(json!({
        "version": 3,
        "expression": { "type": "true" },
        "options": {
            "numeric_comparison_policy": "exact",
            "unexpected_policy": true
        }
    }))
    .expect_err("unknown filter option should be rejected")
    .to_string();

    assert!(error.contains("unknown field `unexpected_policy`"));
}

#[test]
fn test_filter_serde_encodes_match_all_and_match_none() {
    assert_eq!(
        serde_json::to_value(MetadataFilter::all()).unwrap(),
        json!({
            "version": 3,
            "expression": {
                "type": "true"
            },
            "options": {
                "numeric_comparison_policy": "exact"
            }
        })
    );
    assert_eq!(
        serde_json::to_value(MetadataFilter::none()).unwrap(),
        json!({
            "version": 3,
            "expression": {
                "type": "false"
            },
            "options": {
                "numeric_comparison_policy": "exact"
            }
        })
    );
}

#[test]
fn test_filter_deserialize_rejects_missing_wire_version() {
    let error = serde_json::from_value::<MetadataFilter>(json!({
        "expression": {
            "type": "condition",
            "condition": {
                "op": "exists",
                "key": "status"
            }
        }
    }))
    .expect_err("missing wire version should be rejected")
    .to_string();

    assert!(error.contains("missing field `version`"));
}

#[test]
fn test_filter_deserialize_rejects_legacy_private_expr_format() {
    let error = serde_json::from_value::<MetadataFilter>(json!({
        "version": 3,
        "expression": {
            "Or": [
                {
                    "And": [
                        {
                            "Condition": {
                                "Equal": {
                                    "key": "status",
                                    "value": {
                                        "version": 1,
                                        "value": {"scalar": {"string": "active"}}
                                    }
                                }
                            }
                        },
                        {
                            "Condition": {
                                "GreaterEqual": {
                                    "key": "score",
                                    "value": {
                                        "version": 1,
                                        "value": {"scalar": {"int64": 10}}
                                    }
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
        }
    }))
    .unwrap_err()
    .to_string();

    assert!(error.contains("missing field"));
}

#[test]
fn test_filter_deserialize_rejects_expression_exceeding_depth_limit() {
    let mut expression = json!({ "type": "true" });
    for _ in 0..=FilterLimits::default().max_depth() {
        expression = json!({
            "type": "not",
            "expression": expression,
        });
    }

    let error = serde_json::from_value::<MetadataFilter>(json!({
        "version": 3,
        "expression": expression,
    }))
    .expect_err("overly deep filter wire payload must be rejected")
    .to_string();

    assert!(error.contains("filter depth"));
}

#[test]
fn test_filter_deserialize_rejects_unsupported_wire_version() {
    let error = serde_json::from_value::<MetadataFilter>(json!({
        "version": 4,
        "expression": { "type": "true" }
    }))
    .unwrap_err()
    .to_string();

    assert!(error.contains("unsupported MetadataFilter wire format version 4"));
}

#[test]
fn test_filter_deserialize_rejects_v1_wire_version() {
    let error = serde_json::from_value::<MetadataFilter>(json!({
        "version": 1,
        "expression": { "type": "true" }
    }))
    .unwrap_err()
    .to_string();

    assert!(error.contains("unsupported MetadataFilter wire format version 1"));
}

#[test]
fn test_filter_deserialize_rejects_v2_wire_version() {
    let error = serde_json::from_value::<MetadataFilter>(json!({
        "version": 2,
        "expression": { "type": "true" }
    }))
    .unwrap_err()
    .to_string();

    assert!(error.contains("unsupported MetadataFilter wire format version 2"));
}

#[test]
fn test_filter_deserialize_rejects_missing_expression() {
    let error = serde_json::from_value::<MetadataFilter>(json!({
        "version": 3
    }))
    .unwrap_err()
    .to_string();

    assert!(error.contains("missing field `expression`"));
}

#[test]
fn test_filter_deserialize_rejects_empty_wire_group() {
    let error = serde_json::from_value::<MetadataFilter>(json!({
        "version": 3,
        "expression": {
            "type": "and",
            "children": []
        }
    }))
    .unwrap_err()
    .to_string();

    assert!(error.contains("empty 'and' filter group is not allowed"));
}
