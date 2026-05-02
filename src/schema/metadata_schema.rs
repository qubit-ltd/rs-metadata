/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! [`MetadataSchema`] — schema validation for metadata and filters.

use std::collections::BTreeMap;

use qubit_datatype::DataType;
use qubit_value::Value;
use serde::{Deserialize, Serialize};

use crate::schema::{MetadataField, MetadataSchemaBuilder, UnknownFieldPolicy};
use crate::{Metadata, MetadataError, MetadataResult};

/// Schema for metadata fields.
///
/// A schema declares valid keys, their concrete [`DataType`], and whether they
/// are required. It can validate actual [`Metadata`] values and validate that a
/// [`crate::MetadataFilter`] references known fields with compatible operators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataSchema {
    /// Field definitions keyed by metadata key.
    fields: BTreeMap<String, MetadataField>,
    /// How validation handles unknown metadata keys.
    unknown_field_policy: UnknownFieldPolicy,
}

impl MetadataSchema {
    /// Creates a schema builder.
    #[inline]
    #[must_use]
    pub fn builder() -> MetadataSchemaBuilder {
        MetadataSchemaBuilder::default()
    }

    /// Creates a schema from field definitions and unknown-field policy.
    #[inline]
    pub(crate) fn new(
        fields: BTreeMap<String, MetadataField>,
        unknown_field_policy: UnknownFieldPolicy,
    ) -> Self {
        Self {
            fields,
            unknown_field_policy,
        }
    }

    /// Returns the field definition for `key`.
    #[inline]
    #[must_use]
    pub fn field(&self, key: &str) -> Option<&MetadataField> {
        self.fields.get(key)
    }

    /// Returns the declared data type for `key`.
    #[inline]
    #[must_use]
    pub fn field_type(&self, key: &str) -> Option<DataType> {
        self.field(key).map(MetadataField::data_type)
    }

    /// Returns the unknown-field policy.
    #[inline]
    #[must_use]
    pub fn unknown_field_policy(&self) -> UnknownFieldPolicy {
        self.unknown_field_policy
    }

    /// Returns an iterator over schema fields in key-sorted order.
    #[inline]
    pub fn fields(&self) -> impl Iterator<Item = (&str, &MetadataField)> {
        self.fields.iter().map(|(key, field)| (key.as_str(), field))
    }

    /// Validates a metadata object against this schema.
    ///
    /// # Errors
    ///
    /// Returns an error when a required field is missing, a declared field has a
    /// different concrete type, or an unknown field is present while the schema
    /// rejects unknown fields.
    pub fn validate(&self, meta: &Metadata) -> MetadataResult<()> {
        for (key, field) in &self.fields {
            if field.is_required() && !meta.contains_key(key) {
                return Err(MetadataError::MissingRequiredField {
                    key: key.clone(),
                    expected: field.data_type(),
                });
            }
        }

        for (key, value) in meta.iter() {
            self.validate_entry(key, value)?;
        }
        Ok(())
    }

    /// Validates one metadata entry against this schema.
    pub(crate) fn validate_entry(&self, key: &str, value: &Value) -> MetadataResult<()> {
        match self.field(key) {
            Some(field) if field.data_type() != value.data_type() => Err(
                MetadataError::type_mismatch(key, field.data_type(), value.data_type()),
            ),
            Some(_) => Ok(()),
            None if matches!(self.unknown_field_policy, UnknownFieldPolicy::Reject) => {
                Err(MetadataError::UnknownField {
                    key: key.to_string(),
                })
            }
            None => Ok(()),
        }
    }
}

impl Default for MetadataSchema {
    #[inline]
    fn default() -> Self {
        Self {
            fields: BTreeMap::new(),
            unknown_field_policy: UnknownFieldPolicy::Reject,
        }
    }
}
