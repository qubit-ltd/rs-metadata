// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Regression coverage for the metadata decoder fuzz round-trip oracle.

#![cfg(feature = "json")]

use qubit_budget::ResourceLimit;
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
