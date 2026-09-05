// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for metadata JSON budget integration and domain limits.

#![cfg(all(feature = "json", feature = "schema"))]

use qubit_budget::BudgetError;
use qubit_budget::InsufficientBudgetError;
use qubit_budget::ResourceBudget;
use qubit_budget::ResourceLimit;
use qubit_budget::StructureLimits;
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueLimits;
use qubit_datatype::DataType;
use qubit_metadata::FilterExpression;
use qubit_metadata::FilterLimitKind;
use qubit_metadata::FilterLimits;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataError;
use qubit_metadata::MetadataFilter;
use qubit_metadata::MetadataLimits;
use qubit_metadata::MetadataSchema;
use qubit_metadata::MetadataWireDecodeError;
use qubit_metadata::MetadataWireEncodeError;
use qubit_metadata::MetadataWireLimitKind;

fn filter_input(expression: &str) -> Vec<u8> {
    format!(r#"{{"version":1,"expression":{expression},"options":{{"numeric_comparison_policy":"exact"}}}}"#,)
        .into_bytes()
}

fn input_limit(maximum: usize) -> MetadataLimits {
    MetadataLimits::builder()
        .json_decode(
            JsonDecodeLimits::builder()
                .input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, maximum))
                .build(),
        )
        .build()
}

#[test]
fn test_resource_budget_preserves_filter_limit_facts() {
    let mut budget = ResourceBudget::<FilterLimitKind, usize>::new(FilterLimitKind::Nodes, 1);
    budget.try_consume(1).expect("first node should fit");
    let error = budget.try_consume(1).expect_err("second node should fail");
    assert_eq!(
        error,
        InsufficientBudgetError {
            resource: FilterLimitKind::Nodes,
            limit: 1,
            remaining: 0,
            requested: 1,
        }
    );
}

#[test]
fn test_decode_rejects_input_before_json_parsing() {
    let error = Metadata::decode_json_slice_with_limits(b"null!", input_limit(4))
        .expect_err("input bytes should be checked before parsing");
    assert!(
        matches!(
            error,
            MetadataWireDecodeError::Budget(BudgetError::Insufficient {
                resource: JsonResource::InputBytes,
                limit: 4,
                remaining: 4,
                requested: 5,
            })
        ),
        "unexpected budget error: {error:?}"
    );
}

#[test]
fn test_decode_accepts_exact_input_boundary_then_reports_json_error() {
    let error =
        Metadata::decode_json_slice_with_limits(b"null", input_limit(4)).expect_err("null is not a metadata envelope");
    assert!(matches!(error, MetadataWireDecodeError::InvalidJson(_)));
}

