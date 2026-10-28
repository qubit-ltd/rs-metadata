// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Metadata-filter wire tests.

use qubit_metadata::FilterExpression;
use qubit_metadata::FilterLimits;
use qubit_metadata::MetadataFilter;
#[cfg(feature = "json")]
use qubit_metadata::MetadataLimits;
use serde_json::Deserializer;
use serde_json::Error;
use serde_json::from_str;
use serde_json::from_value;
use serde_json::json;
use serde_json::to_string;
use serde_json::to_value;

/// Builds a filter containing one existence condition.
fn filter_for_existing_key(key: &str) -> MetadataFilter {
    let expression = FilterExpression::builder()
        .exists(key)
        .build()
        .expect("expression should build");
    MetadataFilter::builder()
        .expression(expression)
        .build()
        .expect("filter should build")
}

/// Deserializes a filter under explicit receiver limits.
fn deserialize_with_filter_limits(encoded: &str, limits: FilterLimits) -> Result<MetadataFilter, Error> {
    let mut deserializer = Deserializer::from_str(encoded);
    MetadataFilter::deserialize_with_filter_limits(&mut deserializer, limits)
}

#[test]
fn test_receiver_accepts_small_expression_with_wider_sender_limits() {
    let encoded = to_string(&filter_for_existing_key("status")).unwrap();
    let receiver = FilterLimits::builder()
        .max_depth(1)
        .max_nodes(1)
        .max_set_values(1)
        .max_key_bytes(6)
        .build()
        .unwrap();

    let decoded =
        deserialize_with_filter_limits(&encoded, receiver).expect("actual expression should fit receiver limits");
    assert_eq!(decoded.limits(), receiver);
}

#[test]
fn test_receiver_rejects_expression_exceeding_its_key_limit() {
    let encoded = to_string(&filter_for_existing_key("status")).unwrap();
    let receiver = FilterLimits::builder().max_key_bytes(5).build().unwrap();

    let error = deserialize_with_filter_limits(&encoded, receiver).unwrap_err();
    assert!(error.to_string().contains("KeyBytes value 6 exceeds the maximum of 5"));
}

#[test]
fn test_filter_wire_rejects_sender_limits_field() {
    let encoded = json!({
        "version": 1,
        "expression": {"kind": "exists", "key": "id"},
        "options": {"numeric_comparison_policy": "exact"},
        "limits": {"max_key_bytes": 5}
    });

    let error = from_value::<MetadataFilter>(encoded).expect_err("V1 filters must reject sender-supplied limits");
    let message = error.to_string();
    assert!(message.contains("unknown field"), "unexpected error: {message}");
    assert!(message.contains("limits"), "missing field context: {message}");
}

#[test]
fn test_receiver_accepts_expression_at_exact_boundary() {
    let encoded = to_string(&filter_for_existing_key("status")).unwrap();
    let receiver = FilterLimits::builder().max_key_bytes(6).build().unwrap();

    assert!(
        deserialize_with_filter_limits(&encoded, receiver).is_ok(),
        "receiver must accept an expression at its exact key limit"
    );
}

#[test]
fn test_receiver_rejects_raw_nodes_elided_by_normalization() {
    let encoded = r#"
        {
            "version": 1,
            "expression": {
                "kind": "or",
                "children": [
                    { "kind": "all" },
                    { "kind": "all" }
                ]
            },
            "options": { "numeric_comparison_policy": "exact" }
        }
    "#;
    let receiver = FilterLimits::builder().max_nodes(1).build().unwrap();

    let error = deserialize_with_filter_limits(encoded, receiver).unwrap_err();
    assert!(error.to_string().contains("Nodes value 2 exceeds the maximum of 1"));
}

#[test]
fn test_metadata_filter_wire_contains_expression() {
    let expression = FilterExpression::builder().exists("status").build().unwrap();
    let filter = MetadataFilter::builder().expression(expression).build().unwrap();
    let encoded = to_value(filter).unwrap();

    assert_eq!(encoded["version"], json!(1));
    assert_eq!(encoded["expression"]["kind"], json!("exists"));
    assert!(encoded.get("limits").is_none());
}

