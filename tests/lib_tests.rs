// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the crate's public exports.

use qubit_metadata::{FilterExpression, Metadata, MetadataFilter};

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
