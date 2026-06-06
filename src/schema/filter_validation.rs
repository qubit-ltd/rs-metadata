// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Filter validation support for [`MetadataSchema`].

use qubit_datatype::DataType;
use qubit_value::Value;

use super::metadata_field::MetadataField;
use super::metadata_schema::MetadataSchema;
use super::unknown_field_policy::UnknownFieldPolicy;
use crate::{
    Condition,
    MetadataError,
    MetadataFilter,
    MetadataResult,
    MetadataValidationError,
    MetadataValidationResult,
    NumberComparisonPolicy,
};

impl MetadataSchema {
    /// Validates a metadata filter against this schema.
    ///
    /// # Errors
    ///
    /// Returns an aggregate error containing every unknown field, invalid range
    /// operator, and incompatible filter value discovered during this
    /// validation pass. Unknown filter fields are accepted when the
    /// schema's [`UnknownFieldPolicy`] is [`UnknownFieldPolicy::Allow`].
    pub fn validate_filter(
        &self,
        filter: &MetadataFilter,
    ) -> MetadataValidationResult<()> {
        let mut issues = Vec::new();
        let number_comparison_policy =
            filter.options().number_comparison_policy;
        if let Err(error) = filter.visit_conditions(|condition| {
            self.collect_condition_issues(
                condition,
                number_comparison_policy,
                &mut issues,
            );
            Ok(())
        }) {
            issues.push(error);
        }
        if let Some(error) = MetadataValidationError::from_issues(issues) {
            Err(error)
        } else {
            Ok(())
        }
    }

    /// Collects every issue for one filter condition.
    fn collect_condition_issues(
        &self,
        condition: &Condition,
        number_comparison_policy: NumberComparisonPolicy,
        issues: &mut Vec<MetadataError>,
    ) {
        match condition {
            Condition::Equal { key, value } => collect_issue(
                issues,
                self.validate_value_condition(
                    key,
                    "eq",
                    value,
                    number_comparison_policy,
                ),
            ),
            Condition::NotEqual { key, value } => collect_issue(
                issues,
                self.validate_value_condition(
                    key,
                    "ne",
                    value,
                    number_comparison_policy,
                ),
            ),
            Condition::Less { key, value } => collect_issue(
                issues,
                self.validate_range_condition(
                    key,
                    "lt",
                    value,
                    number_comparison_policy,
                ),
            ),
            Condition::LessEqual { key, value } => collect_issue(
                issues,
                self.validate_range_condition(
                    key,
                    "le",
                    value,
                    number_comparison_policy,
                ),
            ),
            Condition::Greater { key, value } => collect_issue(
                issues,
                self.validate_range_condition(
                    key,
                    "gt",
                    value,
                    number_comparison_policy,
                ),
            ),
            Condition::GreaterEqual { key, value } => collect_issue(
                issues,
                self.validate_range_condition(
                    key,
                    "ge",
                    value,
                    number_comparison_policy,
                ),
            ),
            Condition::In { key, values } => {
                self.collect_set_value_condition_issues(
                    key,
                    "in_set",
                    values,
                    number_comparison_policy,
                    issues,
                );
            }
            Condition::NotIn { key, values } => {
                self.collect_set_value_condition_issues(
                    key,
                    "not_in_set",
                    values,
                    number_comparison_policy,
                    issues,
                );
            }
            Condition::Exists { key } | Condition::NotExists { key } => {
                collect_issue(issues, self.filter_field(key).map(|_| ()));
            }
        }
    }

    /// Collects field and value issues for a set-membership condition.
    fn collect_set_value_condition_issues(
        &self,
        key: &str,
        operator: &'static str,
        values: &[Value],
        number_comparison_policy: NumberComparisonPolicy,
        issues: &mut Vec<MetadataError>,
    ) {
        let field = match self.filter_field(key) {
            Ok(field) => field,
            Err(error) => {
                issues.push(error);
                return;
            }
        };
        let Some(field) = field else {
            return;
        };
        for value in values {
            if value_matches_field_type(
                value,
                field.data_type(),
                number_comparison_policy,
            ) {
                continue;
            }
            issues.push(MetadataError::InvalidFilterOperator {
                key: key.to_string(),
                operator,
                data_type: field.data_type(),
                message: format!(
                    "filter value type {} is not compatible with field type {} under {:?} number comparison policy",
                    value.data_type(),
                    field.data_type(),
                    number_comparison_policy
                ),
            });
        }
    }

    /// Validates a non-range value condition.
    fn validate_value_condition(
        &self,
        key: &str,
        operator: &'static str,
        value: &Value,
        number_comparison_policy: NumberComparisonPolicy,
    ) -> MetadataResult<()> {
        let Some(field) = self.filter_field(key)? else {
            return Ok(());
        };
        if value_matches_field_type(
            value,
            field.data_type(),
            number_comparison_policy,
        ) {
            return Ok(());
        }
        Err(MetadataError::InvalidFilterOperator {
            key: key.to_string(),
            operator,
            data_type: field.data_type(),
            message: format!(
                "filter value type {} is not compatible with field type {} under {:?} number comparison policy",
                value.data_type(),
                field.data_type(),
                number_comparison_policy
            ),
        })
    }

