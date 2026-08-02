// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the bounded public JSON decoding APIs.

use qubit_metadata::{
    FilterLimits,
    Metadata,
    MetadataFilter,
    MetadataSchema,
    MetadataWireDecodeError,
    MetadataWireLimits,
};

#[test]
fn test_decode_json_slice_with_limits_rejects_oversized_input_before_parsing() {
    let limits = MetadataWireLimits::new(4);

    assert!(matches!(
        Metadata::decode_json_slice_with_limits(b"null!", limits),
        Err(MetadataWireDecodeError::InputTooLarge {
            input_bytes: 5,
            max_input_bytes: 4,
        })
    ));
    assert!(matches!(
        MetadataSchema::decode_json_slice_with_limits(b"null!", limits),
        Err(MetadataWireDecodeError::InputTooLarge {
            input_bytes: 5,
            max_input_bytes: 4,
        })
    ));

    assert!(matches!(
        MetadataFilter::decode_json_slice_with_limits(
            b"null!",
            limits,
            FilterLimits::MAX,
        ),
        Err(MetadataWireDecodeError::InputTooLarge {
            input_bytes: 5,
            max_input_bytes: 4,
        })
    ));
}

#[test]
fn test_decode_json_slice_with_limits_accepts_the_exact_byte_boundary() {
    let limits = MetadataWireLimits::new(4);

    assert!(matches!(
        Metadata::decode_json_slice_with_limits(b"null", limits),
        Err(MetadataWireDecodeError::InvalidJson(_))
    ));
    assert!(matches!(
        MetadataSchema::decode_json_slice_with_limits(b"null", limits),
        Err(MetadataWireDecodeError::InvalidJson(_))
    ));

    assert!(matches!(
        MetadataFilter::decode_json_slice_with_limits(
            b"null",
            limits,
            FilterLimits::MAX,
        ),
        Err(MetadataWireDecodeError::InvalidJson(_))
    ));
}

#[test]
fn test_decode_json_slice_with_limits_rejects_excessive_metadata_and_schema_entries()
 {
    let metadata = Metadata::new().with("first", 1_i64).with("second", 2_i64);
    let metadata_json = serde_json::to_vec(&metadata).unwrap();
    let metadata_limits = MetadataWireLimits::new(metadata_json.len())
        .with_max_metadata_entries(1);

    assert!(matches!(
        Metadata::decode_json_slice_with_limits(
            &metadata_json,
            metadata_limits
        ),
        Err(MetadataWireDecodeError::LimitExceeded {
            kind: qubit_metadata::MetadataWireLimitKind::MetadataEntries,
            value: 2,
            maximum: 1,
        })
    ));
    let key_limits =
        MetadataWireLimits::new(metadata_json.len()).with_max_key_bytes(4);
    assert!(matches!(
        Metadata::decode_json_slice_with_limits(&metadata_json, key_limits),
        Err(MetadataWireDecodeError::LimitExceeded {
            kind: qubit_metadata::MetadataWireLimitKind::KeyBytes,
            value: 5,
            maximum: 4,
        })
    ));

    let schema = MetadataSchema::builder()
        .required("first", qubit_datatype::DataType::Int64)
        .required("second", qubit_datatype::DataType::Int64)
        .build()
        .unwrap();
    let schema_json = serde_json::to_vec(&schema).unwrap();
    let schema_limits =
        MetadataWireLimits::new(schema_json.len()).with_max_schema_fields(1);

    assert!(matches!(
        MetadataSchema::decode_json_slice_with_limits(
            &schema_json,
            schema_limits
        ),
        Err(MetadataWireDecodeError::LimitExceeded {
            kind: qubit_metadata::MetadataWireLimitKind::SchemaFields,
            value: 2,
            maximum: 1,
        })
    ));
}
