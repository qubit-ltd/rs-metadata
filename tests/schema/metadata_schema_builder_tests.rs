// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::MetadataSchemaBuilder`].

use qubit_datatype::DataType;
use qubit_metadata::{
    MetadataError,
    MetadataField,
    MetadataSchemaBuilder,
    UnknownFilterFieldPolicy,
    UnknownMetadataFieldPolicy,
};

#[test]
fn test_schema_builder_default_builds_empty_schema() {
    let schema = MetadataSchemaBuilder::default()
        .unknown_metadata_field_policy(UnknownMetadataFieldPolicy::Allow)
        .unknown_filter_field_policy(UnknownFilterFieldPolicy::AllowUnchecked)
        .optional("id", DataType::String)
        .build()
        .expect("schema should build");
    assert_eq!(schema.field_type("id"), Some(DataType::String));
}

#[test]
fn test_schema_builder_rejects_duplicate_field_declarations() {
    let error = MetadataSchemaBuilder::default()
        .required("id", DataType::String)
        .optional("id", DataType::Int64)
        .build()
        .expect_err("duplicate schema declarations must be rejected");

    assert_eq!(
        error,
        MetadataError::DuplicateSchemaField {
            key: "id".to_string(),
        }
    );
}

#[test]
fn test_schema_builder_replace_field_explicitly_overwrites_declaration() {
    let schema = MetadataSchemaBuilder::default()
        .required("id", DataType::String)
        .replace_field("id", MetadataField::new(DataType::Int64, false))
        .build()
        .expect("explicit replacement should build");

    assert_eq!(schema.field_type("id"), Some(DataType::Int64));
    assert!(!schema.field("id").unwrap().is_required());
}
