// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
//
// =============================================================================
//! Tests for [`qubit_metadata::FilterLimitKind`].

use qubit_metadata::FilterLimitKind;

#[test]
fn test_filter_limit_kind_variants_are_distinct() {
    let kinds = [
        FilterLimitKind::Depth,
        FilterLimitKind::Nodes,
        FilterLimitKind::SetValues,
        FilterLimitKind::KeyBytes,
    ];
    assert_eq!(kinds.len(), 4);
    assert_ne!(kinds[0], kinds[1]);
    assert_eq!(format!("{:?}", kinds[3]), "KeyBytes");
}
