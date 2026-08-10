// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
//! Tests for metadata JSON budget integration and domain limits.

#![cfg(all(feature = "json", feature = "schema"))]

use qubit_budget::{
    BudgetError,
    JsonResource,
    ResourceBudget,
    ResourceLimit,
    StructureLimits,
};
use qubit_metadata::{
    FilterLimitKind,
    FilterLimits,
    Metadata,
    MetadataFilter,
    MetadataLimits,
    MetadataSchema,
    MetadataWireDecodeError,
    default_json_limits,
};

fn filter_input(expression: &str) -> Vec<u8> {
    format!(
        r#"{{"version":1,"expression":{expression},"options":{{"numeric_comparison_policy":"exact"}}}}"#,
    )
    .into_bytes()
}

fn input_limit(maximum: usize) -> MetadataLimits {
    MetadataLimits::default().with_json(
        default_json_limits().with_input_bytes_limit(ResourceLimit::new(
            JsonResource::InputBytes,
            maximum,
        )),
    )
}

#[test]
fn test_resource_budget_preserves_filter_limit_facts() {
    let mut budget = ResourceBudget::<FilterLimitKind, usize>::new(
        FilterLimitKind::Nodes,
        1,
    );
    budget.try_consume(1).expect("first node should fit");
    let error = budget.try_consume(1).expect_err("second node should fail");
    assert_eq!(
        error,
        BudgetError::Insufficient {
            resource: FilterLimitKind::Nodes,
            limit: 1,
            remaining: 0,
            requested: 1,
        }
    );
}

#[test]
fn test_decode_rejects_input_before_json_parsing() {
    let error =
        Metadata::decode_json_slice_with_limits(b"null!", input_limit(4))
            .expect_err("input bytes should be checked before parsing");
    assert!(
        matches!(
            error,
            MetadataWireDecodeError::Budget(BudgetError::LimitExceeded {
                resource: JsonResource::InputBytes,
                actual: 5,
                maximum: 4,
            })
        ),
        "unexpected budget error: {error:?}"
    );
}

#[test]
fn test_decode_accepts_exact_input_boundary_then_reports_json_error() {
    let error =
        Metadata::decode_json_slice_with_limits(b"null", input_limit(4))
            .expect_err("null is not a metadata envelope");
    assert!(matches!(error, MetadataWireDecodeError::InvalidJson(_)));
}

#[test]
fn test_filter_domain_limits_remain_separate_from_json_budget() {
    let input = filter_input(r#"{"kind":"not","expression":{"kind":"all"}}"#);
    let limits = FilterLimits::builder().max_depth(1).build().unwrap();
    let error = MetadataFilter::decode_json_slice_with_limits(
        &input,
        MetadataLimits::default(),
        limits,
    )
    .expect_err("nested expression should exceed receiver depth");
    assert!(error.to_string().contains("Depth value 2"));
}

#[test]
fn test_metadata_domain_entry_limit_is_preserved() {
    let metadata = Metadata::new().with("first", 1_i64).with("second", 2_i64);
    let input = serde_json::to_vec(&metadata).unwrap();
    let limits = MetadataLimits::default().with_max_metadata_entries(1);
    let error = Metadata::decode_json_slice_with_limits(&input, limits)
        .expect_err("metadata domain entry limit should reject the input");
    assert!(
        error
            .to_string()
            .contains("map has more than the limit of 1")
    );
}

#[test]
fn test_metadata_domain_key_limit_is_preserved() {
    let metadata = Metadata::new().with("first", 1_i64);
    let input = serde_json::to_vec(&metadata).unwrap();
    let limits = MetadataLimits::default().with_max_key_bytes(4);
    let error = Metadata::decode_json_slice_with_limits(&input, limits)
        .expect_err("metadata domain key limit should reject the input");
    assert!(error.to_string().contains("exceeding the limit of 4 bytes"));
}

#[test]
fn test_json_budget_structural_limit_is_structured_source() {
    let metadata = Metadata::new().with("name", "alice");
    let input = serde_json::to_vec(&metadata).unwrap();
    let limits = default_json_limits().with_structure_limits(
        StructureLimits::empty()
            .with_nodes_limit(ResourceLimit::new(JsonResource::Nodes, 1)),
    );
    let error = Metadata::decode_json_slice_with_limits(
        &input,
        MetadataLimits::default().with_json(limits),
    )
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
        .required("name", qubit_datatype::DataType::String)
        .build()
        .unwrap();
    let input = serde_json::to_vec(&schema).unwrap();
    assert_eq!(MetadataSchema::decode_json_slice(&input).unwrap(), schema);

    let filter = MetadataFilter::builder()
        .expression(
            qubit_metadata::FilterExpression::builder()
                .exists("name")
                .build()
                .unwrap(),
        )
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
