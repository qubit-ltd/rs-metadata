/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use qubit_common::DataType;
use qubit_metadata::{MetadataSchemaBuilder, UnknownFieldPolicy};

#[test]
fn schema_builder_default_builds_empty_schema() {
    let schema = MetadataSchemaBuilder::default()
        .unknown_field_policy(UnknownFieldPolicy::Allow)
        .optional("id", DataType::String)
        .build();
    assert_eq!(schema.field_type("id"), Some(DataType::String));
}
