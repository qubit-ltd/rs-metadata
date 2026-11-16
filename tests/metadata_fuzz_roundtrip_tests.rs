// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Regression coverage for the metadata decoder fuzz round-trip oracle.

#![cfg(feature = "json")]

mod support;
use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonResource;
#[cfg(feature = "filter")]
use qubit_metadata::DEFAULT_MAX_JSON_BYTES;
#[cfg(feature = "filter")]
use qubit_metadata::FilterExpression;
#[cfg(feature = "filter")]
use qubit_metadata::FilterLimits;
use qubit_metadata::Metadata;
#[cfg(feature = "filter")]
use qubit_metadata::MetadataFilter;
use qubit_metadata::MetadataLimits;
use qubit_metadata::MetadataWireDecodeError;
use qubit_metadata::MetadataWireEncodeError;
use qubit_metadata::default_json_encode_limits;
use support::bounded_encode;

fn assert_bounded(
    canonical: &[u8],
    encode_vec: impl Fn(JsonEncodeLimits) -> Result<Vec<u8>, MetadataWireEncodeError>,
    encode_writer: impl FnMut(&mut bounded_encode::ProbeWriter, JsonEncodeLimits) -> Result<(), MetadataWireEncodeError>,
) {
    let length = canonical.len();
    bounded_encode::check(
        canonical,
        [1, length.saturating_sub(1).max(1), length, length.saturating_add(1)],
        encode_vec,
        encode_writer,
    );
}

#[test]
fn test_bounded_metadata_encode_oracle_covers_output_and_value_budgets() {
    let empty = Metadata::new();
    let empty_canonical = serde_json::to_vec(&empty).unwrap();
    assert_bounded(
        &empty_canonical,
        |limits| empty.to_json_vec_with_limits(limits),
        |writer, limits| empty.to_json_writer_with_limits(writer, limits),
    );

    let metadata = Metadata::new().with("tenant", "acme \"北\"").with("empty", "");
    let canonical = serde_json::to_vec(&metadata).unwrap();
    assert_bounded(
        &canonical,
        |limits| metadata.to_json_vec_with_limits(limits),
        |writer, limits| metadata.to_json_writer_with_limits(writer, limits),
    );

    let nested = Metadata::new().with(
        "document",
        qubit_value::Value::Json(serde_json::json!({"nested": [{"ratio": 1.25}, true]})),
    );
    let canonical = serde_json::to_vec(&nested).unwrap();
    assert_bounded(
        &canonical,
        |limits| nested.to_json_vec_with_limits(limits),
        |writer, limits| nested.to_json_writer_with_limits(writer, limits),
    );

    for limits in [
        default_json_encode_limits().into_builder().max_depth(1).build(),
        default_json_encode_limits().into_builder().max_string_bytes(1).build(),
    ] {
        let error = metadata
            .to_json_vec_with_limits(limits)
            .expect_err("small value budget must reject");
        assert!(matches!(error, MetadataWireEncodeError::Budget(_)));
        let mut writer = bounded_encode::ProbeWriter::new();
        let writer_error = metadata
            .to_json_writer_with_limits(&mut writer, limits)
            .expect_err("writer must reject same value budget");
        assert!(matches!(writer_error, MetadataWireEncodeError::Budget(_)));
        assert_eq!(writer.writes, 0);
        assert_eq!(writer.bytes, bounded_encode::PREFIX);
    }
}

#[test]
fn test_metadata_validation_fuzz_oracle() {
    let samples = [
        Metadata::new(),
        Metadata::new().with("n", 7_i64).with("enabled", true),
        Metadata::new().with("text", "quote=\" newline=\n 東京 🛰"),
        Metadata::new().with("json", serde_json::json!({"a": [1, {"b": true}]})),
    ];
    for metadata in samples {
        let canonical = serde_json::to_vec(&metadata).unwrap();
        for first in [0, 1, 127, 255] {
            bounded_encode::check_validation(&metadata, canonical.len(), first);
        }
    }
}

#[cfg(feature = "schema")]
#[test]
fn test_bounded_schema_encode_oracle() {
    use qubit_datatype::DataType;
    use qubit_metadata::MetadataSchema;
    let schema = MetadataSchema::builder()
        .required("tenant", DataType::String)
        .build()
        .unwrap();
    let canonical = serde_json::to_vec(&schema).unwrap();
    assert_bounded(
        &canonical,
        |limits| schema.to_json_vec_with_limits(limits),
        |writer, limits| schema.to_json_writer_with_limits(writer, limits),
    );
}

