// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use std::error::Error;

use qubit_metadata::Metadata;
use qubit_metadata::MetadataError;
use qubit_metadata::MetadataLimitKind;
use qubit_metadata::MetadataLimits;
use qubit_metadata::MetadataWireDecodeError;

pub fn main() -> Result<(), MetadataWireDecodeError> {
    // Zero is valid; the protocol maximum is valid too.
    let _valid_limits = MetadataLimits::builder()
        .max_metadata_entries(0)
        .max_schema_fields(4_096)
        .max_key_bytes(256)
        .build()
        .expect("zero and protocol-maximum caps are valid");

    // Match configuration failures by category and retain both numbers.
    let error = MetadataLimits::builder()
        .max_schema_fields(usize::MAX)
        .build()
        .expect_err("an over-limit schema cap must fail");
    assert!(matches!(
        error,
        MetadataError::InvalidMetadataLimit {
            kind: MetadataLimitKind::SchemaFields,
            value: usize::MAX,
            maximum: 4_096,
        }
    ));

    let invalid_limits = MetadataLimits::builder()
        .max_metadata_entries(4_097)
        .build()
        .map_err(MetadataWireDecodeError::InvalidLimits)
        .expect_err("an invalid configuration is retained as a wire error");
    let source = Error::source(&invalid_limits).expect("InvalidLimits retains its source");
    let structured = source
        .downcast_ref::<MetadataError>()
        .expect("the source is the original MetadataError");
    assert!(matches!(
        structured,
        MetadataError::InvalidMetadataLimit {
            kind: MetadataLimitKind::MetadataEntries,
            value: 4_097,
            maximum: 4_096,
        }
    ));

    let limits = MetadataLimits::builder()
        .max_metadata_entries(128)
        .max_key_bytes(128)
        .build()
        .map_err(MetadataWireDecodeError::InvalidLimits)?;
    let metadata = Metadata::decode_json_slice_with_limits(
        br#"{"version":1,"values":{"tenant_id":{"scalar":{"string":"acme"}}}}"#,
        limits,
    )?;
    assert_eq!(
        metadata
            .get_ref::<str>("tenant_id")
            .expect("tenant_id is a string"),
        "acme"
    );
    Ok(())
}
