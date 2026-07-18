// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! [`MetadataSchemaBuilder`] — fluent schema construction API.

use std::collections::BTreeMap;

use qubit_datatype::DataType;

use crate::schema::{
    MetadataField,
    MetadataSchema,
    UnknownFieldPolicy,
};

/// Builder for [`MetadataSchema`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MetadataSchemaBuilder {
    /// Field definitions being built.
    fields: BTreeMap<String, MetadataField>,
    /// Unknown-field policy copied into the built schema.
    unknown_field_policy: UnknownFieldPolicy,
}

impl MetadataSchemaBuilder {
    /// Adds a required field definition.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to declare.
    /// * `data_type` - Concrete data type accepted by the field.
    ///
    /// # Returns
    ///
    /// The updated builder.
    #[inline(always)]
    #[must_use]
    pub fn required(mut self, key: &str, data_type: DataType) -> Self {
        let _ = self
            .fields
            .insert(key.to_string(), MetadataField::new(data_type, true));
        self
    }

    /// Adds an optional field definition.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to declare.
    /// * `data_type` - Concrete data type accepted by the field.
    ///
    /// # Returns
    ///
    /// The updated builder.
    #[inline(always)]
    #[must_use]
    pub fn optional(mut self, key: &str, data_type: DataType) -> Self {
        let _ = self
            .fields
            .insert(key.to_string(), MetadataField::new(data_type, false));
        self
    }

    /// Sets the policy for metadata keys not declared by the schema.
    ///
    /// # Parameters
    ///
    /// * `policy` - Unknown-field policy to store.
    ///
    /// # Returns
    ///
    /// The updated builder.
    #[inline(always)]
    #[must_use]
    pub fn unknown_field_policy(mut self, policy: UnknownFieldPolicy) -> Self {
        self.unknown_field_policy = policy;
        self
    }

    /// Builds the schema.
    ///
    /// # Returns
    ///
    /// The immutable schema described by this builder.
    #[inline(always)]
    #[must_use]
    pub fn build(self) -> MetadataSchema {
        MetadataSchema::new(self.fields, self.unknown_field_policy)
    }
}
