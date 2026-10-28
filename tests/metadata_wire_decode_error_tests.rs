// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for structured metadata JSON decoding errors.

#![cfg(feature = "json")]

use std::error::Error;
use std::io;

use qubit_budget::BudgetError;
use qubit_budget::Observation;
use qubit_budget::QuantityConversionError;
use qubit_budget::QuantityMeasurement;
use qubit_budget::json::JsonResource;
use qubit_json::decode::DiagnosticPolicy;
use qubit_json::decode::JsonDecodeErrorKind;
use qubit_json::decode::JsonDecoder;
use qubit_json::decode::JsonRootKind;
use qubit_json::decode::NormalizingJsonDecodePolicy;
use qubit_json::decode::NormalizingJsonDecoder;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataError;
use qubit_metadata::MetadataLimits;
#[cfg(feature = "schema")]
use qubit_metadata::MetadataSchema;
use qubit_metadata::MetadataWireDecodeError;

struct FailedRead;

impl io::Read for FailedRead {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "private-read-detail",
        ))
    }
}

#[test]
fn test_read_error_obeys_diagnostic_policy_and_preserves_source_chain() {
    for policy in [DiagnosticPolicy::Redacted, DiagnosticPolicy::Detailed] {
        let mut decoder = JsonDecoder::with_limits(MetadataLimits::default().json_decode())
            .with_diagnostic_policy(policy);
        let json_error = decoder
            .decode_input::<serde_json::Value, _>(&mut FailedRead)
            .expect_err("input read should fail");
        let error = MetadataWireDecodeError::from(json_error);

        let MetadataWireDecodeError::Read { kind, source } = &error else {
            panic!("read failures must retain their structured metadata variant");
        };
        assert_eq!(*kind, io::ErrorKind::PermissionDenied);
        assert_eq!(
            error.source().is_some(),
            policy == DiagnosticPolicy::Detailed
        );

        let display = error.to_string();
        let debug = format!("{error:?}");
        if policy == DiagnosticPolicy::Redacted {
            assert!(source.is_none());
            assert!(error.source().is_none());
            assert!(!display.contains("private-read-detail"));
            assert!(!debug.contains("private-read-detail"));
        } else {
            let source = source
                .as_ref()
                .expect("detailed diagnostics retain read source");
            assert!(source.to_string().contains("private-read-detail"));
            let chain_source = error.source().expect("detailed source chain");
            assert!(chain_source.to_string().contains("private-read-detail"));
            assert!(display.contains("PermissionDenied"));
        }
    }
}

#[cfg(feature = "filter")]
#[test]
fn test_filter_unsupported_version_retains_version_number() {
    use qubit_metadata::FilterLimits;
    use qubit_metadata::MetadataFilter;

    let input = br#"{"version":7,"expression":{"kind":"all"},"options":{"numeric_comparison_policy":"exact"}}"#;
    let error = MetadataFilter::decode_json_slice_with_limits(input, MetadataLimits::default(), FilterLimits::MAX)
        .expect_err("filter version");
    assert!(matches!(
        error,
        MetadataWireDecodeError::UnsupportedVersion { expected: 1, actual: 7 }
    ));
}

#[test]
fn test_budget_error_preserves_source_chain() {
    let error = MetadataWireDecodeError::Budget(BudgetError::LimitExceeded {
        resource: JsonResource::InputBytes,
        observed: Observation::Exact(5),
        maximum: 4,
    });
    assert!(error.to_string().contains("InputBytes"));
    assert!(error.source().is_some());
}

#[test]
fn test_invalid_json_preserves_source_chain() {
    let error = MetadataWireDecodeError::InvalidJson(
        serde_json::from_str::<serde_json::Value>("{").expect_err("input should be malformed"),
    );
    assert!(error.source().is_some());
}

#[test]
fn test_quantity_error_preserves_source_chain() {
    let source = QuantityConversionError::new(QuantityMeasurement::U64(9), "u8");
    let error = MetadataWireDecodeError::Quantity {
        resource: JsonResource::InputBytes,
        source,
    };
    assert!(error.to_string().contains("u8"));
    assert!(error.source().is_some());
}

#[test]
fn test_domain_error_exposes_structured_source_without_values() {
    let metadata = Metadata::new().with("private-key", "private-value");
    let input = serde_json::to_vec(&metadata).expect("wire");
    let limits = MetadataLimits::builder()
        .max_key_bytes(2)
        .build()
        .expect("limits should build");
    let error = Metadata::decode_json_slice_with_limits(&input, limits).expect_err("key limit");
    assert!(error.source().expect("domain source").is::<MetadataError>());
    let message = error.to_string();
    assert!(message.contains("KeyBytes"));
    assert!(!message.contains("private-key"));
    assert!(!message.contains("private-value"));
}

