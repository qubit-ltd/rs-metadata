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

use qubit_budget::BudgetError;
use qubit_budget::Observation;
use qubit_budget::QuantityConversionError;
use qubit_budget::QuantityMeasurement;
use qubit_budget::json::JsonResource;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataError;
use qubit_metadata::MetadataLimits;
#[cfg(feature = "schema")]
use qubit_metadata::MetadataSchema;
use qubit_metadata::MetadataWireDecodeError;

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
    let error = Metadata::decode_json_slice_with_limits(&input, MetadataLimits::builder().max_key_bytes(2).build())
        .expect_err("key limit");
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
