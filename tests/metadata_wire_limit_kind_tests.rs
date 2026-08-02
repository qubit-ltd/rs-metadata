// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::MetadataWireLimitKind`].

#![cfg(feature = "json")]

use qubit_metadata::MetadataWireLimitKind;

#[test]
fn test_metadata_wire_limit_kind_variants_are_distinct() {
    assert_ne!(
        MetadataWireLimitKind::MetadataEntries,
        MetadataWireLimitKind::SchemaFields
    );
    assert_ne!(
        MetadataWireLimitKind::MetadataEntries,
        MetadataWireLimitKind::KeyBytes
    );
    assert_ne!(
        MetadataWireLimitKind::SchemaFields,
        MetadataWireLimitKind::KeyBytes
    );
}
