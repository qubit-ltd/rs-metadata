// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::FilterLimits`].

use qubit_metadata::FilterLimits;

#[test]
fn test_filter_limits_builder_exposes_configured_bounds() {
    let limits = FilterLimits::builder()
        .max_depth(4)
        .max_nodes(8)
        .max_set_values(16)
        .max_key_bytes(32)
        .build()
        .expect("configured limits should be valid");

    assert_eq!(limits.max_depth(), 4);
    assert_eq!(limits.max_nodes(), 8);
    assert_eq!(limits.max_set_values(), 16);
    assert_eq!(limits.max_key_bytes(), 32);
}

#[test]
fn test_filter_limits_default_equals_hard_maximum() {
    assert_eq!(FilterLimits::default(), FilterLimits::MAX);
}

#[test]
fn test_filter_limits_builder_rejects_zero_and_values_above_hard_maximum() {
    assert!(FilterLimits::builder().max_depth(0).build().is_err());
    assert!(
        FilterLimits::builder()
            .max_nodes(FilterLimits::MAX.max_nodes() + 1)
            .build()
            .is_err()
    );
}