#[test]
fn test_filter_domain_limits_remain_separate_from_json_budget() {
    let input = filter_input(r#"{"kind":"not","expression":{"kind":"all"}}"#);
    let limits = FilterLimits::builder().max_depth(1).build().unwrap();
    let error = MetadataFilter::decode_json_slice_with_limits(&input, MetadataLimits::default(), limits)
        .expect_err("nested expression should exceed receiver depth");
    assert!(matches!(
        error,
        MetadataWireDecodeError::Filter(MetadataError::FilterLimitExceeded {
            kind: FilterLimitKind::Depth,
            value: 2,
            maximum: 1,
        })
    ));
}

#[test]
fn test_filter_key_limit_preserves_structured_error() {
    let input = filter_input(r#"{"kind":"exists","key":"status"}"#);
    let limits = FilterLimits::builder().max_key_bytes(5).build().unwrap();
    let error = MetadataFilter::decode_json_slice_with_limits(&input, MetadataLimits::default(), limits)
        .expect_err("the filter key should exceed the receiver limit");
    assert!(matches!(
        error,
        MetadataWireDecodeError::Filter(MetadataError::FilterLimitExceeded {
            kind: FilterLimitKind::KeyBytes,
            value: 6,
            maximum: 5,
        })
    ));
}

#[test]
fn test_filter_node_limit_preserves_structured_error() {
    let input = filter_input(r#"{"kind":"or","children":[{"kind":"all"},{"kind":"all"}]}"#);
    let limits = FilterLimits::builder().max_nodes(1).build().unwrap();
    let error = MetadataFilter::decode_json_slice_with_limits(&input, MetadataLimits::default(), limits)
        .expect_err("the second node should exceed the receiver limit");
    assert!(matches!(
        error,
        MetadataWireDecodeError::Filter(MetadataError::FilterLimitExceeded {
            kind: FilterLimitKind::Nodes,
            value: 2,
            maximum: 1,
        })
    ));
}

#[test]
fn test_filter_set_values_limit_preserves_structured_error() {
    let input =
        filter_input(r#"{"kind":"in","key":"status","values":[{"scalar":{"string":"a"}},{"scalar":{"string":"b"}}]}"#);
    let limits = FilterLimits::builder().max_set_values(1).build().unwrap();
    let error = MetadataFilter::decode_json_slice_with_limits(&input, MetadataLimits::default(), limits)
        .expect_err("the second set value should exceed the receiver limit");
    assert!(matches!(
        error,
        MetadataWireDecodeError::Filter(MetadataError::FilterLimitExceeded {
            kind: FilterLimitKind::SetValues,
            value: 2,
            maximum: 1,
        })
    ));
}

#[test]
fn test_metadata_domain_entry_limit_is_preserved() {
    let metadata = Metadata::new().with("first", 1_i64).with("second", 2_i64);
    let input = serde_json::to_vec(&metadata).unwrap();
    let limits = MetadataLimits::builder().max_metadata_entries(1).build();
    let error = Metadata::decode_json_slice_with_limits(&input, limits)
        .expect_err("metadata domain entry limit should reject the input");
    assert!(matches!(
        error,
        MetadataWireDecodeError::Domain(MetadataError::WireLimitExceeded {
            kind: MetadataWireLimitKind::Entries,
            value: 2,
            maximum: 1,
        })
    ));
}

#[test]
fn test_metadata_domain_key_limit_is_preserved() {
    let metadata = Metadata::new().with("first", 1_i64);
    let input = serde_json::to_vec(&metadata).unwrap();
    let limits = MetadataLimits::builder().max_key_bytes(4).build();
    let error = Metadata::decode_json_slice_with_limits(&input, limits)
        .expect_err("metadata domain key limit should reject the input");
    assert!(matches!(
        error,
        MetadataWireDecodeError::Domain(MetadataError::WireLimitExceeded {
            kind: MetadataWireLimitKind::KeyBytes,
            value: 5,
            maximum: 4,
        })
    ));
}

#[test]
fn test_json_budget_structural_limit_is_structured_source() {
    let metadata = Metadata::new().with("name", "alice");
    let input = serde_json::to_vec(&metadata).unwrap();
    let structure = StructureLimits::builder()
        .nodes_limit(ResourceLimit::new(JsonResource::Nodes, 1))
        .build();
    let value = JsonValueLimits::builder().structure_limits(structure).build();
    let limits = JsonDecodeLimits::builder().value_limits(value).build();
    let error = Metadata::decode_json_slice_with_limits(&input, MetadataLimits::builder().json_decode(limits).build())
        .expect_err("metadata envelope needs more than one JSON node");
    assert!(
        matches!(
            &error,
            MetadataWireDecodeError::Budget(BudgetError::Insufficient {
                resource: JsonResource::Nodes,
                ..
            })
        ),
        "unexpected budget error: {error:?}"
    );
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn test_metadata_schema_and_filter_round_trip() {
    let metadata = Metadata::new().with("name", "alice");
    let input = serde_json::to_vec(&metadata).unwrap();
    assert_eq!(Metadata::decode_json_slice(&input).unwrap(), metadata);

    let schema = MetadataSchema::builder()
        .required("name", DataType::String)
        .build()
        .unwrap();
    let input = serde_json::to_vec(&schema).unwrap();
    assert_eq!(MetadataSchema::decode_json_slice(&input).unwrap(), schema);

    let filter = MetadataFilter::builder()
        .expression(FilterExpression::builder().exists("name").build().unwrap())
        .build()
        .unwrap();
    let input = serde_json::to_vec(&filter).unwrap();
    assert_eq!(MetadataFilter::decode_json_slice(&input).unwrap(), filter);
}

#[test]
fn test_strict_envelopes_reject_duplicates_and_unknown_fields() {
    for input in [
        br#"{"values":{}}"#.as_slice(),
        br#"{"version":1,"values":{},"extra":true}"#.as_slice(),
        br#"{"version":1,"version":1,"values":{}}"#.as_slice(),
    ] {
        assert!(Metadata::decode_json_slice(input).is_err());
    }
}

#[test]
fn test_default_json_key_budget_allows_large_nested_keys() {
    let nested_key = "k".repeat(1_024);
    let input = format!(r#"{{"version":1,"values":{{"payload":{{"scalar":{{"json":{{"{nested_key}":1}}}}}}}}}}"#,);
    Metadata::decode_json_slice(input.as_bytes()).expect("nested JSON keys should use the generic 256 KiB budget");
}

#[test]
fn test_metadata_outer_key_budget_remains_256_bytes() {
    let outer_key = "k".repeat(257);
    let input = format!(r#"{{"version":1,"values":{{"{outer_key}":{{"scalar":{{"int64":1}}}}}}}}"#,);
    let error =
        Metadata::decode_json_slice(input.as_bytes()).expect_err("metadata outer keys must retain the 256-byte cap");
    assert!(matches!(
        error,
        MetadataWireDecodeError::Domain(MetadataError::WireLimitExceeded {
            kind: MetadataWireLimitKind::KeyBytes,
            value: 257,
            maximum: 256
        })
    ));
}

#[test]
fn test_metadata_encode_uses_output_budget() {
    let metadata = Metadata::new().with("name", "alice");
    let limits = JsonEncodeLimits::builder()
        .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 1))
        .build();
    let error = metadata
        .to_json_vec_with_limits(limits)
        .expect_err("one output byte cannot encode metadata");
    assert!(matches!(
        error,
        MetadataWireEncodeError::Budget(BudgetError::Insufficient {
            resource: JsonResource::OutputBytes,
            ..
        })
    ));
}

#[test]
fn test_metadata_schema_and_filter_encode_round_trip() {
    let schema = MetadataSchema::builder()
        .required("name", DataType::String)
        .build()
        .unwrap();
    let schema_json = schema.to_json_vec().unwrap();
    assert_eq!(MetadataSchema::decode_json_slice(&schema_json).unwrap(), schema);

    let filter = MetadataFilter::builder()
        .expression(FilterExpression::builder().exists("name").build().unwrap())
        .build()
        .unwrap();
    let filter_json = filter.to_json_vec().unwrap();
    assert_eq!(MetadataFilter::decode_json_slice(&filter_json).unwrap(), filter);
}

#[test]
fn test_schema_limits_and_wire_version_are_structured() {
    let schema = MetadataSchema::builder()
        .required("one", DataType::Int64)
        .optional("two", DataType::String)
        .build()
        .expect("schema");
    let input = serde_json::to_vec(&schema).expect("wire");
    let error =
        MetadataSchema::decode_json_slice_with_limits(&input, MetadataLimits::builder().max_schema_fields(1).build())
            .expect_err("field count");
    assert!(matches!(
        error,
        MetadataWireDecodeError::Domain(MetadataError::WireLimitExceeded {
            kind: MetadataWireLimitKind::Entries,
            value: 2,
            maximum: 1,
        })
    ));
    let error =
        MetadataSchema::decode_json_slice_with_limits(&input, MetadataLimits::builder().max_key_bytes(2).build())
            .expect_err("key length");
    assert!(matches!(
        error,
        MetadataWireDecodeError::Domain(MetadataError::WireLimitExceeded {
            kind: MetadataWireLimitKind::KeyBytes,
            value: 3,
            maximum: 2,
        })
    ));
    let error = Metadata::decode_json_slice(br#"{"version":2,"values":{}}"#).expect_err("version");
    assert!(matches!(
        error,
        MetadataWireDecodeError::UnsupportedVersion { expected: 1, actual: 2 }
    ));
}
