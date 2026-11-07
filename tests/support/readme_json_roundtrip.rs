// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// readme-json-roundtrip-example:start
use qubit_metadata::Metadata;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let metadata = Metadata::new().with("tenant_id", "acme");
    let bytes = metadata.to_json_vec()?;
    let decoded = Metadata::decode_json_slice(&bytes)?;
    assert_eq!(decoded.get_ref::<str>("tenant_id")?, "acme");
    Ok(())
}
// readme-json-roundtrip-example:end
