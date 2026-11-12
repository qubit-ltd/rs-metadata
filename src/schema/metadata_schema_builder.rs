// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! [`MetadataSchemaBuilder`] — fluent schema construction API.

use std::collections::BTreeMap;
use std::fmt;

use qubit_datatype::DataType;
use qubit_redact::Redact;
use qubit_redact::RedactionWriter;
use qubit_redact::Redactor;

use crate::MetadataError;
use crate::MetadataResult;
use crate::schema::MetadataField;
use crate::schema::MetadataSchema;
use crate::schema::UnknownFilterFieldPolicy;
use crate::schema::UnknownMetadataFieldPolicy;
use crate::schema::internal::SchemaFields;

/// Builder for [`MetadataSchema`].
///
/// Its [`fmt::Debug`] output uses strict redaction with one cumulative
/// diagnostic budget for the field map and builder state. If that budget is
/// exceeded, Debug displays `<redaction incomplete>`. A stored build error is
/// represented only by whether an error exists; [`Self::build`] returns its
/// original details. Debug output is bounded, but field names are not treated
/// as a confidentiality boundary.
///
/// # Examples
///
/// ```
/// use qubit_datatype::DataType;
/// use qubit_metadata::MetadataSchema;
///
/// # fn main() -> qubit_metadata::MetadataResult<()> {
/// let schema = MetadataSchema::builder()
///     .required("tenant", DataType::String)
///     .build()?;
/// assert!(schema.field("tenant").is_some());
/// # Ok(())
/// # }
/// ```
#[derive(Clone, PartialEq, Eq, Default)]
pub struct MetadataSchemaBuilder {
    /// Field definitions being built.
    fields: BTreeMap<String, MetadataField>,
    /// Policy for metadata keys not declared in the schema.
    unknown_metadata_field_policy: UnknownMetadataFieldPolicy,
    /// Policy for filter keys not declared in the schema.
    unknown_filter_field_policy: UnknownFilterFieldPolicy,
    /// First duplicate declaration detected while building the schema.
    error: Option<MetadataError>,
}

impl Redact for MetadataSchemaBuilder {
    /// Writes builder fields through one cumulative diagnostic budget.
    ///
    /// # Parameters
    ///
    /// * `writer` - Redaction writer receiving the builder representation.
    fn write_redacted(&self, writer: &mut RedactionWriter<'_>) {
        writer.record("MetadataSchemaBuilder", |fields| {
            fields.nested("fields", &SchemaFields(&self.fields));
            fields.unmarked("unknown_metadata_field_policy", || self.unknown_metadata_field_policy);
            fields.unmarked("unknown_filter_field_policy", || self.unknown_filter_field_policy);
            fields.unmarked("has_error", || self.error.is_some());
        });
    }
}

impl fmt::Debug for MetadataSchemaBuilder {
    /// Writes the strict-policy diagnostic representation.
    ///
    /// # Parameters
    ///
    /// * `formatter` - Formatter receiving the redacted representation.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = Redactor::strict().redact_text(self);
        let text = output.text_or_marker("<redaction incomplete>");
        formatter.write_str(text.as_ref())
    }
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
    #[inline]
    #[must_use]
    pub fn required(self, key: &str, data_type: DataType) -> Self {
        self.declare_field(key, MetadataField::new(data_type, true))
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
    #[inline]
    #[must_use]
    pub fn optional(self, key: &str, data_type: DataType) -> Self {
        self.declare_field(key, MetadataField::new(data_type, false))
    }

    /// Explicitly replaces a field definition.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to replace.
    /// * `field` - Replacement definition.
    ///
    /// # Returns
    ///
    /// The updated builder. This is the only declaration method that may
    /// overwrite an existing field.
    #[inline]
    #[must_use]
    pub fn replace_field(mut self, key: &str, field: MetadataField) -> Self {
        let _ = self.fields.insert(key.to_string(), field);
        self
    }

    /// Sets the policy for metadata keys not declared by the schema.
    ///
    /// # Parameters
    ///
    /// * `policy` - Unknown metadata-field policy to store.
    ///
    /// # Returns
    ///
    /// The updated builder.
    #[inline]
    #[must_use]
    pub fn unknown_metadata_field_policy(mut self, policy: UnknownMetadataFieldPolicy) -> Self {
        self.unknown_metadata_field_policy = policy;
        self
    }

    /// Sets the policy for filter keys not declared by the schema.
    ///
    /// # Parameters
    ///
    /// * `policy` - Unknown filter-field policy to store.
    ///
    /// # Returns
    ///
    /// The updated builder.
    #[inline]
    #[must_use]
    pub fn unknown_filter_field_policy(mut self, policy: UnknownFilterFieldPolicy) -> Self {
        self.unknown_filter_field_policy = policy;
        self
    }

    /// Builds the schema.
    ///
    /// # Returns
    ///
    /// The immutable schema described by this builder.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::DuplicateSchemaField`] when `required` or
    /// `optional` declare the same key more than once.
    #[inline]
    pub fn build(self) -> MetadataResult<MetadataSchema> {
        if let Some(error) = self.error {
            return Err(error);
        }
        Ok(MetadataSchema::new(
            self.fields,
            self.unknown_metadata_field_policy,
            self.unknown_filter_field_policy,
        ))
    }

    /// Declares a field unless the key has already been declared.
    ///
    /// # Parameters
    ///
    /// * `key` - Name assigned to the field.
    /// * `field` - Definition stored for the key.
    ///
    /// # Returns
    ///
    /// The updated builder, retaining the first duplicate-key error if present.
    fn declare_field(mut self, key: &str, field: MetadataField) -> Self {
        if self.fields.contains_key(key) {
            if self.error.is_none() {
                self.error = Some(MetadataError::DuplicateSchemaField { key: key.to_string() });
            }
            return self;
        }
        let _ = self.fields.insert(key.to_string(), field);
        self
    }
}
