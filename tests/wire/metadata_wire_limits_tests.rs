// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the bounded public JSON decoding APIs.

use qubit_metadata::{
    FilterLimits, Metadata, MetadataFilter, MetadataSchema,
    MetadataWireDecodeError, MetadataWireLimits,
};
use qubit_value::{ValueWireDecodeError, WireLimits};

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
fn test_decode_json_slice_with_limits_rejects_excessive_metadata_and_schema_entries(
) {
    let metadata = Metadata::new().with("first", 1_i64).with("second", 2_i64);
    let metadata_json = serde_json::to_vec(&metadata).unwrap();
    let metadata_limits = MetadataWireLimits::new(metadata_json.len())
        .with_max_metadata_entries(1);

    assert!(matches!(
        Metadata::decode_json_slice_with_limits(
            &metadata_json,
            metadata_limits
        ),
        Err(MetadataWireDecodeError::LimitExceeded {
            kind: qubit_metadata::MetadataWireLimitKind::MetadataEntries,
            value: 2,
            maximum: 1,
        })
    ));
    let key_limits =
        MetadataWireLimits::new(metadata_json.len()).with_max_key_bytes(4);
    assert!(matches!(
        Metadata::decode_json_slice_with_limits(&metadata_json, key_limits),
        Err(MetadataWireDecodeError::LimitExceeded {
            kind: qubit_metadata::MetadataWireLimitKind::KeyBytes,
            value: 5,
            maximum: 4,
        })
    ));

    let schema = MetadataSchema::builder()
        .required("first", qubit_datatype::DataType::Int64)
        .required("second", qubit_datatype::DataType::Int64)
        .build()
        .unwrap();
    let schema_json = serde_json::to_vec(&schema).unwrap();
    let schema_limits =
        MetadataWireLimits::new(schema_json.len()).with_max_schema_fields(1);

    assert!(matches!(
        MetadataSchema::decode_json_slice_with_limits(
            &schema_json,
            schema_limits
        ),
        Err(MetadataWireDecodeError::LimitExceeded {
            kind: qubit_metadata::MetadataWireLimitKind::SchemaFields,
            value: 2,
            maximum: 1,
        })
    ));
}

#[test]
fn test_decode_json_slice_with_limits_honors_expanded_map_bounds() {
    let mut metadata = Metadata::new();
    for index in 0..=4_096 {
        let key = format!("key-{index}");
        metadata.set(&key, index as i64);
    }
    let input = serde_json::to_vec(&metadata)
        .expect("expanded metadata should serialize");
    let limits = MetadataWireLimits::new(input.len())
        .with_wire(WireLimits::new(input.len()).with_max_map_entries(4_097))
        .with_max_metadata_entries(4_097);

    let decoded = Metadata::decode_json_slice_with_limits(&input, limits)
        .expect("expanded metadata should fit expanded limits");
    assert_eq!(decoded, metadata);
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
    let mut filter = serde_json::to_value(MetadataFilter::all()).unwrap();
    filter["limits"]["max_nodes"] = serde_json::json!(0);
    let input = serde_json::to_vec(&filter).unwrap();
    let limits = MetadataWireLimits::new(input.len());

    assert!(matches!(
        MetadataFilter::decode_json_slice_with_limits(
            &input,
            limits,
            FilterLimits::MAX,
        ),
        Err(MetadataWireDecodeError::Filter(
            qubit_metadata::MetadataError::InvalidFilterLimit { .. }
        ))
    ));
}
