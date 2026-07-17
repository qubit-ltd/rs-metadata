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
    NumericComparisonPolicy,
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
        let numeric_comparison_policy =
            filter.options().numeric_comparison_policy;
        if let Err(error) = filter.visit_conditions(|condition| {
            self.collect_condition_issues(
                condition,
                numeric_comparison_policy,
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
        numeric_comparison_policy: NumericComparisonPolicy,
        issues: &mut Vec<MetadataError>,
    ) {
        match condition {
            Condition::Equal { key, value } => collect_issue(
                issues,
                self.validate_value_condition(
                    key,
                    "eq",
                    value,
                    numeric_comparison_policy,
                ),
            ),
            Condition::NotEqual { key, value } => collect_issue(
                issues,
                self.validate_value_condition(
                    key,
                    "ne",
                    value,
                    numeric_comparison_policy,
                ),
            ),
            Condition::Less { key, value } => collect_issue(
                issues,
                self.validate_range_condition(
                    key,
                    "lt",
                    value,
                    numeric_comparison_policy,
                ),
            ),
            Condition::LessEqual { key, value } => collect_issue(
                issues,
                self.validate_range_condition(
                    key,
                    "le",
                    value,
                    numeric_comparison_policy,
                ),
            ),
            Condition::Greater { key, value } => collect_issue(
                issues,
                self.validate_range_condition(
                    key,
                    "gt",
                    value,
                    numeric_comparison_policy,
                ),
            ),
            Condition::GreaterEqual { key, value } => collect_issue(
                issues,
                self.validate_range_condition(
                    key,
                    "ge",
                    value,
                    numeric_comparison_policy,
                ),
            ),
            Condition::In { key, values } => {
                self.collect_set_value_condition_issues(
                    key,
                    "in_set",
                    values,
                    numeric_comparison_policy,
                    issues,
                );
            }
            Condition::NotIn { key, values } => {
                self.collect_set_value_condition_issues(
                    key,
                    "not_in_set",
                    values,
                    numeric_comparison_policy,
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
        numeric_comparison_policy: NumericComparisonPolicy,
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
                    "filter value type {} is not compatible with field type {} under {:?} number comparison policy",
                    value.data_type(),
                    field.data_type(),
                    numeric_comparison_policy
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
        numeric_comparison_policy: NumericComparisonPolicy,
    ) -> MetadataResult<()> {
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
                "filter value type {} is not compatible with field type {} under {:?} number comparison policy",
                value.data_type(),
                field.data_type(),
                numeric_comparison_policy
            ),
        })
    }

    /// Validates a range value condition.
    fn validate_range_condition(
        &self,
        key: &str,
        operator: &'static str,
        value: &Value,
        numeric_comparison_policy: NumericComparisonPolicy,
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
        if value_matches_field_type(value, field.data_type()) {
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
                numeric_comparison_policy
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

/// Returns `true` when `data_type` supports range comparisons.
#[inline]
fn is_range_comparable_type(data_type: DataType) -> bool {
    data_type.is_numeric() || matches!(data_type, DataType::String)
}

/// Returns `true` when a filter value is compatible with a schema field type.
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
