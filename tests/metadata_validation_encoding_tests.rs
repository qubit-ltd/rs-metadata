// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
#![cfg(feature = "json")]

use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueLimits;
use qubit_json::encode::JsonSerializationErrorKind;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataError;
use qubit_metadata::MetadataWireEncodeError;
use qubit_metadata::MetadataWireLimitKind;
use qubit_metadata::default_json_encode_limits;
use qubit_value::Value;

#[derive(Debug, PartialEq, Eq)]
enum Signature {
    Success,
    Budget(JsonResource),
    Quantity(JsonResource),
    Wire(MetadataWireLimitKind, usize, usize),
    Domain,
    Syntax,
    Json(JsonSerializationErrorKind),
    Io,
}

fn signature(result: Result<(), MetadataWireEncodeError>) -> Signature {
    match result {
        Ok(()) => Signature::Success,
        Err(MetadataWireEncodeError::Budget(error)) => Signature::Budget(*error.resource()),
        Err(MetadataWireEncodeError::Quantity { resource, .. }) => Signature::Quantity(resource),
        Err(MetadataWireEncodeError::Domain(MetadataError::WireLimitExceeded { kind, value, maximum })) => {
            Signature::Wire(kind, value, maximum)
        }
        Err(MetadataWireEncodeError::Domain(_)) => Signature::Domain,
        Err(MetadataWireEncodeError::Syntax(_)) => Signature::Syntax,
        Err(MetadataWireEncodeError::Json(error)) => Signature::Json(error.kind()),
        Err(MetadataWireEncodeError::Io(_)) => Signature::Io,
        Err(_) => panic!("unexpected metadata encoding error"),
    }
}

fn assert_equivalent(metadata: &Metadata, limits: JsonEncodeLimits) -> Signature {
    let encoded = metadata.to_json_vec_with_limits(limits);
    let mut written = Vec::new();
    let writer_result = metadata.to_json_writer_with_limits(&mut written, limits);
    if let Ok(bytes) = &encoded {
        assert!(writer_result.is_ok());
        assert_eq!(&written, bytes);
    } else {
        assert!(written.is_empty());
    }
    let expected = signature(encoded.map(drop));
    assert_eq!(signature(writer_result), expected);
    assert_eq!(signature(metadata.validate_json_with_limits(limits)), expected);
    expected
}

#[test]
fn test_validation_only_matches_structured_errors_and_validation_order() {
    let empty = Metadata::new();
    let ordinary = Metadata::new().with("integer", 7_i64).with("enabled", true);
    let mut special = Metadata::new();
    special.set("escaped", "quote=\" slash=\\ newline=\n tab=\t");
    special.set("unicode", "café 東京 🛰");
    special.set("nested", serde_json::json!({"outer": [1, {"inner": true}]}));
    let non_finite = Metadata::new().with("nan", Value::Float64(f64::NAN));
    for metadata in [&empty, &ordinary, &special, &non_finite] {
        assert_equivalent(metadata, default_json_encode_limits());
    }

    let mut too_many = Metadata::new();
    for index in 0..=4096 {
        too_many.set(&format!("key_{index:04}"), index as i64);
    }
    assert_eq!(
        assert_equivalent(&too_many, default_json_encode_limits()),
        Signature::Wire(MetadataWireLimitKind::Entries, 4097, 4096)
    );
    let long_key = "k".repeat(257);
    let long_key_metadata = Metadata::new().with(&long_key, 1_i64);
    assert_eq!(
        assert_equivalent(&long_key_metadata, default_json_encode_limits()),
        Signature::Wire(MetadataWireLimitKind::KeyBytes, 257, 256)
    );

    let value_budget_cases = [
        (
            serde_json::json!({"nested": true}),
            JsonValueLimits::builder().max_depth(1).build(),
            JsonResource::Depth,
        ),
        (
            serde_json::json!(true),
            JsonValueLimits::builder().max_nodes(0).build(),
            JsonResource::Nodes,
        ),
        (
            serde_json::json!([1]),
            JsonValueLimits::builder().max_sequence_items(0).build(),
            JsonResource::SequenceItems,
        ),
        (
            serde_json::json!({"a": 1}),
            JsonValueLimits::builder().max_map_entries(0).build(),
            JsonResource::MapEntries,
        ),
        (
            serde_json::json!({"a": 1}),
            JsonValueLimits::builder().max_key_bytes(0).build(),
            JsonResource::KeyBytes,
        ),
        (
            serde_json::json!("long"),
            JsonValueLimits::builder().max_string_bytes(2).build(),
            JsonResource::StringBytes,
        ),
        (
            serde_json::json!(123),
            JsonValueLimits::builder().max_number_bytes(2).build(),
            JsonResource::NumberBytes,
        ),
        (
            serde_json::json!({"a": "b"}),
            JsonValueLimits::builder().max_payload_bytes(1).build(),
            JsonResource::PayloadBytes,
        ),
    ];
    for (json, value_limits, expected_resource) in value_budget_cases {
        let metadata = Metadata::new().with("json", Value::Json(json));
        let limits = default_json_encode_limits()
            .into_builder()
            .value_limits(value_limits)
            .build();
        assert_eq!(
            assert_equivalent(&metadata, limits),
            Signature::Budget(expected_resource)
        );
    }

    let boundary = Metadata::new().with("edge", "quote=\" café");
    let encoded_len = serde_json::to_vec(&boundary).expect("serialize boundary").len();
    for (cap, expected) in [
        (encoded_len - 1, Signature::Budget(JsonResource::OutputBytes)),
        (encoded_len, Signature::Success),
        (encoded_len + 1, Signature::Success),
    ] {
        let limits = default_json_encode_limits()
            .into_builder()
            .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, cap))
            .build();
        assert_eq!(assert_equivalent(&boundary, limits), expected);
    }

    // Wire contract is checked before value budgets.
    let mut wire_and_value = Metadata::new().with("json", Value::Json(serde_json::json!("too long")));
    for index in 0..4096 {
        wire_and_value.set(&format!("key_{index:04}"), "x");
    }
    let limits = default_json_encode_limits()
        .into_builder()
        .value_limits(JsonValueLimits::builder().max_string_bytes(1).build())
        .build();
    assert_eq!(
        assert_equivalent(&wire_and_value, limits),
        Signature::Wire(MetadataWireLimitKind::Entries, 4097, 4096)
    );

    // Value preflight precedes output accounting.
    let metadata = Metadata::new().with("json", Value::Json(serde_json::json!("too long")));
    let limits = default_json_encode_limits()
        .into_builder()
        .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 1))
        .value_limits(JsonValueLimits::builder().max_string_bytes(1).build())
        .build();
    assert_eq!(
        assert_equivalent(&metadata, limits),
        Signature::Budget(JsonResource::StringBytes)
    );

    // Incremental output preserves serializer/output-budget precedence by key
    // order.
    let mut output_then_serialize = Metadata::new();
    output_then_serialize.set("a_large", "x".repeat(512));
    output_then_serialize.set("z_nan", Value::Float64(f64::NAN));
    let mut serialize_then_output = Metadata::new();
    serialize_then_output.set("a_nan", Value::Float64(f64::NAN));
    serialize_then_output.set("z_large", "x".repeat(512));
    let limits = default_json_encode_limits()
        .into_builder()
        .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 128))
        .build();
    assert_eq!(
        assert_equivalent(&output_then_serialize, limits),
        Signature::Budget(JsonResource::OutputBytes)
    );
    assert_eq!(
        assert_equivalent(&serialize_then_output, limits),
        Signature::Json(JsonSerializationErrorKind::CustomSerialization)
    );
}
