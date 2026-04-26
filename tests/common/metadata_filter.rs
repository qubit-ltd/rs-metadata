/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Unit tests for [`qubit_metadata::MetadataFilter`] DSL composition.

use qubit_metadata::{
    FilterMatchOptions, Metadata, MetadataFilter, MissingKeyPolicy, NumberComparisonPolicy,
};
use serde_json::json;

fn sample() -> Metadata {
    let mut m = Metadata::new();
    m.set("status", "active");
    m.set("score", 42_i64);
    m.set("ratio", 0.75_f64);
    m.set("verified", true);
    m.set("tag", "rust");
    m
}

#[test]
fn and_predicates_all_match() {
    let f = MetadataFilter::builder()
        .eq("status", "active")
        .and_ge("score", 10_i64)
        .and_exists("verified");
    assert!(f.build().matches(&sample()));
}

#[test]
fn and_predicates_one_fails() {
    let f = MetadataFilter::builder()
        .eq("status", "active")
        .and_gt("score", 100_i64);
    assert!(!f.build().matches(&sample()));
}

#[test]
fn or_predicates_one_matches() {
    let f = MetadataFilter::builder()
        .eq("status", "inactive")
        .or_eq("status", "active");
    assert!(f.build().matches(&sample()));
}

#[test]
fn or_predicates_all_fail() {
    let f = MetadataFilter::builder()
        .eq("status", "inactive")
        .or_eq("status", "pending");
    assert!(!f.build().matches(&sample()));
}

#[test]
fn not_inverts_expression_result() {
    let yes = MetadataFilter::builder().eq("status", "active").not();
    let no = MetadataFilter::builder().eq("status", "inactive").not();
    assert!(!yes.build().matches(&sample()));
    assert!(no.build().matches(&sample()));
}

#[test]
fn empty_filter_matches_anything() {
    let f = MetadataFilter::builder().build();
    assert!(f.matches(&sample()));
    assert!(f.matches(&Metadata::new()));
}

#[test]
fn negated_empty_filter_matches_nothing() {
    let f = MetadataFilter::builder().not();
    assert!(!f.clone().build().matches(&sample()));
    assert!(!f.build().matches(&Metadata::new()));
}

#[test]
fn group_composition_works() {
    // status == active AND (score >= 80 OR tag == rust)
    let f = MetadataFilter::builder()
        .eq("status", "active")
        .and(|g| g.ge("score", 80_i64).or_eq("tag", "rust"));
    assert!(f.build().matches(&sample()));
}

#[test]
fn negated_group_composition_works() {
    // status == active AND NOT (score >= 80 OR tag == java)
    let f = MetadataFilter::builder()
        .eq("status", "active")
        .and_not(|g| g.ge("score", 80_i64).or_eq("tag", "java"));
    assert!(f.build().matches(&sample()));
}

#[test]
fn missing_key_policy_can_be_configured_on_filter() {
    let f = MetadataFilter::builder()
        .ne("missing", "x")
        .missing_key_policy(MissingKeyPolicy::NoMatch);
    assert!(!f.build().matches(&sample()));
}

#[test]
fn number_comparison_policy_can_be_configured_on_filter() {
    let mut m = Metadata::new();
    m.set("n", 9_007_199_254_740_993_i64);

    let conservative = MetadataFilter::builder().gt("n", 0.5_f64);
    assert!(!conservative.clone().build().matches(&m));

    let approximate = conservative
        .clone()
        .number_comparison_policy(NumberComparisonPolicy::Approximate);
    assert!(approximate.build().matches(&m));
}

#[test]
fn options_round_trip_works() {
    let options = FilterMatchOptions {
        missing_key_policy: MissingKeyPolicy::NoMatch,
        number_comparison_policy: NumberComparisonPolicy::Approximate,
    };
    let f = MetadataFilter::builder()
        .eq("status", "active")
        .with_options(options)
        .build();
    assert_eq!(f.options(), options);
}