#[test]
fn test_generic_deserializer_reports_unsupported_version_as_serde_error() {
    let encoded = json!({
        "version": 7,
        "expression": {"kind": "all"},
        "options": {"numeric_comparison_policy": "exact"}
    });

    let error = from_value::<MetadataFilter>(encoded).expect_err("unsupported version");
    assert!(
        error
            .to_string()
            .contains("unsupported MetadataFilter wire format version 7; expected 1")
    );
}

#[test]
fn test_metadata_filter_wire_rejects_duplicate_envelope_fields() {
    for input in [
        r#"{"version":1,"version":1,"expression":{"kind":"all"},"options":{"numeric_comparison_policy":"exact"}}"#,
        r#"{"version":1,"expression":{"kind":"all"},"expression":{"kind":"none"},"options":{"numeric_comparison_policy":"exact"}}"#,
        r#"{"version":1,"expression":{"kind":"all"},"options":{"numeric_comparison_policy":"exact"},"options":{"numeric_comparison_policy":"exact"}}"#,
    ] {
        assert!(
            from_str::<MetadataFilter>(input).is_err(),
            "duplicate envelope fields must be rejected: {input}"
        );
    }
}

#[test]
fn test_metadata_filter_wire_reports_missing_and_unknown_envelope_fields() {
    for input in [
        r#"{}"#,
        r#"{"version":1}"#,
        r#"{"version":1,"expression":{"kind":"all"}}"#,
        r#"{"version":1,"expression":{"kind":"all"},"options":{"numeric_comparison_policy":"exact"},"extra":true}"#,
    ] {
        assert!(
            from_str::<MetadataFilter>(input).is_err(),
            "missing or unknown envelope fields must be rejected: {input}"
        );
    }
}

#[test]
fn test_expression_wire_rejects_duplicate_and_conflicting_fields() {
    for input in [
        r#"{"version":1,"expression":{"kind":"eq","kind":"eq","key":"k","value":{"scalar":{"string":"a"}}},"options":{"numeric_comparison_policy":"exact"}}"#,
        r#"{"version":1,"expression":{"kind":"eq","key":"k","key":"k","value":{"scalar":{"string":"a"}}},"options":{"numeric_comparison_policy":"exact"}}"#,
        r#"{"version":1,"expression":{"kind":"exists","key":"k","value":{"scalar":{"string":"a"}}},"options":{"numeric_comparison_policy":"exact"}}"#,
        r#"{"version":1,"expression":{"kind":"in","key":"k","values":[{"scalar":{"string":"a"}}],"values":[{"scalar":{"string":"b"}}]},"options":{"numeric_comparison_policy":"exact"}}"#,
        r#"{"version":1,"expression":{"kind":"and","children":[{"kind":"all"},{"kind":"none"}],"children":[{"kind":"all"},{"kind":"none"}]},"options":{"numeric_comparison_policy":"exact"}}"#,
        r#"{"version":1,"expression":{"kind":"eq","key":"k","value":{"scalar":{"string":"a"}},"value":{"scalar":{"string":"b"}}},"options":{"numeric_comparison_policy":"exact"}}"#,
        r#"{"version":1,"expression":{"kind":"not","expression":{"kind":"all"},"expression":{"kind":"none"}},"options":{"numeric_comparison_policy":"exact"}}"#,
        r#"{"version":1,"expression":{"kind":"all","key":"extra"},"options":{"numeric_comparison_policy":"exact"}}"#,
        r#"{"version":1,"expression":{"kind":"eq","key":"k"},"options":{"numeric_comparison_policy":"exact"}}"#,
        r#"{"version":1,"expression":{"kind":"and","children":[{"kind":"all"},{"kind":"none"}],"key":"extra"},"options":{"numeric_comparison_policy":"exact"}}"#,
        r#"{"version":1,"expression":{"kind":"none","value":{"scalar":{"string":"a"}}},"options":{"numeric_comparison_policy":"exact"}}"#,
        r#"{"version":1,"expression":{"kind":"ne","key":"k","value":{"scalar":{"string":"a"}},"values":[{"scalar":{"string":"b"}}]},"options":{"numeric_comparison_policy":"exact"}}"#,
        r#"{"version":1,"expression":{"kind":"lt","key":"k","value":{"scalar":{"string":"a"}},"children":[{"kind":"all"},{"kind":"none"}]},"options":{"numeric_comparison_policy":"exact"}}"#,
        r#"{"version":1,"expression":{"kind":"in","key":"k","values":[{"scalar":{"string":"a"}}],"value":{"scalar":{"string":"b"}}},"options":{"numeric_comparison_policy":"exact"}}"#,
        r#"{"version":1,"expression":{"kind":"not_in","key":"k","values":[{"scalar":{"string":"a"}}],"expression":{"kind":"all"}},"options":{"numeric_comparison_policy":"exact"}}"#,
        r#"{"version":1,"expression":{"kind":"exists","key":"k","children":[{"kind":"all"},{"kind":"none"}]},"options":{"numeric_comparison_policy":"exact"}}"#,
        r#"{"version":1,"expression":{"kind":"or","children":[{"kind":"all"},{"kind":"none"}],"expression":{"kind":"all"}},"options":{"numeric_comparison_policy":"exact"}}"#,
        r#"{"version":1,"expression":{},"options":{"numeric_comparison_policy":"exact"}}"#,
    ] {
        assert!(
            from_str::<MetadataFilter>(input).is_err(),
            "invalid expression wire must be rejected: {input}"
        );
    }
}

