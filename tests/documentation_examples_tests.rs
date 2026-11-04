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
use qubit_metadata::MetadataWireDecodeError;
use qubit_metadata::MetadataWireEncodeError;
use qubit_metadata::default_json_encode_limits;

mod support {
    pub mod readme_directional_limits;
    pub mod user_guide_bounded_decode;
    pub mod user_guide_bounded_encode;
    #[cfg(feature = "filter")]
    pub mod user_guide_filter_bounded_encode;
}

#[test]
fn test_readme_directional_limits_example() {
    support::readme_directional_limits::main().expect("README example should run");
}

#[test]
fn test_user_guide_domain_limits_example() -> Result<(), MetadataWireDecodeError> {
    support::user_guide_bounded_decode::main()
}

#[test]
fn test_user_guide_bounded_decode_source_matches() {
    let package_manifest = include_str!("../Cargo.toml");
    assert!(package_manifest.contains("version = \"0.7.0\""));
    for (name, document) in [
        ("English README", include_str!("../README.md")),
        ("Chinese README", include_str!("../README.zh_CN.md")),
        ("English guide", include_str!("../doc/user_guide.md")),
        ("Chinese guide", include_str!("../doc/user_guide.zh_CN.md")),
    ] {
        assert!(
            document.contains("qubit-metadata = { version = \"0.7\", features = [\"json\"] }"),
            "{name}: metadata json dependency"
        );
        assert!(
            document.contains("qubit-budget = { version = \"0.10\", features = [\"json\"] }"),
            "{name}: budget json dependency"
        );
        if name.contains("README") {
            assert_documented_example(
                name,
                document,
                "directional-json-limits-example",
                include_str!("support/readme_directional_limits.rs"),
            );
        } else {
            assert_documented_example(
                name,
                document,
                "bounded-json-decoding-example",
                include_str!("support/user_guide_bounded_decode.rs"),
            );
            assert_documented_example(
                name,
                document,
                "bounded-json-encoding-example",
                include_str!("support/user_guide_bounded_encode.rs"),
            );
            #[cfg(feature = "filter")]
            assert_documented_example(
                name,
                document,
                "bounded-filter-json-encoding-example",
                include_str!("support/user_guide_filter_bounded_encode.rs"),
            );
        }
    }
}

#[cfg(feature = "filter")]
#[test]
fn test_user_guide_filter_bounded_json_encoding_example() {
    support::user_guide_filter_bounded_encode::main().expect("filter encoding guide example should run");
}

fn assert_documented_example(name: &str, document: &str, marker: &str, expected: &str) {
    let start = format!("<!-- {marker}:start -->");
    let end = format!("<!-- {marker}:end -->");
    assert_eq!(document.matches(&start).count(), 1, "{name}: start marker for {marker}");
    assert_eq!(document.matches(&end).count(), 1, "{name}: end marker for {marker}");
    let start_index = document.find(&start).expect("start marker");
    let end_index = document.find(&end).expect("end marker");
    assert!(start_index < end_index, "{name}: markers out of order for {marker}");
    let block = &document[start_index + start.len()..end_index];
    assert_eq!(
        block.matches("```rust").count(),
        1,
        "{name}: one Rust block for {marker}"
    );
    assert_eq!(
        block.matches("```").count(),
        2,
        "{name}: no extra fenced blocks for {marker}"
    );
    let code = block
        .trim()
        .strip_prefix("```rust")
        .and_then(|value| value.strip_suffix("```"))
        .expect("one fenced Rust block")
        .trim();
    assert_eq!(code, expected.trim(), "{name}: {marker} must match compiled example");
}

#[test]
fn test_user_guide_bounded_json_encoding_example() {
    support::user_guide_bounded_encode::main().expect("encoding guide example should run");
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
