// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for bounded metadata JSON encoding errors.

#![cfg(feature = "json")]

use std::error::Error;

use qubit_budget::BudgetError;
use qubit_budget::MeasuredBudgetError;
use qubit_budget::QuantityConversionError;
use qubit_budget::json::JsonResource;
use qubit_json::text::JsonEncodeError;
use qubit_json::text::JsonSyntaxError;
use qubit_json::text::JsonSyntaxErrorReason;
use qubit_metadata::MetadataWireEncodeError;

#[test]
fn test_budget_error_preserves_source_chain() {
    let source = BudgetError::Insufficient {
        resource: JsonResource::OutputBytes,
        limit: 4,
        remaining: 0,
        requested: 1,
    };
    let error = MetadataWireEncodeError::Budget(source);

    assert!(error.source().is_some());
    assert!(error.to_string().contains("OutputBytes"));
}

#[test]
fn test_metadata_wire_encode_error_formats_all_variants() {
    let syntax = JsonSyntaxError::new(
        2,
        1,
        3,
        JsonSyntaxErrorReason::UnexpectedByte { byte: b'!' },
    );
    let quantity = QuantityConversionError::new(
        qubit_budget::QuantityMeasurement::U64(9),
        "u8",
    );
    let errors = [
        MetadataWireEncodeError::Budget(BudgetError::Insufficient {
            resource: JsonResource::OutputBytes,
            limit: 4,
            remaining: 0,
            requested: 1,
        }),
        MetadataWireEncodeError::Quantity {
            resource: JsonResource::OutputBytes,
            source: quantity,
        },
        MetadataWireEncodeError::Syntax(syntax),
        MetadataWireEncodeError::Json(serde_json::Error::io(
            std::io::Error::other("serialize"),
        )),
        MetadataWireEncodeError::Io(std::io::Error::other("write")),
    ];

    for error in errors {
        assert!(!error.to_string().is_empty());
        assert!(error.source().is_some());
    }
}

#[test]
fn test_metadata_wire_encode_error_conversion_covers_shared_encoder_errors() {
    let syntax =
        JsonSyntaxError::new(0, 1, 1, JsonSyntaxErrorReason::UnexpectedEnd);
    let errors = [
        MetadataWireEncodeError::from(JsonEncodeError::InvalidRawJson(syntax)),
        MetadataWireEncodeError::from(JsonEncodeError::Serialize(
            serde_json::Error::io(std::io::Error::other("serialize")),
        )),
        MetadataWireEncodeError::from(JsonEncodeError::Write(
            std::io::Error::other("write"),
        )),
        MetadataWireEncodeError::from(JsonEncodeError::Budget(
            MeasuredBudgetError::Budget(BudgetError::Insufficient {
                resource: JsonResource::OutputBytes,
                limit: 4,
                remaining: 0,
                requested: 1,
            }),
        )),
        MetadataWireEncodeError::from(JsonEncodeError::Budget(
            MeasuredBudgetError::Quantity {
                resource: JsonResource::OutputBytes,
                source: QuantityConversionError::new(
                    qubit_budget::QuantityMeasurement::U64(9),
                    "u8",
                ),
            },
        )),
    ];

    assert!(matches!(errors[0], MetadataWireEncodeError::Syntax(_)));
    assert!(matches!(errors[1], MetadataWireEncodeError::Json(_)));
    assert!(matches!(errors[2], MetadataWireEncodeError::Io(_)));
    assert!(matches!(errors[3], MetadataWireEncodeError::Budget(_)));
    assert!(matches!(
        errors[4],
        MetadataWireEncodeError::Quantity { .. }
    ));
}
