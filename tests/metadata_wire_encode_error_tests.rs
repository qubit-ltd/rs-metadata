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
use std::io;
use std::io::Write;

use qubit_budget::BudgetError;
use qubit_budget::MeasuredBudgetError;
use qubit_budget::QuantityConversionError;
use qubit_budget::QuantityMeasurement;
use qubit_budget::json::JsonResource;
use qubit_json::decode::JsonSyntaxError;
use qubit_json::decode::JsonSyntaxErrorReason;
use qubit_json::encode::JsonEncodeError;
use qubit_json::encode::JsonEncoder;
use qubit_json::encode::JsonSerializationError;
use qubit_json::encode::JsonSerializationErrorKind;
use qubit_metadata::MetadataWireEncodeError;
use serde::Serialize;
use serde::Serializer;
use serde::ser::SerializeStruct;

struct InvalidRawValue;

impl Serialize for InvalidRawValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let token = concat!("$", "serde_json", ":", ":private::RawValue");
        let mut state = serializer.serialize_struct(token, 1)?;
        state.serialize_field(token, "[")?;
        state.end()
    }
}

struct RejectingWriter;

impl Write for RejectingWriter {
    fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("rejected"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

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
    let syntax = JsonSyntaxError::new(2, 1, 3, JsonSyntaxErrorReason::UnexpectedByte { byte: b'!' });
    let quantity = QuantityConversionError::new(QuantityMeasurement::U64(9), "u8");
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
        MetadataWireEncodeError::Json(JsonSerializationError::new(
            JsonSerializationErrorKind::CustomSerialization,
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
    let serialization = JsonEncoder::unlimited()
        .to_vec(&u128::MAX)
        .expect_err("wide integer must fail JSON serialization");
    let invalid_raw = JsonEncoder::unlimited()
        .to_vec(&InvalidRawValue)
        .expect_err("invalid RawValue text must fail");
    let writer = JsonEncoder::unlimited()
        .write_buffered(RejectingWriter, &true)
        .expect_err("rejecting writer must fail");
    let errors = [
        MetadataWireEncodeError::from(serialization),
        MetadataWireEncodeError::from(JsonEncodeError::from(MeasuredBudgetError::Budget(
            BudgetError::Insufficient {
                resource: JsonResource::OutputBytes,
                limit: 4,
                remaining: 0,
                requested: 1,
            },
        ))),
        MetadataWireEncodeError::from(JsonEncodeError::from(MeasuredBudgetError::Quantity {
            resource: JsonResource::OutputBytes,
            source: QuantityConversionError::new(QuantityMeasurement::U64(9), "u8"),
        })),
        MetadataWireEncodeError::from(invalid_raw),
        MetadataWireEncodeError::from(writer),
    ];

    assert!(matches!(errors[0], MetadataWireEncodeError::Json(_)));
    assert!(matches!(errors[1], MetadataWireEncodeError::Budget(_)));
    assert!(matches!(errors[2], MetadataWireEncodeError::Quantity { .. }));
    assert!(matches!(errors[3], MetadataWireEncodeError::Syntax(_)));
    assert!(matches!(errors[4], MetadataWireEncodeError::Io(_)));
}
