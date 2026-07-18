// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::MetadataField`].

use qubit_datatype::DataType;
use qubit_metadata::MetadataField;

#[test]
fn test_metadata_field_new_assigns_values() {
    let field = MetadataField::new(DataType::String, true);
    assert_eq!(field.data_type(), DataType::String);
    assert!(field.is_required());
}
