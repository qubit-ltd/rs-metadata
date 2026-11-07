// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// user-guide-directional-limits-example:start
use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonResource;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataLimits;
use qubit_metadata::default_json_decode_limits;
use qubit_metadata::default_json_encode_limits;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let decode = default_json_decode_limits()
        .into_builder()
        .input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, 64 * 1024))
        .build();
    let encode = default_json_encode_limits()
        .into_builder()
        .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 128 * 1024))
        .build();
    let limits = MetadataLimits::builder()
        .json_decode(decode)
        .json_encode(encode)
        .build()?;
    assert_eq!(limits.json_decode().max_input_bytes(), Some(64 * 1024));
    assert_eq!(limits.json_encode().max_output_bytes(), Some(128 * 1024));

    let metadata = Metadata::new().with("tenant_id", "acme");
    let bytes = metadata.to_json_vec_with_limits(limits.json_encode())?;
    assert!(!bytes.is_empty());
    Ok(())
}
// user-guide-directional-limits-example:end