    /// Validates a range value condition.
    fn validate_range_condition(
        &self,
        key: &str,
        operator: &'static str,
        value: &Value,
        number_comparison_policy: NumberComparisonPolicy,
    ) -> MetadataResult<()> {
        let Some(field) = self.filter_field(key)? else {
            return Ok(());
        };
        if !is_range_comparable_type(field.data_type()) {
            return Err(MetadataError::InvalidFilterOperator {
                key: key.to_string(),
                operator,
                data_type: field.data_type(),
                message: "range operators require a numeric or string field"
                    .to_string(),
            });
        }
        if value_matches_field_type(
            value,
            field.data_type(),
            number_comparison_policy,
        ) {
            return Ok(());
        }
        Err(MetadataError::InvalidFilterOperator {
            key: key.to_string(),
            operator,
            data_type: field.data_type(),
            message: format!(
                "filter value type {} is not compatible with field type {} under {:?} number comparison policy",
                value.data_type(),
                field.data_type(),
                number_comparison_policy
            ),
        })
    }

    /// Returns the declared filter field, or accepts unknown fields when
    /// allowed.
    fn filter_field(
        &self,
        key: &str,
    ) -> MetadataResult<Option<&MetadataField>> {
        match self.field(key) {
            Some(field) => Ok(Some(field)),
            None if matches!(
                self.unknown_field_policy(),
                UnknownFieldPolicy::Allow
            ) =>
            {
                Ok(None)
            }
            None => Err(MetadataError::UnknownFilterField {
                key: key.to_string(),
            }),
        }
    }
}

/// Appends `result` to the issue list when it contains a validation error.
#[inline]
fn collect_issue(issues: &mut Vec<MetadataError>, result: MetadataResult<()>) {
    if let Err(error) = result {
        issues.push(error);
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

/// Returns `true` when `data_type` is a primitive floating-point type.
#[inline]
fn is_float_data_type(data_type: DataType) -> bool {
    matches!(data_type, DataType::Float32 | DataType::Float64)
}

/// Returns `true` when `data_type` is a big-number type.
#[inline]
fn is_big_number_data_type(data_type: DataType) -> bool {
    matches!(data_type, DataType::BigInteger | DataType::BigDecimal)
}

/// Returns `true` when `data_type` supports range comparisons.
#[inline]
fn is_range_comparable_type(data_type: DataType) -> bool {
    is_numeric_data_type(data_type) || matches!(data_type, DataType::String)
}

/// Returns `true` when a filter value is compatible with a schema field type.
#[inline]
fn value_matches_field_type(
    value: &Value,
    field_type: DataType,
    number_comparison_policy: NumberComparisonPolicy,
) -> bool {
    let value_type = value.data_type();
    if value_type == field_type {
        return true;
    }
    if !is_numeric_data_type(value_type) || !is_numeric_data_type(field_type) {
        return false;
    }
    if matches!(
        number_comparison_policy,
        NumberComparisonPolicy::Approximate
    ) {
        return true;
    }
    value_matches_numeric_field_conservatively(value, field_type)
}

/// Returns `true` when conservative runtime numeric comparison can handle the
/// pair.
fn value_matches_numeric_field_conservatively(
    value: &Value,
    field_type: DataType,
) -> bool {
    let value_type = value.data_type();
    if !is_float_data_type(value_type) && !is_float_data_type(field_type) {
        return true;
    }
    if is_float_data_type(value_type) && is_float_data_type(field_type) {
        return true;
    }
    if is_big_number_data_type(value_type)
        || is_big_number_data_type(field_type)
    {
        return false;
    }
    if is_float_data_type(value_type) {
        return float_value_fits_integer_field(value, field_type);
    }
    integer_value_is_safe_for_float_field(value)
}

const MAX_SAFE_INTEGER_F64_U128: u128 = 9_007_199_254_740_992;
const I64_MIN_F64: f64 = -9_223_372_036_854_775_808.0;
const I64_EXCLUSIVE_MAX_F64: f64 = 9_223_372_036_854_775_808.0;
const U64_EXCLUSIVE_MAX_F64: f64 = 18_446_744_073_709_551_616.0;

/// Extracts a finite floating-point literal from a filter value.
#[inline]
fn finite_float_value(value: &Value) -> Option<f64> {
    let number = value.to::<f64>().ok()?;
    number.is_finite().then_some(number)
}

/// Returns `true` when a float literal can be compared exactly to an integer
/// field.
fn float_value_fits_integer_field(value: &Value, field_type: DataType) -> bool {
    let Some(number) = finite_float_value(value) else {
        return false;
    };
    if number.fract() != 0.0 {
        return false;
    }
    if matches!(field_type, DataType::Int128 | DataType::UInt128) {
        return false;
    }
    if matches!(
        field_type,
        DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::Int64
            | DataType::IntSize
    ) {
        return (I64_MIN_F64..I64_EXCLUSIVE_MAX_F64).contains(&number);
    }
    matches!(
        field_type,
        DataType::UInt8
            | DataType::UInt16
            | DataType::UInt32
            | DataType::UInt64
            | DataType::UIntSize
    ) && (0.0..U64_EXCLUSIVE_MAX_F64).contains(&number)
}

/// Returns `true` when an integer literal can be compared exactly to a float
/// field.
fn integer_value_is_safe_for_float_field(value: &Value) -> bool {
    if let Ok(value) = value.to::<i128>() {
        return value.unsigned_abs() <= MAX_SAFE_INTEGER_F64_U128;
    }
    value
        .to::<u128>()
        .is_ok_and(|value| value <= MAX_SAFE_INTEGER_F64_U128)
}
