// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::MetadataWireLimitKind`].

use qubit_metadata::MetadataWireLimitKind;

#[test]
fn test_metadata_wire_limit_kind_variants_are_distinct() {
    let kinds = [MetadataWireLimitKind::Entries, MetadataWireLimitKind::KeyBytes];
    assert_eq!(kinds.len(), 2);
    assert_ne!(kinds[0], kinds[1]);
    assert_eq!(format!("{:?}", kinds[1]), "KeyBytes");
}
