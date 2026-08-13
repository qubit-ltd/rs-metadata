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

use qubit_budget::json::JsonResource;
use qubit_budget::{
    BudgetError,
    Observation,
    QuantityConversionError,
    QuantityMeasurement,
};
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
        serde_json::from_str::<serde_json::Value>("{")
            .expect_err("input should be malformed"),
    );
    assert!(error.source().is_some());
}

#[test]
fn test_quantity_error_preserves_source_chain() {
    let source =
        QuantityConversionError::new(QuantityMeasurement::U64(9), "u8");
    let error = MetadataWireDecodeError::Quantity {
        resource: JsonResource::InputBytes,
        source,
    };
    assert!(error.to_string().contains("u8"));
    assert!(error.source().is_some());
}
