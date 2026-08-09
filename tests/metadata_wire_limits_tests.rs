// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the bounded public JSON decoding APIs.

#![cfg(all(feature = "json", feature = "schema"))]

use qubit_budget::ResourceBudget;
use qubit_metadata::{
    FilterLimitKind,
    FilterLimits,
    Metadata,
    MetadataFilter,
    MetadataSchema,
    MetadataWireDecodeError,
    MetadataWireLimits,
};
use qubit_value::{
    ValueWireDecodeError,
    WireLimits,
};

/// Wraps one expression in a valid metadata-filter V1 envelope.
fn filter_input(expression: &str) -> Vec<u8> {
    format!(
        r#"{{
            "version":1,
            "expression":{expression},
            "options":{{"numeric_comparison_policy":"exact"}}
        }}"#,
    )
    .into_bytes()
}

#[test]
fn test_resource_budget_preserves_filter_limit_facts() {
    let mut budget = ResourceBudget::new(FilterLimitKind::Nodes, 1);

    budget
        .try_consume(1)
        .expect("first node should fit the budget");
    let error = budget
        .try_consume(1)
        .expect_err("second node should exceed the budget");

    assert_eq!(error.kind(), &FilterLimitKind::Nodes);
    assert_eq!(error.maximum(), 1);
    assert_eq!(error.observed_at_least(), 2);
}

