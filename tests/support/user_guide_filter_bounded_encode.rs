// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_metadata::FilterExpression;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataFilter;
use qubit_metadata::default_json_encode_limits;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let expression = FilterExpression::builder().eq("tenant", "acme").build()?;
    let filter = MetadataFilter::builder().expression(expression).build()?;
    let bytes = filter.to_json_vec_with_limits(default_json_encode_limits())?;
    let restored = MetadataFilter::decode_json_slice(&bytes)?;
    assert!(restored.matches(&Metadata::new().with("tenant", "acme")));
    let mut output = Vec::new();
    filter.to_json_writer_with_limits(&mut output, default_json_encode_limits())?;
    assert_eq!(output, bytes);
    Ok(())
}
