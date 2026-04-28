/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Filter validation support for [`MetadataSchema`].

use qubit_common::DataType;
use qubit_value::Value;

use super::metadata_field::MetadataField;
use super::metadata_schema::MetadataSchema;
use crate::{Condition, MetadataError, MetadataFilter, MetadataResult};

impl MetadataSchema {
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
            Condition::Equal { key, value } => self.validate_value_condition(key, "eq", value),
            Condition::NotEqual { key, value } => self.validate_value_condition(key, "ne", value),
            Condition::Less { key, value } => self.validate_range_condition(key, "lt", value),
            Condition::LessEqual { key, value } => self.validate_range_condition(key, "le", value),
            Condition::Greater { key, value } => self.validate_range_condition(key, "gt", value),
            Condition::GreaterEqual { key, value } => {
                self.validate_range_condition(key, "ge", value)
            }
            Condition::In { key, values } => {
                for value in values {
                    self.validate_value_condition(key, "in_set", value)?;
                }
                Ok(())
            }
            Condition::NotIn { key, values } => {
                for value in values {
                    self.validate_value_condition(key, "not_in_set", value)?;
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
