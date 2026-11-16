// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_metadata::Metadata;
use qubit_metadata::MetadataLimits;
use qubit_metadata::MetadataWireDecodeError;

pub fn main() -> Result<(), MetadataWireDecodeError> {
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
        metadata.get_ref::<str>("tenant_id").expect("tenant_id is a string"),
        "acme"
    );
    Ok(())
}
