/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! [`MetadataSchemaBuilder`] — fluent schema construction API.

use std::collections::BTreeMap;

use qubit_datatype::DataType;

use crate::schema::{MetadataField, MetadataSchema, UnknownFieldPolicy};

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
    #[inline]
    #[must_use]
    pub fn required(mut self, key: &str, data_type: DataType) -> Self {
        self.fields
            .insert(key.to_string(), MetadataField::new(data_type, true));
        self
    }

    /// Adds an optional field definition.
    #[inline]
    #[must_use]
    pub fn optional(mut self, key: &str, data_type: DataType) -> Self {
        self.fields
            .insert(key.to_string(), MetadataField::new(data_type, false));
        self
    }

    /// Sets the policy for metadata keys not declared by the schema.
    #[inline]
    #[must_use]
    pub fn unknown_field_policy(mut self, policy: UnknownFieldPolicy) -> Self {
        self.unknown_field_policy = policy;
        self
    }

    /// Builds the schema.
    #[inline]
    #[must_use]
    pub fn build(self) -> MetadataSchema {
        MetadataSchema::new(self.fields, self.unknown_field_policy)
    }
}
