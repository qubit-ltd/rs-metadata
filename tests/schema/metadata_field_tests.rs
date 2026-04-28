/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use qubit_common::DataType;
use qubit_metadata::MetadataField;

#[test]
fn metadata_field_new_assigns_values() {
    let field = MetadataField::new(DataType::String, true);
    assert_eq!(field.data_type(), DataType::String);
    assert!(field.is_required());
}
