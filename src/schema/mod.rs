/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Metadata schema module.
//!
//! Encapsulates schema concepts (schema definition, field definition, builder,
//! and unknown-field policy) that are closely related and often used together.

mod filter_validation;
mod metadata_field;
mod metadata_schema;
mod metadata_schema_builder;
mod unknown_field_policy;

pub use metadata_field::MetadataField;
pub use metadata_schema::MetadataSchema;
pub use metadata_schema_builder::MetadataSchemaBuilder;
pub use unknown_field_policy::UnknownFieldPolicy;
