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
use qubit_metadata::Metadata;
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