#[test]
fn test_expression_wire_rejects_non_object_expression_nodes() {
    let input = r#"{"version":1,"expression":"not-an-object","options":{"numeric_comparison_policy":"exact"}}"#;
    assert!(
        from_str::<MetadataFilter>(input).is_err(),
        "a non-object expression node must be rejected"
    );
}

#[test]
fn test_metadata_filter_wire_rejects_non_object_envelope() {
    assert!(
        from_str::<MetadataFilter>("[]").is_err(),
        "a non-object metadata-filter envelope must be rejected"
    );
}

#[test]
fn test_expression_wire_rejects_non_array_membership_and_child_sequences() {
    for input in [
        r#"{"version":1,"expression":{"kind":"in","key":"k","values":"not-an-array"},"options":{"numeric_comparison_policy":"exact"}}"#,
        r#"{"version":1,"expression":{"kind":"and","children":"not-an-array"},"options":{"numeric_comparison_policy":"exact"}}"#,
    ] {
        assert!(
            from_str::<MetadataFilter>(input).is_err(),
            "non-array membership values or child sequences must be rejected: {input}"
        );
    }
}

#[cfg(feature = "json")]
#[test]
fn test_filter_fuzz_positive_seeds_round_trip() {
    let receiver_limits = FilterLimits::builder()
        .max_depth(4)
        .max_nodes(16)
        .max_set_values(8)
        .max_key_bytes(32)
        .build()
        .expect("fuzz receiver limits should be valid");
    let cases: [(&str, &[u8]); 2] = [
        (
            "minimal",
            include_bytes!("../../fixtures/decode_metadata_filter_minimal.json"),
        ),
        (
            "nested",
            include_bytes!("../../fixtures/decode_metadata_filter_nested.json"),
        ),
    ];
    for (name, input) in cases {
        let value = MetadataFilter::decode_json_slice_with_limits(input, MetadataLimits::default(), receiver_limits)
            .unwrap_or_else(|error| panic!("{name} seed must decode: {error}"));
        let encoded = to_string(&value).expect("seed filter must serialize");
        let restored = MetadataFilter::decode_json_slice_with_limits(
            encoded.as_bytes(),
            MetadataLimits::default(),
            receiver_limits,
        )
        .expect("serialized seed must decode");
        assert_eq!(restored, value, "{name} seed must round trip");
    }
}