#[test]
fn filter_serde_round_trip() {
    let f = MetadataFilter::builder()
        .eq("status", "active")
        .and_ge("score", 10_i64)
        .or_not(|g| g.exists("tag").and_eq("tag", "java"))
        .missing_key_policy(MissingKeyPolicy::NoMatch)
        .number_comparison_policy(NumberComparisonPolicy::Approximate)
        .build();

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
        .build();

    let json = serde_json::to_value(&f).unwrap();
    assert_eq!(
        json,
        json!({
            "version": 1,
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
                "missing_key_policy": "Match",
                "number_comparison_policy": "Conservative"
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
        .build();

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
fn filter_serde_encodes_match_all_and_match_none() {
    assert_eq!(
        serde_json::to_value(MetadataFilter::all()).unwrap(),
        json!({
            "version": 1,
            "options": {
                "missing_key_policy": "Match",
                "number_comparison_policy": "Conservative"
            }
        })
    );
    assert_eq!(
        serde_json::to_value(MetadataFilter::none()).unwrap(),
        json!({
            "version": 1,
            "expr": {
                "type": "false"
            },
            "options": {
                "missing_key_policy": "Match",
                "number_comparison_policy": "Conservative"
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
fn filter_deserialize_accepts_legacy_private_expr_format() {
    let f: MetadataFilter = serde_json::from_value(json!({
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
            "missing_key_policy": "NoMatch",
            "number_comparison_policy": "Conservative"
        }
    }))
    .unwrap();

    assert!(f.matches(&sample()));

    let json = serde_json::to_string(&f).unwrap();
    assert!(json.contains("\"version\":1"));
    assert!(json.contains("\"type\":\"or\""));
    assert!(!json.contains("\"Or\""));
    assert!(!json.contains("\"GreaterEqual\""));
}

#[test]
fn filter_deserialize_rejects_unsupported_wire_version() {
    let error = serde_json::from_value::<MetadataFilter>(json!({
        "version": 2
    }))
    .unwrap_err()
    .to_string();

    assert!(error.contains("unsupported MetadataFilter wire format version 2"));
}

#[test]
fn filter_constructors_and_option_setters_work() {
    let options = FilterMatchOptions {
        missing_key_policy: MissingKeyPolicy::NoMatch,
        number_comparison_policy: NumberComparisonPolicy::Approximate,
    };

    assert!(MetadataFilter::all().matches(&sample()));
    assert!(!MetadataFilter::none().matches(&sample()));
    assert!((!MetadataFilter::none()).matches(&sample()));

    let strict = MetadataFilter::builder()
        .ne("missing", "x")
        .build()
        .with_missing_key_policy(MissingKeyPolicy::NoMatch);
    assert!(!strict.matches(&sample()));

    let approximate = MetadataFilter::builder()
        .gt("score", 0.5_f64)
        .build()
        .with_number_comparison_policy(NumberComparisonPolicy::Approximate)
        .with_options(options);
    assert_eq!(approximate.options(), options);
}

#[test]
fn or_operator_methods_cover_each_predicate() {
    let meta = sample();

    assert!(
        MetadataFilter::builder()
            .eq("status", "inactive")
            .or_ne("status", "inactive")
            .build()
            .matches(&meta)
    );
    assert!(
        MetadataFilter::builder()
            .eq("status", "inactive")
            .or_lt("score", 50_i64)
            .build()
            .matches(&meta)
    );
    assert!(
        MetadataFilter::builder()
            .eq("status", "inactive")
            .or_le("score", 42_i64)
            .build()
            .matches(&meta)
    );
    assert!(
        MetadataFilter::builder()
            .eq("status", "inactive")
            .or_gt("score", 40_i64)
            .build()
            .matches(&meta)
    );
    assert!(
        MetadataFilter::builder()
            .eq("status", "inactive")
            .or_ge("score", 42_i64)
            .build()
            .matches(&meta)
    );
    assert!(
        MetadataFilter::builder()
            .eq("status", "inactive")
            .or_in_set("status", ["active", "pending"])
            .build()
            .matches(&meta)
    );
    assert!(
        MetadataFilter::builder()
            .eq("status", "inactive")
            .or_not_in_set("status", ["pending"])
            .build()
            .matches(&meta)
    );
    assert!(
        MetadataFilter::builder()
            .eq("status", "inactive")
            .or_exists("verified")
            .build()
            .matches(&meta)
    );
    assert!(
        MetadataFilter::builder()
            .eq("status", "inactive")
            .or_not_exists("missing")
            .build()
            .matches(&meta)
    );
}

#[test]
fn builder_aliases_and_empty_groups_preserve_expected_identities() {
    let meta = sample();

    assert!(
        MetadataFilter::builder()
            .not_in_set("status", ["inactive"])
            .build()
            .matches(&meta)
    );
    assert!(MetadataFilter::builder().or(|g| g).build().matches(&meta));
    assert!(
        MetadataFilter::builder()
            .eq("status", "active")
            .and(|g| g)
            .build()
            .matches(&meta)
    );
    assert!(
        MetadataFilter::builder()
            .or(|g| g.eq("status", "active"))
            .build()
            .matches(&meta)
    );
    assert!(
        MetadataFilter::builder()
            .not()
            .or_eq("status", "active")
            .build()
            .matches(&meta)
    );
    assert!(
        MetadataFilter::builder()
            .eq("status", "active")
            .or_not(|g| g)
            .build()
            .matches(&meta)
    );
    assert!(
        !MetadataFilter::builder()
            .eq("status", "active")
            .and_not(|g| g)
            .build()
            .matches(&meta)
    );
}

#[test]
fn chained_or_expressions_are_flattened() {
    let filter = MetadataFilter::builder()
        .eq("status", "inactive")
        .or_eq("tag", "java")
        .or_eq("status", "active")
        .build();

    assert!(filter.matches(&sample()));
}
