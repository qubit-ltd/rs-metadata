// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::FilterLimitsBuilder`].

use qubit_metadata::FilterLimitKind;
use qubit_metadata::FilterLimits;
use qubit_metadata::MetadataError;

#[test]
fn test_build_rejects_each_zero_limit() {
    for result in [
        (
            "max_depth",
            FilterLimitKind::Depth,
            FilterLimits::builder().max_depth(0).build(),
        ),
        (
            "max_nodes",
            FilterLimitKind::Nodes,
            FilterLimits::builder().max_nodes(0).build(),
        ),
        (
            "max_set_values",
            FilterLimitKind::SetValues,
            FilterLimits::builder().max_set_values(0).build(),
        ),
        (
            "max_key_bytes",
            FilterLimitKind::KeyBytes,
            FilterLimits::builder().max_key_bytes(0).build(),
        ),
    ] {
        let (limit_name, kind, result) = result;
        assert!(
            matches!(
                result,
                Err(MetadataError::InvalidFilterLimit {
                    kind: actual_kind,
                    value: 0,
                    ..
                }) if actual_kind == kind
            ),
            "expected zero {limit_name} to be rejected with {kind:?} limit kind"
        );
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