#[cfg(feature = "filter")]
#[test]
fn test_bounded_filter_encode_oracle() {
    let expression = FilterExpression::builder()
        .in_set("region", ["north", "s\"outh", "東京"])
        .eq("enabled", true)
        .build()
        .unwrap();
    let filter = MetadataFilter::builder().expression(expression).build().unwrap();
    let canonical = serde_json::to_vec(&filter).unwrap();
    assert_bounded(
        &canonical,
        |limits| filter.to_json_vec_with_limits(limits),
        |writer, limits| filter.to_json_writer_with_limits(writer, limits),
    );
}

#[test]
fn test_metadata_roundtrip_oracle_allows_number_normalization_growth() {
    let mut input = String::from(r#"{"version":1,"values":{"#);
    for index in 0..4_096 {
        if index > 0 {
            input.push(',');
        }
        let prefix = format!("k{index}");
        let key = format!("{prefix}{}", "a".repeat(224 - prefix.len()));
        input.push('"');
        input.push_str(&key);
        input.push_str(r#"":{"scalar":{"float64":1e2}}"#);
    }
    input.push_str("}}");

    let metadata =
        Metadata::decode_json_slice(input.as_bytes()).expect("default decoder should accept the boundary input");
    let encoded = serde_json::to_vec(&metadata).expect("metadata should serialize");
    assert_eq!(input.len(), 1_040_408);
    assert_eq!(encoded.len(), 1_048_600);
    assert_eq!(encoded.len() - input.len(), 8_192);

    let error = Metadata::decode_json_slice(&encoded)
        .expect_err("the default complete-input limit should reject normalized output");
    assert!(matches!(error, MetadataWireDecodeError::Budget(_)));
    assert!(error.to_string().contains("InputBytes"), "unexpected error: {error}");

    let json_decode = MetadataLimits::default()
        .json_decode()
        .into_builder()
        .input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, encoded.len()))
        .build();
    let limits = MetadataLimits::builder()
        .json_decode(json_decode)
        .build()
        .expect("default domain limits should remain valid");
    let decoded = Metadata::decode_json_slice_with_limits(&encoded, limits)
        .expect("strict decoding should accept the exact encoded length");
    assert_eq!(decoded, metadata);
}

#[cfg(feature = "filter")]
#[test]
fn test_filter_roundtrip_oracle_allows_number_normalization_growth() {
    let mut expression = FilterExpression::builder().eq("n", 100.0_f64);
    for index in 0..6 {
        let length = if index == 0 { 198_066 } else { 170_000 };
        expression = expression.eq(&format!("s{index}"), "a".repeat(length));
    }
    let expression = expression.build().expect("bounded filter should build");
    let filter = MetadataFilter::builder()
        .expression(expression)
        .build()
        .expect("filter should build");
    let canonical = serde_json::to_vec(&filter).expect("filter should serialize");

    // Locate the float64 token in its structural context; string operands may
    // contain arbitrary text and must not be searched or replaced globally.
    let marker = b"\"float64\":";
    let token_start = canonical
        .windows(marker.len())
        .position(|window| window == marker)
        .expect("serialized float64 operand should exist")
        + marker.len();
    assert_eq!(&canonical[token_start..token_start + 5], b"100.0");
    let mut input = canonical.clone();
    input.splice(token_start..token_start + 5, b"1e2".iter().copied());

    let receiver_limits = FilterLimits::builder()
        .max_depth(4)
        .max_nodes(16)
        .max_set_values(8)
        .max_key_bytes(32)
        .build()
        .expect("fuzz receiver limits should be valid");
    assert_eq!(input.len(), DEFAULT_MAX_JSON_BYTES - 1);
    assert_eq!(canonical.len(), DEFAULT_MAX_JSON_BYTES + 1);

    let decoded = MetadataFilter::decode_json_slice_with_limits(&input, MetadataLimits::default(), receiver_limits)
        .expect("candidate input should pass the original fuzz limits");
    let encoded = serde_json::to_vec(&decoded).expect("filter should serialize");
    assert_eq!(encoded, canonical);

    let error = MetadataFilter::decode_json_slice_with_limits(&encoded, MetadataLimits::default(), receiver_limits)
        .expect_err("the default complete-input limit should reject normalized output");
    assert!(matches!(
        error,
        MetadataWireDecodeError::Budget(qubit_budget::BudgetError::Insufficient {
            resource: JsonResource::InputBytes,
            limit: DEFAULT_MAX_JSON_BYTES,
            requested,
            ..
        })
            if requested == encoded.len()
    ));

    let json_decode = MetadataLimits::default()
        .json_decode()
        .into_builder()
        .input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, encoded.len()))
        .build();
    let limits = MetadataLimits::builder()
        .json_decode(json_decode)
        .build()
        .expect("default domain limits should remain valid");
    let restored = MetadataFilter::decode_json_slice_with_limits(&encoded, limits, receiver_limits)
        .expect("strict decoding should accept the exact encoded length");
    assert_eq!(restored, decoded);
}
