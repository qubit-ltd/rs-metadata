/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

use qubit_datatype::DataType;
use qubit_metadata::{MetadataSchemaBuilder, UnknownFieldPolicy};

#[test]
fn schema_builder_default_builds_empty_schema() {
    let schema = MetadataSchemaBuilder::default()
        .unknown_field_policy(UnknownFieldPolicy::Allow)
        .optional("id", DataType::String)
        .build();
    assert_eq!(schema.field_type("id"), Some(DataType::String));
}
