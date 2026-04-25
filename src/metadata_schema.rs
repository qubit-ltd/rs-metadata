/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! [`MetadataSchema`] — schema validation for metadata and filters.

use std::collections::BTreeMap;

use qubit_common::DataType;
use qubit_value::Value;
use serde::{Deserialize, Serialize};

use crate::{
    Condition, Metadata, MetadataError, MetadataField, MetadataFilter, MetadataResult,
    MetadataSchemaBuilder, UnknownFieldPolicy,
};

/// Schema for metadata fields.
///
/// A schema declares valid keys, their concrete [`DataType`], and whether they
/// are required. It can validate actual [`Metadata`] values and validate that a
/// [`MetadataFilter`] references known fields with compatible operators.
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
            match self.field(key) {
                Some(field) if field.data_type() != value.data_type() => {
                    return Err(MetadataError::type_mismatch(
                        key,
                        field.data_type(),
                        value.data_type(),
                    ));
                }
                Some(_) => {}
                None if matches!(self.unknown_field_policy, UnknownFieldPolicy::Reject) => {
                    return Err(MetadataError::UnknownField {
                        key: key.to_string(),
                    });
                }
                None => {}
            }
        }
        Ok(())
    }

    /// Validates a metadata filter against this schema.
    ///
    /// # Errors
    ///
    /// Returns an error when the filter references an unknown field, uses a range
    /// operator on a non-comparable field, or compares a field with an incompatible
    /// value type.
    pub fn validate_filter(&self, filter: &MetadataFilter) -> MetadataResult<()> {
        filter.visit_conditions(|condition| self.validate_condition(condition))
    }

    /// Validates one filter condition against this schema.
    fn validate_condition(&self, condition: &Condition) -> MetadataResult<()> {
        match condition {
            Condition::Equal { key, value } | Condition::NotEqual { key, value } => {
                self.validate_value_condition(key, "eq", value)
            }
            Condition::Less { key, value } => self.validate_range_condition(key, "lt", value),
            Condition::LessEqual { key, value } => self.validate_range_condition(key, "le", value),
            Condition::Greater { key, value } => self.validate_range_condition(key, "gt", value),
            Condition::GreaterEqual { key, value } => {
                self.validate_range_condition(key, "ge", value)
            }
            Condition::In { key, values } | Condition::NotIn { key, values } => {
                for value in values {
                    self.validate_value_condition(key, "in_set", value)?;
                }
                Ok(())
            }
            Condition::Exists { key } | Condition::NotExists { key } => {
                self.require_field(key)?;
                Ok(())
            }
        }
    }

    /// Validates a non-range value condition.
    fn validate_value_condition(
        &self,
        key: &str,
        operator: &'static str,
        value: &Value,
    ) -> MetadataResult<()> {
        let field = self.require_field(key)?;
        if value_matches_field_type(value, field.data_type()) {
            return Ok(());
        }
        Err(MetadataError::InvalidFilterOperator {
            key: key.to_string(),
            operator,
            data_type: field.data_type(),
            message: format!(
                "filter value type {} is not compatible with field type {}",
                value.data_type(),
                field.data_type()
            ),
        })
    }

    /// Validates a range value condition.
    fn validate_range_condition(
        &self,
        key: &str,
        operator: &'static str,
        value: &Value,
    ) -> MetadataResult<()> {
        let field = self.require_field(key)?;
        if !is_range_comparable_type(field.data_type()) {
            return Err(MetadataError::InvalidFilterOperator {
                key: key.to_string(),
                operator,
                data_type: field.data_type(),
                message: "range operators require a numeric or string field".to_string(),
            });
        }
        if value_matches_field_type(value, field.data_type()) {
            return Ok(());
        }
        Err(MetadataError::InvalidFilterOperator {
            key: key.to_string(),
            operator,
            data_type: field.data_type(),
            message: format!(
                "filter value type {} is not compatible with field type {}",
                value.data_type(),
                field.data_type()
            ),
        })
    }

    /// Returns the field for `key` or a schema error if it is unknown.
    fn require_field(&self, key: &str) -> MetadataResult<&MetadataField> {
        self.field(key)
            .ok_or_else(|| MetadataError::UnknownFilterField {
                key: key.to_string(),
            })
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

/// Returns `true` when `data_type` is numeric.
#[inline]
fn is_numeric_data_type(data_type: DataType) -> bool {
    matches!(
        data_type,
        DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::Int64
            | DataType::Int128
            | DataType::UInt8
            | DataType::UInt16
            | DataType::UInt32
            | DataType::UInt64
            | DataType::UInt128
            | DataType::Float32
            | DataType::Float64
            | DataType::BigInteger
            | DataType::BigDecimal
            | DataType::IntSize
            | DataType::UIntSize
    )
}

/// Returns `true` when `data_type` supports range comparisons.
#[inline]
fn is_range_comparable_type(data_type: DataType) -> bool {
    is_numeric_data_type(data_type) || matches!(data_type, DataType::String)
}

/// Returns `true` when a filter value is compatible with a schema field type.
#[inline]
fn value_matches_field_type(value: &Value, field_type: DataType) -> bool {
    let value_type = value.data_type();
    value_type == field_type || is_numeric_data_type(value_type) && is_numeric_data_type(field_type)
}
