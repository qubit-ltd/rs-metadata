// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::MetadataWireDecodeError`].

#[cfg(feature = "json")]
use qubit_metadata::{
    MetadataWireDecodeError,
    MetadataWireLimitKind,
};

#[cfg(feature = "json")]
#[test]
fn test_metadata_wire_decode_error_describes_input_limit() {
    let error = MetadataWireDecodeError::InputTooLarge {
        input_bytes: 5,
        max_input_bytes: 4,
    };

    assert_eq!(
        error.to_string(),
        "metadata JSON input has 5 bytes, exceeding the limit of 4 bytes"
    );
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
#[cfg(feature = "json")]
fn test_metadata_wire_decode_error_describes_decoded_resource_limit() {
    let error = MetadataWireDecodeError::LimitExceeded {
        kind: MetadataWireLimitKind::MetadataEntries,
        value: 2,
        maximum: 1,
    };

    assert_eq!(
        error.to_string(),
        "metadata JSON MetadataEntries value 2 exceeds the limit of 1"
    );
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
#[cfg(all(feature = "json", feature = "filter"))]
fn test_metadata_wire_decode_error_preserves_filter_contract_failure() {
    let error = MetadataWireDecodeError::Filter(
        qubit_metadata::MetadataError::MissingFilterExpression,
    );

    assert_eq!(error.to_string(), "Metadata filter requires an expression");
    assert!(std::error::Error::source(&error).is_some());
}
