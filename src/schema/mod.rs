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

mod builder;
mod field;
mod definition;
mod filter_validation;
mod unknown_field_policy;

pub use builder::MetadataSchemaBuilder;
pub use field::MetadataField;
pub use definition::MetadataSchema;
pub use unknown_field_policy::UnknownFieldPolicy;
