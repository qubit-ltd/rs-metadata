// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Integration tests for examples documented in the README and user guide.

#![cfg(feature = "json")]

use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonResource;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataLimits;
use qubit_metadata::MetadataWireDecodeError;
use qubit_metadata::MetadataWireEncodeError;
use qubit_metadata::default_json_decode_limits;
use qubit_metadata::default_json_encode_limits;

mod support {
    pub mod user_guide_bounded_decode;
}

#[test]
fn test_readme_directional_limits_example() {
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
        .build()
        .expect("limits should build");
    assert_eq!(limits.json_decode().max_input_bytes(), Some(64 * 1024));
    assert_eq!(limits.json_encode().max_output_bytes(), Some(128 * 1024));
}

#[test]
fn test_user_guide_domain_limits_example() -> Result<(), MetadataWireDecodeError> {
    support::user_guide_bounded_decode::main()
}

#[test]
fn test_user_guide_bounded_decode_source_matches() {
    let expected = include_str!("support/user_guide_bounded_decode.rs").trim();
    let start = "<!-- bounded-json-decoding-example:start -->";
    let end = "<!-- bounded-json-decoding-example:end -->";
    for (name, guide) in [
        ("English", include_str!("../doc/user_guide.md")),
        ("Chinese", include_str!("../doc/user_guide.zh_CN.md")),
    ] {
        assert_eq!(guide.matches(start).count(), 1, "{name}: start marker");
        assert_eq!(guide.matches(end).count(), 1, "{name}: end marker");
        let (_, after_start) = guide.split_once(start).expect("start marker");
        let (block, _) = after_start.split_once(end).expect("end marker after start");
        let code = block
            .trim()
            .strip_prefix("```rust")
            .and_then(|value| value.strip_suffix("```"))
            .expect("one fenced Rust block")
            .trim();
        assert_eq!(code, expected, "{name}: guide must match compiled example");
    }
}

#[test]
fn test_user_guide_bounded_json_encoding_example() {
    let encode = default_json_encode_limits()
        .into_builder()
        .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 128 * 1024))
        .build();
    let limits = MetadataLimits::builder()
        .json_encode(encode)
        .build()
        .expect("limits should build");
    let metadata = Metadata::new()
        .with("tenant_id", "acme")
        .with("document_id", "doc-42")
        .with("chunk_index", 3_i64);

    let bytes = metadata
        .to_json_vec_with_limits(limits.json_encode())
        .expect("metadata should encode");
    let restored = Metadata::decode_json_slice_with_limits(&bytes, limits).expect("encoded metadata should decode");
    assert_eq!(restored, metadata);

    let mut output = Vec::new();
    metadata
        .to_json_writer_with_limits(&mut output, limits.json_encode())
        .expect("metadata should write");
    assert_eq!(output, bytes);
}

#[test]
fn test_user_guide_output_budget_rejects_metadata() {
    let limits = default_json_encode_limits()
        .into_builder()
        .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 1))
        .build();
    let metadata = Metadata::new().with("tenant_id", "acme");

    assert!(matches!(
        metadata.to_json_vec_with_limits(limits),
        Err(MetadataWireEncodeError::Budget(_))
    ));
}
