// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Domain key boundaries for metadata, schema, and filter decoding.

#![cfg(feature = "json")]

#[cfg(feature = "schema")]
use qubit_datatype::DataType;
#[cfg(feature = "filter")]
use qubit_metadata::FilterExpression;
#[cfg(feature = "filter")]
use qubit_metadata::FilterLimitKind;
#[cfg(feature = "filter")]
use qubit_metadata::FilterLimits;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataError;
#[cfg(feature = "filter")]
use qubit_metadata::MetadataFilter;
use qubit_metadata::MetadataLimits;
#[cfg(feature = "schema")]
use qubit_metadata::MetadataSchema;
use qubit_metadata::MetadataWireDecodeError;
use qubit_metadata::MetadataWireLimitKind;

fn metadata_with_key(key: &str) -> Metadata {
    Metadata::new().with(key, "value")
}

#[cfg(feature = "schema")]
fn schema_with_key(key: &str) -> MetadataSchema {
    MetadataSchema::builder()
        .required(key, DataType::String)
        .build()
        .expect("schema should build")
}

#[cfg(feature = "filter")]
fn filter_with_key(key: &str) -> MetadataFilter {
    MetadataFilter::builder()
        .expression(
            FilterExpression::builder()
                .exists(key)
                .build()
                .expect("expression should build"),
        )
        .build()
        .expect("filter should build")
}

fn assert_domain_key_error(error: MetadataWireDecodeError, value: usize, maximum: usize) {
    assert!(matches!(
        error,
        MetadataWireDecodeError::Domain(MetadataError::WireLimitExceeded {
            kind: MetadataWireLimitKind::KeyBytes,
            value: actual,
            maximum: actual_maximum,
        }) if actual == value && actual_maximum == maximum
    ));
}

#[test]
fn test_metadata_keys_use_utf8_byte_limits_and_accept_exact_boundary() {
    let limits = MetadataLimits::builder()
        .max_key_bytes(16)
        .build()
        .expect("limits");
    for length in [15, 16] {
        let input = serde_json::to_vec(&metadata_with_key(&"k".repeat(length))).expect("wire");
        Metadata::decode_json_slice_with_limits(&input, limits).expect("key should fit");
    }
    let input = serde_json::to_vec(&metadata_with_key(&"k".repeat(17))).expect("wire");
    assert_domain_key_error(
        Metadata::decode_json_slice_with_limits(&input, limits)
            .expect_err("key should exceed limit"),
        17,
        16,
    );

    let input = serde_json::to_vec(&metadata_with_key(&"北".repeat(8))).expect("wire");
    assert_domain_key_error(
        Metadata::decode_json_slice_with_limits(&input, limits)
            .expect_err("24 UTF-8 bytes should exceed limit"),
        24,
        16,
    );
    let escaped = String::from_utf8(input)
        .expect("valid UTF-8")
        .replace(&"北".repeat(8), &"\\u5317".repeat(8));
    assert_domain_key_error(
        Metadata::decode_json_slice_with_limits(escaped.as_bytes(), limits)
            .expect_err("escaped key should have the same decoded byte length"),
        24,
        16,
    );
}

#[cfg(feature = "schema")]
#[test]
fn test_schema_keys_use_utf8_byte_limits_and_accept_exact_boundary() {
    let limits = MetadataLimits::builder()
        .max_key_bytes(16)
        .build()
        .expect("limits");
    for length in [15, 16] {
        let input = serde_json::to_vec(&schema_with_key(&"k".repeat(length))).expect("wire");
        MetadataSchema::decode_json_slice_with_limits(&input, limits).expect("key should fit");
    }
    let input = serde_json::to_vec(&schema_with_key(&"k".repeat(17))).expect("wire");
    assert_domain_key_error(
        MetadataSchema::decode_json_slice_with_limits(&input, limits)
            .expect_err("key should exceed limit"),
        17,
        16,
    );

    let input = serde_json::to_vec(&schema_with_key(&"北".repeat(8))).expect("wire");
    assert_domain_key_error(
        MetadataSchema::decode_json_slice_with_limits(&input, limits)
            .expect_err("24 UTF-8 bytes should exceed limit"),
        24,
        16,
    );
    let escaped = String::from_utf8(input)
        .expect("valid UTF-8")
        .replace(&"北".repeat(8), &"\\u5317".repeat(8));
    assert_domain_key_error(
        MetadataSchema::decode_json_slice_with_limits(escaped.as_bytes(), limits)
            .expect_err("escaped key should have the same decoded byte length"),
        24,
        16,
    );
}

#[cfg(feature = "filter")]
#[test]
fn test_filter_keys_use_utf8_byte_limits_and_accept_exact_boundary() {
    let limits = FilterLimits::builder()
        .max_key_bytes(16)
        .build()
        .expect("limits");
    for length in [15, 16] {
        let input = serde_json::to_vec(&filter_with_key(&"k".repeat(length))).expect("wire");
        let decoded = MetadataFilter::decode_json_slice_with_limits(
            &input,
            MetadataLimits::default(),
            limits,
        )
        .expect("key should fit");
        assert_eq!(decoded, filter_with_key(&"k".repeat(length)));
    }
    let input = serde_json::to_vec(&filter_with_key(&"k".repeat(17))).expect("wire");
    assert!(matches!(
        MetadataFilter::decode_json_slice_with_limits(&input, MetadataLimits::default(), limits)
            .expect_err("key should exceed limit"),
        MetadataWireDecodeError::Filter(MetadataError::FilterLimitExceeded {
            kind: FilterLimitKind::KeyBytes,
            value: 17,
            maximum: 16,
        })
    ));

    let input = serde_json::to_vec(&filter_with_key(&"北".repeat(8))).expect("wire");
    let escaped = String::from_utf8(input)
        .expect("valid UTF-8")
        .replace(&"北".repeat(8), &"\\u5317".repeat(8));
    for input in [
        serde_json::to_vec(&filter_with_key(&"北".repeat(8))).expect("wire"),
        escaped.into_bytes(),
    ] {
        assert!(matches!(
            MetadataFilter::decode_json_slice_with_limits(
                &input,
                MetadataLimits::default(),
                limits
            )
            .expect_err("24 UTF-8 bytes should exceed limit"),
            MetadataWireDecodeError::Filter(MetadataError::FilterLimitExceeded {
                kind: FilterLimitKind::KeyBytes,
                value: 24,
                maximum: 16,
            })
        ));
    }
}

#[test]
fn test_ordinary_serde_metadata_key_limit_retains_255_256_257_boundary() {
    for length in [255, 256] {
        let input = serde_json::to_vec(&metadata_with_key(&"k".repeat(length))).expect("wire");
        serde_json::from_slice::<Metadata>(&input).expect("key should fit hard limit");
    }
    let input = serde_json::to_vec(&metadata_with_key(&"k".repeat(256))).expect("wire");
    let input = String::from_utf8(input).expect("valid UTF-8").replace(
        &format!("\"{}\"", "k".repeat(256)),
        &format!("\"{}\"", "k".repeat(257)),
    );
    let error = serde_json::from_slice::<Metadata>(input.as_bytes())
        .expect_err("key should exceed hard limit");
    assert!(error.to_string().contains("257"));
    assert!(error.to_string().contains("256"));
}
