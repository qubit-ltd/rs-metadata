// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! [`MetadataSchema`] — schema validation for metadata and filters.

use std::collections::BTreeMap;

use qubit_datatype::DataType;
use qubit_value::Value;
use serde::{Deserialize, Serialize};

use crate::schema::{
    MetadataField, MetadataSchemaBuilder, UnknownFilterFieldPolicy, UnknownMetadataFieldPolicy,
};
use crate::{
    Metadata, MetadataError, MetadataResult, MetadataValidationError, MetadataValidationResult,
};

/// Schema for metadata fields.
///
/// A schema declares valid keys, their concrete [`DataType`], and whether they
/// are required. It can validate actual [`Metadata`] values and validate that a
/// [`crate::MetadataFilter`] references known fields with compatible operators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetadataSchema {
    /// Field definitions keyed by metadata key.
    fields: BTreeMap<String, MetadataField>,
    /// How validation handles unknown metadata keys.
    unknown_metadata_field_policy: UnknownMetadataFieldPolicy,
    /// How validation handles unknown filter keys.
    unknown_filter_field_policy: UnknownFilterFieldPolicy,
}

impl MetadataSchema {
    /// Creates a schema builder.
    ///
    /// # Returns
    ///
    /// An empty schema builder using the default unknown-field policy.
    #[inline(always)]
    #[must_use]
    pub fn builder() -> MetadataSchemaBuilder {
        MetadataSchemaBuilder::default()
    }

    /// Creates a schema from field definitions and unknown-field policies.
    ///
    /// # Parameters
    ///
    /// * `fields` - Field definitions keyed by metadata key.
    /// * `unknown_metadata_field_policy` - Policy for undeclared metadata keys.
    /// * `unknown_filter_field_policy` - Policy for undeclared filter keys.
    ///
    /// # Returns
    ///
    /// A new immutable schema.
    #[inline]
    pub(crate) fn new(
        fields: BTreeMap<String, MetadataField>,
        unknown_metadata_field_policy: UnknownMetadataFieldPolicy,
        unknown_filter_field_policy: UnknownFilterFieldPolicy,
    ) -> Self {
        Self {
            fields,
            unknown_metadata_field_policy,
            unknown_filter_field_policy,
        }
    }

    /// Returns the field definition for `key`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to look up.
    ///
    /// # Returns
    ///
    /// `Some` field definition for a declared key; otherwise, `None`.
    #[inline(always)]
    pub fn field(&self, key: &str) -> Option<&MetadataField> {
        self.fields.get(key)
    }

    /// Returns the declared data type for `key`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to look up.
    ///
    /// # Returns
    ///
    /// `Some` declared data type for a known key; otherwise, `None`.
    #[inline(always)]
    pub fn field_type(&self, key: &str) -> Option<DataType> {
        self.field(key).map(MetadataField::data_type)
    }

    /// Returns the policy for undeclared metadata fields.
    ///
    /// # Returns
    ///
    /// The policy applied to undeclared metadata keys.
    #[inline(always)]
    #[must_use]
    pub fn unknown_metadata_field_policy(&self) -> UnknownMetadataFieldPolicy {
        self.unknown_metadata_field_policy
    }

    /// Returns the policy for undeclared filter fields.
    ///
    /// # Returns
    ///
    /// The policy applied to filter keys not declared in this schema.
    #[inline(always)]
    #[must_use]
    pub fn unknown_filter_field_policy(&self) -> UnknownFilterFieldPolicy {
        self.unknown_filter_field_policy
    }

    /// Returns an iterator over schema fields in key-sorted order.
    ///
    /// # Returns
    ///
    /// An iterator yielding key and field-definition pairs.
    #[inline(always)]
    pub fn fields(&self) -> impl Iterator<Item = (&str, &MetadataField)> {
        self.fields.iter().map(|(key, field)| (key.as_str(), field))
    }

    /// Validates a metadata object against this schema.
    ///
    /// # Parameters
    ///
    /// * `meta` - Metadata object to validate.
    ///
    /// # Returns
    ///
    /// `Ok(())` when every entry satisfies the schema.
    ///
    /// # Errors
    ///
    /// Returns an aggregate error containing every required-field,
    /// declared-type, and unknown-field issue discovered during this
    /// validation pass.
    pub fn validate(&self, meta: &Metadata) -> MetadataValidationResult<()> {
        let mut issues = Vec::new();
        for (key, field) in &self.fields {
            if field.is_required() && meta.get_raw(key).is_none_or(Value::is_unset) {
                issues.push(MetadataError::MissingRequiredField {
                    key: key.clone(),
                    expected: field.data_type(),
                });
            }
        }

        for (key, value) in meta.iter() {
            if let Err(error) = self.validate_entry(key, value) {
                issues.push(error);
            }
        }
        if let Some(error) = MetadataValidationError::from_issues(issues) {
            Err(error)
        } else {
            Ok(())
        }
    }

    /// Validates one metadata entry against this schema.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to validate.
    /// * `value` - Stored value to validate.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the entry is accepted.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::UnknownField`] for a rejected undeclared key,
    /// or [`MetadataError::TypeMismatch`] for a declared type mismatch.
    pub(crate) fn validate_entry(&self, key: &str, value: &Value) -> MetadataResult<()> {
        match self.field(key) {
            Some(field) if field.data_type() != value.data_type() => Err(
                MetadataError::type_mismatch(key, field.data_type(), value.data_type()),
            ),
            Some(_) => Ok(()),
            None if matches!(
                self.unknown_metadata_field_policy,
                UnknownMetadataFieldPolicy::Reject
            ) =>
            {
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
            unknown_metadata_field_policy: UnknownMetadataFieldPolicy::Reject,
            unknown_filter_field_policy: UnknownFilterFieldPolicy::Reject,
        }
    }
}
