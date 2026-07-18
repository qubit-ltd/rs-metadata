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
fn test_filter_limits_new_exposes_configured_bounds() {
    let limits = FilterLimits::new(4, 8, 16, 32);

    assert_eq!(limits.max_depth(), 4);
    assert_eq!(limits.max_nodes(), 8);
    assert_eq!(limits.max_set_values(), 16);
    assert_eq!(limits.max_key_length(), 32);
}

#[test]
fn test_filter_limits_default_has_positive_bounds() {
    let limits = FilterLimits::default();

    assert!(limits.max_depth() > 0);
    assert!(limits.max_nodes() > 0);
    assert!(limits.max_set_values() > 0);
    assert!(limits.max_key_length() > 0);
}
