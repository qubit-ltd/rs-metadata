// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::MetadataWireDecodeError`].

#[cfg(feature = "json")]
use qubit_metadata::MetadataWireDecodeError;

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
