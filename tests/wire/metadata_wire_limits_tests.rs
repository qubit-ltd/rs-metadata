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