#[test]
fn test_filter_decode_accepts_exact_node_boundary() {
    let input = filter_input(r#"{"kind":"not","expression":{"kind":"all"}}"#);
    let limits = FilterLimits::builder().max_nodes(2).build().unwrap();

    assert!(
        MetadataFilter::decode_json_slice_with_limits(
            &input,
            MetadataWireLimits::new(input.len()),
            limits,
        )
        .is_ok()
    );
}

#[test]
fn test_filter_decode_accepts_exact_depth_boundary() {
    let input = filter_input(r#"{"kind":"not","expression":{"kind":"all"}}"#);
    let limits = FilterLimits::builder().max_depth(2).build().unwrap();

    assert!(
        MetadataFilter::decode_json_slice_with_limits(
            &input,
            MetadataWireLimits::new(input.len()),
            limits,
        )
        .is_ok()
    );
}

#[test]
fn test_filter_decode_accepts_exact_set_value_boundary() {
    let input = filter_input(
        r#"{"kind":"in","key":"k","values":[{"scalar":{"int64":1}},{"scalar":{"int64":2}}]}"#,
    );
    let limits = FilterLimits::builder().max_set_values(2).build().unwrap();

    assert!(
        MetadataFilter::decode_json_slice_with_limits(
            &input,
            MetadataWireLimits::new(input.len()),
            limits,
        )
        .is_ok()
    );
}

#[test]
fn test_filter_decode_accepts_exact_key_byte_boundary() {
    let input = filter_input(
        r#"{"kind":"eq","key":"long","value":{"scalar":{"int64":1}}}"#,
    );
    let limits = FilterLimits::builder().max_key_bytes(4).build().unwrap();

    assert!(
        MetadataFilter::decode_json_slice_with_limits(
            &input,
            MetadataWireLimits::new(input.len()),
            limits,
        )
        .is_ok()
    );
}

#[test]
fn test_filter_decode_stops_before_overdepth_node_body() {
    let input = filter_input(
        r#"{"kind":"not","expression":{"kind":"exists","key":42}}"#,
    );
    let limits = FilterLimits::builder().max_depth(1).build().unwrap();

    let error = MetadataFilter::decode_json_slice_with_limits(
        &input,
        MetadataWireLimits::new(input.len()),
        limits,
    )
    .expect_err("depth limit should stop nested-node decoding");

    assert!(
        error
            .to_string()
            .contains("Metadata filter Depth value 2 exceeds the maximum of 1")
    );
}

#[test]
fn test_filter_decode_stops_before_overbudget_node_body() {
    let input = filter_input(
        r#"{"kind":"and","children":[{"kind":"exists","key":42}]}"#,
    );
    let limits = FilterLimits::builder().max_nodes(1).build().unwrap();

    let error = MetadataFilter::decode_json_slice_with_limits(
        &input,
        MetadataWireLimits::new(input.len()),
        limits,
    )
    .expect_err("node limit should stop child-node decoding");

    assert!(
        error
            .to_string()
            .contains("Metadata filter Nodes value 2 exceeds the maximum of 1")
    );
}

#[test]
fn test_filter_decode_stops_before_oversized_membership_value() {
    let input = filter_input(
        r#"{"kind":"in","key":"k","values":[{"scalar":{"int64":1}},{"invalid":true}]}"#,
    );
    let limits = FilterLimits::builder().max_set_values(1).build().unwrap();

    let error = MetadataFilter::decode_json_slice_with_limits(
        &input,
        MetadataWireLimits::new(input.len()),
        limits,
    )
    .expect_err("set limit should stop the second value from decoding");

    assert!(error.to_string().contains(
        "Metadata filter SetValues value 2 exceeds the maximum of 1"
    ));
}

#[test]
fn test_filter_decode_stops_after_reading_oversized_key() {
    let input =
        filter_input(r#"{"kind":"eq","key":"long","value":{"invalid":true}}"#);
    let limits = FilterLimits::builder().max_key_bytes(1).build().unwrap();

    let error = MetadataFilter::decode_json_slice_with_limits(
        &input,
        MetadataWireLimits::new(input.len()),
        limits,
    )
    .expect_err("key limit should stop value decoding");

    assert!(
        error.to_string().contains(
            "Metadata filter KeyBytes value 4 exceeds the maximum of 1"
        )
    );
}

#[test]
fn test_filter_decode_shares_wire_node_budget_across_siblings() {
    let input = filter_input(
        r#"{"kind":"and","children":[{"kind":"all"},{"kind":"exists","key":42}]}"#,
    );
    let wire = WireLimits::new(input.len()).with_max_nodes(2);
    let limits = MetadataWireLimits::new(input.len()).with_wire(wire);

    let error = MetadataFilter::decode_json_slice_with_limits(
        &input,
        limits,
        FilterLimits::MAX,
    )
    .expect_err("shared wire node budget should stop the second child");

    assert!(
        error
            .to_string()
            .contains("wire input Nodes value 3 exceeds the limit of 2")
    );
}

#[test]
fn test_bounded_filter_decoder_accepts_every_expression_variant() {
    for expression in [
        r#"{"kind":"all"}"#,
        r#"{"kind":"none"}"#,
        r#"{"kind":"eq","key":"k","value":{"scalar":{"int64":1}}}"#,
        r#"{"kind":"ne","key":"k","value":{"scalar":{"int64":1}}}"#,
        r#"{"kind":"lt","key":"k","value":{"scalar":{"int64":1}}}"#,
        r#"{"kind":"le","key":"k","value":{"scalar":{"int64":1}}}"#,
        r#"{"kind":"gt","key":"k","value":{"scalar":{"int64":1}}}"#,
        r#"{"kind":"ge","key":"k","value":{"scalar":{"int64":1}}}"#,
        r#"{"kind":"in","key":"k","values":[{"scalar":{"int64":1}}]}"#,
        r#"{"kind":"not_in","key":"k","values":[{"scalar":{"int64":1}}]}"#,
        r#"{"kind":"exists","key":"k"}"#,
        r#"{"kind":"not_exists","key":"k"}"#,
        r#"{"kind":"and","children":[{"kind":"all"},{"kind":"none"}]}"#,
        r#"{"kind":"or","children":[{"kind":"all"},{"kind":"none"}]}"#,
        r#"{"kind":"not","expression":{"kind":"all"}}"#,
    ] {
        let input = filter_input(expression);
        MetadataFilter::decode_json_slice(&input).unwrap_or_else(|error| {
            panic!("expression should decode: {expression}: {error}")
        });
    }
}

#[test]
fn test_bounded_filter_decoder_accepts_expression_fields_in_any_order() {
    let encoded = r#"{
        "options":{"numeric_comparison_policy":"exact"},
        "version":1,
        "expression":{"kind":"exists","key":"status"}
    }"#;

    assert!(MetadataFilter::decode_json_slice(encoded.as_bytes()).is_ok());
}

#[test]
fn test_decode_json_slice_with_limits_rejects_oversized_input_before_parsing() {
    let limits = MetadataWireLimits::new(4);

    assert!(matches!(
        Metadata::decode_json_slice_with_limits(b"null!", limits),
        Err(MetadataWireDecodeError::InputTooLarge {
            input_bytes: 5,
            max_input_bytes: 4,
        })
    ));
    assert!(matches!(
        MetadataSchema::decode_json_slice_with_limits(b"null!", limits),
        Err(MetadataWireDecodeError::InputTooLarge {
            input_bytes: 5,
            max_input_bytes: 4,
        })
    ));

    assert!(matches!(
        MetadataFilter::decode_json_slice_with_limits(
            b"null!",
            limits,
            FilterLimits::MAX,
        ),
        Err(MetadataWireDecodeError::InputTooLarge {
            input_bytes: 5,
            max_input_bytes: 4,
        })
    ));
}

#[test]
fn test_decode_json_slice_with_limits_accepts_the_exact_byte_boundary() {
    let limits = MetadataWireLimits::new(4);

    assert!(matches!(
        Metadata::decode_json_slice_with_limits(b"null", limits),
        Err(MetadataWireDecodeError::InvalidJson(_))
    ));
    assert!(matches!(
        MetadataSchema::decode_json_slice_with_limits(b"null", limits),
        Err(MetadataWireDecodeError::InvalidJson(_))
    ));

    assert!(matches!(
        MetadataFilter::decode_json_slice_with_limits(
            b"null",
            limits,
            FilterLimits::MAX,
        ),
        Err(MetadataWireDecodeError::InvalidJson(_))
    ));
}

#[test]
fn test_decode_json_slice_success_paths_for_metadata_schema_and_filter() {
    let metadata = Metadata::new().with("name", "alice");
    let metadata_input =
        serde_json::to_vec(&metadata).expect("metadata should serialize");
    assert_eq!(
        Metadata::decode_json_slice(&metadata_input)
            .expect("metadata should decode"),
        metadata
    );

    let schema = MetadataSchema::builder()
        .required("name", qubit_datatype::DataType::String)
        .build()
        .expect("schema should build");
    let schema_input =
        serde_json::to_vec(&schema).expect("schema should serialize");
    assert_eq!(
        MetadataSchema::decode_json_slice(&schema_input)
            .expect("schema should decode"),
        schema
    );

    let filter = MetadataFilter::builder()
        .expression(
            qubit_metadata::FilterExpression::builder()
                .exists("name")
                .build()
                .expect("expression should build"),
        )
        .build()
        .expect("filter should build");
    let filter_input =
        serde_json::to_vec(&filter).expect("filter should serialize");
    assert_eq!(
        MetadataFilter::decode_json_slice(&filter_input)
            .expect("filter should decode"),
        filter
    );
}

#[test]
fn test_decode_json_slice_rejects_malformed_metadata_envelopes() {
    for input in [
        br#"{"values":{}}"#.as_slice(),
        br#"{"version":1}"#.as_slice(),
        br#"{"version":1,"version":1,"values":{}}"#.as_slice(),
        br#"{"version":1,"values":{},"values":{}}"#.as_slice(),
        br#"{"version":1,"values":{},"extra":true}"#.as_slice(),
        br#"null"#.as_slice(),
    ] {
        assert!(
            Metadata::decode_json_slice(input).is_err(),
            "metadata envelope should be rejected: {}",
            String::from_utf8_lossy(input)
        );
    }
}

#[test]
fn test_decode_json_slice_rejects_malformed_schema_envelopes() {
    for input in [
        br#"{"fields":{},"unknown_metadata_field_policy":"reject","unknown_filter_field_policy":"reject"}"#.as_slice(),
        br#"{"version":1,"unknown_metadata_field_policy":"reject","unknown_filter_field_policy":"reject"}"#.as_slice(),
        br#"{"version":1,"fields":{},"unknown_filter_field_policy":"reject"}"#.as_slice(),
        br#"{"version":1,"fields":{},"unknown_metadata_field_policy":"reject"}"#.as_slice(),
        br#"{"version":1,"version":1,"fields":{},"unknown_metadata_field_policy":"reject","unknown_filter_field_policy":"reject"}"#.as_slice(),
        br#"{"version":1,"fields":{},"fields":{},"unknown_metadata_field_policy":"reject","unknown_filter_field_policy":"reject"}"#.as_slice(),
        br#"{"version":1,"fields":{},"unknown_metadata_field_policy":"reject","unknown_metadata_field_policy":"reject","unknown_filter_field_policy":"reject"}"#.as_slice(),
        br#"{"version":1,"fields":{},"unknown_metadata_field_policy":"reject","unknown_filter_field_policy":"reject","extra":true}"#.as_slice(),
        br#"null"#.as_slice(),
    ] {
        assert!(
            MetadataSchema::decode_json_slice(input).is_err(),
            "schema envelope should be rejected: {}",
            String::from_utf8_lossy(input)
        );
    }
}

#[test]
fn test_decode_json_slice_with_limits_reports_excessive_metadata_and_schema_entries()
 {
    let metadata = Metadata::new().with("first", 1_i64).with("second", 2_i64);
    let metadata_json = serde_json::to_vec(&metadata).unwrap();
    let metadata_limits = MetadataWireLimits::new(metadata_json.len())
        .with_max_metadata_entries(1);

    let error = Metadata::decode_json_slice_with_limits(
        &metadata_json,
        metadata_limits,
    )
    .expect_err("metadata entry limit should reject the input");
    assert!(matches!(
        error,
        MetadataWireDecodeError::LimitExceeded {
            kind: qubit_metadata::MetadataWireLimitKind::MetadataEntries,
            value: 2,
            maximum: 1,
        }
    ));

    let key_limits =
        MetadataWireLimits::new(metadata_json.len()).with_max_key_bytes(4);
    let error =
        Metadata::decode_json_slice_with_limits(&metadata_json, key_limits)
            .expect_err("metadata key limit should reject the input");
    assert!(matches!(
        error,
        MetadataWireDecodeError::LimitExceeded {
            kind: qubit_metadata::MetadataWireLimitKind::KeyBytes,
            value: 5,
            maximum: 4,
        }
    ));

    let schema = MetadataSchema::builder()
        .required("first", qubit_datatype::DataType::Int64)
        .required("second", qubit_datatype::DataType::Int64)
        .build()
        .unwrap();
    let schema_json = serde_json::to_vec(&schema).unwrap();
    let schema_limits =
        MetadataWireLimits::new(schema_json.len()).with_max_schema_fields(1);

    let error = MetadataSchema::decode_json_slice_with_limits(
        &schema_json,
        schema_limits,
    )
    .expect_err("schema field limit should reject the input");
    assert!(matches!(
        error,
        MetadataWireDecodeError::LimitExceeded {
            kind: qubit_metadata::MetadataWireLimitKind::SchemaFields,
            value: 2,
            maximum: 1,
        }
    ));
}

#[test]
fn test_decode_json_slice_with_limits_accepts_exact_metadata_schema_and_key_bounds()
 {
    let metadata = Metadata::new().with("first", 1_i64).with("second", 2_i64);
    let metadata_json =
        serde_json::to_vec(&metadata).expect("metadata should serialize");
    let metadata_limits = MetadataWireLimits::new(metadata_json.len())
        .with_max_metadata_entries(2)
        .with_max_key_bytes(6);

    assert_eq!(
        Metadata::decode_json_slice_with_limits(
            &metadata_json,
            metadata_limits
        )
        .expect("metadata exact bounds should decode"),
        metadata
    );

    let schema = MetadataSchema::builder()
        .required("first", qubit_datatype::DataType::Int64)
        .required("second", qubit_datatype::DataType::Int64)
        .build()
        .expect("schema should build");
    let schema_json =
        serde_json::to_vec(&schema).expect("schema should serialize");
    let schema_limits = MetadataWireLimits::new(schema_json.len())
        .with_max_schema_fields(2)
        .with_max_key_bytes(6);

    assert_eq!(
        MetadataSchema::decode_json_slice_with_limits(
            &schema_json,
            schema_limits
        )
        .expect("schema exact bounds should decode"),
        schema
    );
}

#[test]
fn test_filter_wire_unknown_field_precedes_unsupported_version() {
    let input = br#"{
        "version": 2,
        "expression": {"kind": "all"},
        "options": {"numeric_comparison_policy": "exact"},
        "extra": true
    }"#;

    let error = MetadataFilter::decode_json_slice(input)
        .expect_err("unknown field should reject before version conversion");

    assert!(error.to_string().contains("unknown field `extra`"));
}

#[test]
fn test_filter_wire_duplicate_field_precedes_unsupported_version() {
    let input = br#"{
        "version": 2,
        "version": 1,
        "expression": {"kind": "all"},
        "options": {"numeric_comparison_policy": "exact"}
    }"#;

    let error = MetadataFilter::decode_json_slice(input)
        .expect_err("duplicate field should reject before version conversion");

    assert!(error.to_string().contains("duplicate field `version`"));
}

