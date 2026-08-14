// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the crate's public exports.

#[cfg(feature = "filter")]
use qubit_metadata::FilterExpression;
use qubit_metadata::Metadata;
#[cfg(feature = "filter")]
use qubit_metadata::MetadataFilter;

#[cfg(feature = "filter")]
#[test]
fn test_public_exports_are_usable() {
    let meta = Metadata::new().with("k", "v");
    let expression = FilterExpression::builder().eq("k", "v").build().unwrap();
    let filter = MetadataFilter::builder()
        .expression(expression)
        .build()
        .unwrap();
    assert!(filter.matches(&meta));
}

#[cfg(not(feature = "filter"))]
#[test]
fn test_core_metadata_is_usable_without_filter_feature() {
    let metadata = Metadata::new().with("key", "value");

    assert_eq!(metadata.get::<String>("key"), Some("value".to_string()));
}
