// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonResource;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataLimits;
use qubit_metadata::default_json_encode_limits;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let encode = default_json_encode_limits()
        .into_builder()
        .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 128 * 1024))
        .build();
    let limits = MetadataLimits::builder().json_encode(encode).build()?;
    let metadata = Metadata::new()
        .with("tenant_id", "acme")
        .with("document_id", "doc-42")
        .with("chunk_index", 3_i64);

    let bytes = metadata.to_json_vec_with_limits(limits.json_encode())?;
    // The application passes these bytes to its own storage or transport layer.
    let restored = Metadata::decode_json_slice_with_limits(&bytes, limits)?;
    assert_eq!(restored, metadata);

    let mut output = Vec::new();
    metadata.to_json_writer_with_limits(&mut output, limits.json_encode())?;
    assert_eq!(output, bytes);
    Ok(())
}