#[test]
fn test_filter_wire_reports_unsupported_version_after_valid_fields() {
    let input = br#"{
        "version": 2,
        "expression": {"kind": "all"},
        "options": {"numeric_comparison_policy": "exact"}
    }"#;

    let error = MetadataFilter::decode_json_slice(input)
        .expect_err("unsupported version should reject after field decoding");

    assert!(
        error
            .to_string()
            .contains("unsupported MetadataFilter wire format version 2")
    );
}

#[test]
fn test_decode_json_slice_with_limits_rejects_unserializable_expanded_bounds() {
    let mut values = serde_json::Map::new();
    for index in 0..=4_096 {
        let key = format!("key-{index}");
        values.insert(
            key,
            serde_json::json!({"scalar": {"int64": index as i64}}),
        );
    }
    let input = serde_json::to_vec(&serde_json::json!({
        "version": 1,
        "values": values,
    }))
    .expect("expanded metadata should serialize");
    let limits = MetadataWireLimits::new(input.len())
        .with_wire(WireLimits::new(input.len()).with_max_map_entries(4_097))
        .with_max_metadata_entries(4_097);

    assert!(matches!(
        Metadata::decode_json_slice_with_limits(&input, limits),
        Err(MetadataWireDecodeError::InvalidLimit {
            kind: qubit_metadata::MetadataWireLimitKind::MetadataEntries,
            value: 4_097,
            maximum: 4_096,
        })
    ));

    let key_limits =
        MetadataWireLimits::new(input.len()).with_max_key_bytes(257);
    assert!(matches!(
        Metadata::decode_json_slice_with_limits(&input, key_limits),
        Err(MetadataWireDecodeError::InvalidLimit {
            kind: qubit_metadata::MetadataWireLimitKind::KeyBytes,
            value: 257,
            maximum: 256,
        })
    ));
}

