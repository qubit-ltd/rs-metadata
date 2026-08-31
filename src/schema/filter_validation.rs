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
use super::unknown_filter_field_policy::UnknownFilterFieldPolicy;
use crate::Condition;
use crate::MetadataError;
use crate::MetadataFilter;
use crate::MetadataResult;
use crate::MetadataValidationError;
use crate::MetadataValidationResult;

impl MetadataSchema {
    /// Validates a metadata filter against this schema.
    ///
    /// # Parameters
    ///
    /// * `filter` - Filter to validate.
    ///
    /// # Returns
    ///
    /// `Ok(())` when every condition is compatible with this schema.
    ///
    /// # Errors
    ///
    /// Returns an aggregate error containing every unknown field, invalid range
    /// operator, and incompatible filter value discovered during this
    /// validation pass. Unknown filter fields are accepted when the
    /// schema's [`UnknownFilterFieldPolicy`] is
    /// [`UnknownFilterFieldPolicy::AllowUnchecked`].
    pub fn validate_filter(&self, filter: &MetadataFilter) -> MetadataValidationResult<()> {
        let mut issues = Vec::new();
        if let Err(error) = filter.visit_conditions(|condition| {
            self.collect_condition_issues(condition, &mut issues);
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
    ///
    /// # Parameters
    ///
    /// * `condition` - Condition to validate.
    /// * `issues` - Destination for discovered issues.
    fn collect_condition_issues(&self, condition: &Condition, issues: &mut Vec<MetadataError>) {
        match condition {
            Condition::Equal { key, value } => collect_issue(issues, self.validate_value_condition(key, "eq", value)),
            Condition::NotEqual { key, value } => {
                collect_issue(issues, self.validate_value_condition(key, "ne", value))
            }
            Condition::Less { key, value } => collect_issue(issues, self.validate_range_condition(key, "lt", value)),
            Condition::LessEqual { key, value } => {
                collect_issue(issues, self.validate_range_condition(key, "le", value))
            }
            Condition::Greater { key, value } => collect_issue(issues, self.validate_range_condition(key, "gt", value)),
            Condition::GreaterEqual { key, value } => {
                collect_issue(issues, self.validate_range_condition(key, "ge", value))
            }
            Condition::In { key, values } => {
                self.collect_set_value_condition_issues(key, "in_set", values, issues);
            }
            Condition::NotIn { key, values } => {
                self.collect_set_value_condition_issues(key, "not_in_set", values, issues);
            }
            Condition::Exists { key } | Condition::NotExists { key } => {
                collect_issue(issues, self.filter_field(key).map(|_| ()));
            }
        }
    }

    /// Collects field and value issues for a set-membership condition.
    ///
    /// # Parameters
    ///
    /// * `key` - Schema field referenced by the condition.
    /// * `operator` - Stable operator name used in diagnostics.
    /// * `values` - Candidate values to validate.
    /// * `issues` - Destination for discovered issues.
    fn collect_set_value_condition_issues(
        &self,
        key: &str,
        operator: &'static str,
        values: &[Value],
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
            if value_matches_field_type(value, field.data_type()) {
                continue;
            }
            issues.push(MetadataError::InvalidFilterOperator {
                key: key.to_string(),
                operator,
                data_type: field.data_type(),
                message: format!(
                    "filter value type {} is not compatible with field type {}",
                    value.data_type(),
                    field.data_type()
                ),
            });
        }
    }

    /// Validates a non-range value condition.
    ///
    /// # Parameters
    ///
    /// * `key` - Schema field referenced by the condition.
    /// * `operator` - Stable operator name used in diagnostics.
    /// * `value` - Filter value to validate.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the condition is schema-compatible.
    ///
    /// # Errors
    ///
    /// Returns an unknown-field or incompatible-value error.
    fn validate_value_condition(&self, key: &str, operator: &'static str, value: &Value) -> MetadataResult<()> {
        let Some(field) = self.filter_field(key)? else {
            return Ok(());
        };
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
    ///
    /// # Parameters
    ///
    /// * `key` - Schema field referenced by the condition.
    /// * `operator` - Stable operator name used in diagnostics.
    /// * `value` - Range bound to validate.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the range condition is schema-compatible.
    ///
    /// # Errors
    ///
    /// Returns an unknown-field, invalid-operator, or incompatible-value
    /// error.
    fn validate_range_condition(&self, key: &str, operator: &'static str, value: &Value) -> MetadataResult<()> {
        let Some(field) = self.filter_field(key)? else {
            return Ok(());
        };
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

    /// Returns the declared filter field, or accepts unknown fields when
    /// allowed.
    ///
    /// # Parameters
    ///
    /// * `key` - Filter key to resolve.
    ///
    /// # Returns
    ///
    /// `Some` field for a declared key, or `None` when unknown keys are
    /// allowed.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::UnknownFilterField`] when the key is unknown
    /// and rejected.
    fn filter_field(&self, key: &str) -> MetadataResult<Option<&MetadataField>> {
        match self.field(key) {
            Some(field) => Ok(Some(field)),
            None if matches!(
                self.unknown_filter_field_policy(),
                UnknownFilterFieldPolicy::AllowUnchecked
            ) =>
            {
                Ok(None)
            }
            None => Err(MetadataError::UnknownFilterField { key: key.to_string() }),
        }
    }
}

/// Appends `result` to the issue list when it contains a validation error.
///
/// # Parameters
///
/// * `issues` - Destination issue list.
/// * `result` - Validation result to inspect.
#[inline(always)]
fn collect_issue(issues: &mut Vec<MetadataError>, result: MetadataResult<()>) {
    if let Err(error) = result {
        issues.push(error);
    }
}

/// Returns `true` when `data_type` supports range comparisons.
///
/// # Parameters
///
/// * `data_type` - Data type to inspect.
///
/// # Returns
///
/// `true` for numeric and string types.
#[inline(always)]
fn is_range_comparable_type(data_type: DataType) -> bool {
    data_type.is_numeric() || matches!(data_type, DataType::String)
}

/// Returns `true` when a filter value is compatible with a schema field type.
///
/// Numeric compatibility accepts every non-NaN numeric pairing, independently
/// of the filter's runtime numeric comparison policy.
///
/// # Parameters
///
/// * `value` - Filter value to inspect.
/// * `field_type` - Schema field type to compare against.
///
/// # Returns
///
/// `true` when the value is compatible with the field type.
#[inline]
fn value_matches_field_type(value: &Value, field_type: DataType) -> bool {
    if value.is_unset() {
        return false;
    }
    let value_type = value.data_type();
    if value.is_numeric() && field_type.is_numeric() {
        return !value.is_nan();
    }
    value_type == field_type
}
