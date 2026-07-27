// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::FilterLimitsBuilder`].

use qubit_metadata::{
    FilterLimits,
    MetadataError,
};

#[test]
fn test_build_rejects_each_zero_limit() {
    for result in [
        FilterLimits::builder().max_depth(0).build(),
        FilterLimits::builder().max_nodes(0).build(),
        FilterLimits::builder().max_set_values(0).build(),
        FilterLimits::builder().max_key_bytes(0).build(),
    ] {
        assert!(matches!(
            result,
            Err(MetadataError::InvalidFilterLimit { value: 0, .. })
        ));
    }
}

#[test]
fn test_build_uses_last_values() {
    let limits = FilterLimits::builder()
        .max_depth(2)
        .max_depth(3)
        .max_nodes(4)
        .max_set_values(5)
        .max_key_bytes(6)
        .build()
        .expect("limits should build");
    assert_eq!(limits.max_depth(), 3);
    assert_eq!(limits.max_nodes(), 4);
    assert_eq!(limits.max_set_values(), 5);
    assert_eq!(limits.max_key_bytes(), 6);
}