#[test]
fn test_schema_decode_rejects_unserializable_expanded_field_bounds() {
    let schema = MetadataSchema::builder()
        .required("field", qubit_datatype::DataType::Int64)
        .build()
        .expect("schema should build");
    let input = serde_json::to_vec(&schema).expect("schema should serialize");
    let limits = MetadataWireLimits::new(input.len())
        .with_wire(WireLimits::new(input.len()).with_max_map_entries(4_097))
        .with_max_schema_fields(4_097);

    assert!(matches!(
        MetadataSchema::decode_json_slice_with_limits(&input, limits),
        Err(MetadataWireDecodeError::InvalidLimit {
            kind: qubit_metadata::MetadataWireLimitKind::SchemaFields,
            value: 4_097,
            maximum: 4_096,
        })
    ));
}

#[test]
fn test_decode_json_slice_with_limits_applies_shared_value_budget() {
    let metadata = Metadata::new().with("values", serde_json::json!([1, 2]));
    let input =
        serde_json::to_vec(&metadata).expect("metadata should serialize");
    let limits = MetadataWireLimits::new(input.len())
        .with_wire(WireLimits::new(input.len()).with_max_collection_items(1));

    assert!(matches!(
        Metadata::decode_json_slice_with_limits(&input, limits),
        Err(MetadataWireDecodeError::Value(
            ValueWireDecodeError::LimitExceeded {
                kind: qubit_value::ValueWireLimitKind::CollectionItems,
                value: 2,
                maximum: 1,
            }
        ))
    ));
}

#[test]
fn test_decode_json_slice_with_limits_preserves_structured_filter_errors() {
    let filter = serde_json::to_value(MetadataFilter::all()).unwrap();
    assert!(filter.get("limits").is_none());
}