#[test]
fn test_unsupported_version_has_safe_diagnostic_and_no_synthetic_source() {
    let error = Metadata::decode_json_slice(br#"{"version":7,"values":{}}"#).expect_err("version");
    assert_eq!(error.to_string(), "Unsupported metadata wire version 7; expected 1");
    assert!(error.source().is_none());
}

#[cfg(feature = "schema")]
#[test]
fn test_schema_unsupported_version_retains_version_number() {
    let schema = MetadataSchema::default();
    let mut wire = serde_json::to_value(schema).expect("schema");
    wire["version"] = 7.into();
    let error =
        MetadataSchema::decode_json_slice(&serde_json::to_vec(&wire).expect("wire")).expect_err("schema version");
    assert!(matches!(
        error,
        MetadataWireDecodeError::UnsupportedVersion { expected: 1, actual: 7 }
    ));
}

#[test]
fn test_shared_deserialize_error_preserves_diagnostic_policy() {
    for policy in [DiagnosticPolicy::Redacted, DiagnosticPolicy::Detailed] {
        let mut decoder = JsonDecoder::with_limits(MetadataLimits::default().json_decode())
            .with_diagnostic_policy(policy);
        let shared = decoder
            .decode_str::<Metadata>(r#"{"values":{}}"#)
            .expect_err("version is required");
        assert_eq!(shared.kind(), JsonDecodeErrorKind::Deserialize);
        let error = MetadataWireDecodeError::from(shared);
        let MetadataWireDecodeError::Decode(shared) = &error else {
            panic!("shared deserialization facts must be preserved");
        };
        assert_eq!(shared.diagnostic_policy(), policy);
        assert_eq!(shared.kind(), JsonDecodeErrorKind::Deserialize);
        assert_eq!(shared.source().is_some(), policy == DiagnosticPolicy::Detailed);
        assert!(error
            .source()
            .expect("shared error wrapper")
            .is::<qubit_json::decode::JsonDecodeError>());
        if policy == DiagnosticPolicy::Redacted {
            assert!(!format!("{error:?}").contains("missing field"));
        } else {
            assert!(shared
                .source()
                .expect("original serde source")
                .to_string()
                .contains("version"));
        }
    }
}

#[test]
fn test_shared_invalid_utf8_retains_offset() {
    let mut decoder = JsonDecoder::with_limits(MetadataLimits::default().json_decode());
    let shared = decoder
        .decode_utf8::<serde_json::Value>(&[b' ', 0xff])
        .expect_err("invalid UTF-8");
    let MetadataWireDecodeError::Decode(shared) = MetadataWireDecodeError::from(shared) else {
        panic!("UTF-8 facts must be preserved");
    };
    assert_eq!(shared.kind(), JsonDecodeErrorKind::InvalidUtf8);
    assert_eq!(shared.utf8_valid_up_to(), Some(1));
    assert_eq!(shared.utf8_error_len(), Some(1));
    assert!(shared.source().is_none());
}

#[test]
fn test_shared_unexpected_top_level_retains_kinds() {
    let mut decoder = JsonDecoder::with_limits(MetadataLimits::default().json_decode());
    let shared = decoder
        .decode_object_str::<serde_json::Value>("[]")
        .expect_err("an object is required");
    let MetadataWireDecodeError::Decode(shared) = MetadataWireDecodeError::from(shared) else {
        panic!("top-level kinds must be preserved");
    };
    assert_eq!(shared.kind(), JsonDecodeErrorKind::UnexpectedTopLevel);
    assert_eq!(shared.expected_top_level(), Some(JsonRootKind::Object));
    assert_eq!(shared.actual_top_level(), Some(JsonRootKind::Array));
}

#[test]
fn test_shared_normalized_empty_input_retains_category() {
    let mut decoder = NormalizingJsonDecoder::with_limits(
        NormalizingJsonDecodePolicy::default(),
        MetadataLimits::default().json_decode(),
    );
    let shared = decoder
        .decode_str::<serde_json::Value>("")
        .expect_err("normalization leaves no document");
    assert_eq!(shared.kind(), JsonDecodeErrorKind::EmptyInput);
    let stage = shared.stage();
    let MetadataWireDecodeError::Decode(shared) = MetadataWireDecodeError::from(shared) else {
        panic!("empty-input category must be preserved");
    };
    assert_eq!(shared.kind(), JsonDecodeErrorKind::EmptyInput);
    assert_eq!(shared.stage(), stage);
    assert_eq!(shared.raw_input_bytes(), 0);
}
