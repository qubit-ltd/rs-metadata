// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! [`MetadataFilterBuilder`] — fluent builder for composable filters.
use qubit_datatype::NumericComparisonPolicy;
use qubit_value::Value;

use super::condition::Condition;
use super::filter_expression::FilterExpression;
use super::metadata_filter::MetadataFilter;
use crate::{
    FilterMatchOptions,
    IntoMetadataValue,
    MetadataError,
    MetadataResult,
    MetadataSchema,
    MetadataValidationError,
    MetadataValidationResult,
};

/// Builder for [`MetadataFilter`].
///
/// Predicates without an explicit connector (`eq`, `gt`, `exists`, and so on)
/// are appended with logical AND. Use `or_*` methods or group methods for more
/// complex expressions.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MetadataFilterBuilder {
    /// Root expression being built. `None` means match all.
    expression: Option<FilterExpression>,
    /// Match policies copied into the built filter.
    options: FilterMatchOptions,
    /// First structural error found while building grouped expressions.
    error: Option<MetadataError>,
    /// Whether at least one leaf condition has been added to this builder.
    has_condition: bool,
}

impl MetadataFilterBuilder {
    /// Builds an immutable [`MetadataFilter`].
    ///
    /// # Returns
    ///
    /// The built filter.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::InvalidFilterExpression`] when the builder
    /// contains a structurally invalid expression such as an empty grouped
    /// expression.
    #[inline]
    pub fn build(self) -> MetadataResult<MetadataFilter> {
        if let Some(error) = self.error {
            return Err(error);
        }
        Ok(MetadataFilter::new(
            self.expression.unwrap_or_default(),
            self.options,
        ))
    }

    /// Builds an immutable filter and validates it against `schema`.
    ///
    /// # Parameters
    ///
    /// * `schema` - Schema used to validate every leaf condition.
    ///
    /// # Returns
    ///
    /// The built and validated filter.
    ///
    /// # Errors
    ///
    /// Returns an aggregate validation error when the filter expression is
    /// structurally invalid, the filter references unknown schema fields, uses
    /// range operators on non-comparable field types, or compares a field with
    /// an incompatible value type.
    #[inline]
    pub fn build_checked(
        self,
        schema: &MetadataSchema,
    ) -> MetadataValidationResult<MetadataFilter> {
        let filter =
            self.build().map_err(MetadataValidationError::from_issue)?;
        schema.validate_filter(&filter)?;
        Ok(filter)
    }

    /// Replaces the match options used by the built filter.
    ///
    /// # Parameters
    ///
    /// * `options` - Replacement match options.
    ///
    /// # Returns
    ///
    /// This builder with the replacement options.
    #[inline(always)]
    #[must_use]
    pub fn with_options(mut self, options: FilterMatchOptions) -> Self {
        self.options = options;
        self
    }

    /// Sets how the built filter handles mixed numeric comparisons.
    ///
    /// # Parameters
    ///
    /// * `numeric_comparison_policy` - Policy for mixed numeric values.
    ///
    /// # Returns
    ///
    /// This builder with the replacement policy.
    #[inline(always)]
    #[must_use]
    pub fn numeric_comparison_policy(
        mut self,
        numeric_comparison_policy: NumericComparisonPolicy,
    ) -> Self {
        self.options
            .set_numeric_comparison_policy(numeric_comparison_policy);
        self
    }

    /// Appends an equality predicate with AND: `key == value`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to compare.
    /// * `value` - Expected value.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline(always)]
    #[must_use]
    pub fn eq<T>(self, key: &str, value: T) -> Self
    where
        T: IntoMetadataValue,
    {
        self.and_eq(key, value)
    }

    /// Appends a not-equal predicate with AND: `key != value`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to compare.
    /// * `value` - Value to reject.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline(always)]
    #[must_use]
    pub fn ne<T>(self, key: &str, value: T) -> Self
    where
        T: IntoMetadataValue,
    {
        self.and_ne(key, value)
    }

    /// Appends a less-than predicate with AND: `key < value`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to compare.
    /// * `value` - Exclusive upper bound.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline(always)]
    #[must_use]
    pub fn lt<T>(self, key: &str, value: T) -> Self
    where
        T: IntoMetadataValue,
    {
        self.and_lt(key, value)
    }

    /// Appends a less-than-or-equal predicate with AND: `key <= value`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to compare.
    /// * `value` - Inclusive upper bound.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline(always)]
    #[must_use]
    pub fn le<T>(self, key: &str, value: T) -> Self
    where
        T: IntoMetadataValue,
    {
        self.and_le(key, value)
    }

    /// Appends a greater-than predicate with AND: `key > value`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to compare.
    /// * `value` - Exclusive lower bound.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline(always)]
    #[must_use]
    pub fn gt<T>(self, key: &str, value: T) -> Self
    where
        T: IntoMetadataValue,
    {
        self.and_gt(key, value)
    }

    /// Appends a greater-than-or-equal predicate with AND: `key >= value`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to compare.
    /// * `value` - Inclusive lower bound.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline(always)]
    #[must_use]
    pub fn ge<T>(self, key: &str, value: T) -> Self
    where
        T: IntoMetadataValue,
    {
        self.and_ge(key, value)
    }

    /// Appends an inclusion predicate with AND: `key` is in `values`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to compare.
    /// * `values` - Accepted candidate values.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline(always)]
    #[must_use]
    pub fn in_set<I, T>(self, key: &str, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: IntoMetadataValue,
    {
        self.and_in_set(key, values)
    }

    /// Appends an exclusion predicate with AND: `key` is not in `values`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to compare.
    /// * `values` - Rejected candidate values.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline(always)]
    #[must_use]
    pub fn not_in_set<I, T>(self, key: &str, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: IntoMetadataValue,
    {
        self.and_not_in_set(key, values)
    }

    /// Appends an existence predicate with AND.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key that must store a concrete value.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline(always)]
    #[must_use]
    pub fn exists(self, key: &str) -> Self {
        self.and_exists(key)
    }

    /// Appends a non-existence predicate with AND.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key that must be absent or unset.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline(always)]
    #[must_use]
    pub fn not_exists(self, key: &str) -> Self {
        self.and_not_exists(key)
    }

    /// Appends an equality predicate with AND: `key == value`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to compare.
    /// * `value` - Expected value.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline]
    #[must_use]
    pub fn and_eq<T>(self, key: &str, value: T) -> Self
    where
        T: IntoMetadataValue,
    {
        self.and_condition(Condition::Equal {
            key: key.to_string(),
            value: to_value(value),
        })
    }

    /// Appends a not-equal predicate with AND: `key != value`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to compare.
    /// * `value` - Value to reject.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline]
    #[must_use]
    pub fn and_ne<T>(self, key: &str, value: T) -> Self
    where
        T: IntoMetadataValue,
    {
        self.and_condition(Condition::NotEqual {
            key: key.to_string(),
            value: to_value(value),
        })
    }

    /// Appends a less-than predicate with AND: `key < value`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to compare.
    /// * `value` - Exclusive upper bound.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline]
    #[must_use]
    pub fn and_lt<T>(self, key: &str, value: T) -> Self
    where
        T: IntoMetadataValue,
    {
        self.and_condition(Condition::Less {
            key: key.to_string(),
            value: to_value(value),
        })
    }

    /// Appends a less-than-or-equal predicate with AND: `key <= value`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to compare.
    /// * `value` - Inclusive upper bound.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline]
    #[must_use]
    pub fn and_le<T>(self, key: &str, value: T) -> Self
    where
        T: IntoMetadataValue,
    {
        self.and_condition(Condition::LessEqual {
            key: key.to_string(),
            value: to_value(value),
        })
    }

    /// Appends a greater-than predicate with AND: `key > value`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to compare.
    /// * `value` - Exclusive lower bound.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline]
    #[must_use]
    pub fn and_gt<T>(self, key: &str, value: T) -> Self
    where
        T: IntoMetadataValue,
    {
        self.and_condition(Condition::Greater {
            key: key.to_string(),
            value: to_value(value),
        })
    }

    /// Appends a greater-than-or-equal predicate with AND: `key >= value`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to compare.
    /// * `value` - Inclusive lower bound.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline]
    #[must_use]
    pub fn and_ge<T>(self, key: &str, value: T) -> Self
    where
        T: IntoMetadataValue,
    {
        self.and_condition(Condition::GreaterEqual {
            key: key.to_string(),
            value: to_value(value),
        })
    }

    /// Appends an inclusion predicate with AND: `key` is in `values`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to compare.
    /// * `values` - Accepted candidate values.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline]
    #[must_use]
    pub fn and_in_set<I, T>(self, key: &str, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: IntoMetadataValue,
    {
        self.and_condition(Condition::In {
            key: key.to_string(),
            values: Self::collect_values(values),
        })
    }

    /// Appends an exclusion predicate with AND: `key` is not in `values`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to compare.
    /// * `values` - Rejected candidate values.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline]
    #[must_use]
    pub fn and_not_in_set<I, T>(self, key: &str, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: IntoMetadataValue,
    {
        self.and_condition(Condition::NotIn {
            key: key.to_string(),
            values: Self::collect_values(values),
        })
    }

    /// Appends an existence predicate with AND.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key that must store a concrete value.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline]
    #[must_use]
    pub fn and_exists(self, key: &str) -> Self {
        self.and_condition(Condition::Exists {
            key: key.to_string(),
        })
    }

    /// Appends a non-existence predicate with AND.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key that must be absent or unset.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline]
    #[must_use]
    pub fn and_not_exists(self, key: &str) -> Self {
        self.and_condition(Condition::NotExists {
            key: key.to_string(),
        })
    }

    /// Appends an equality predicate with OR: `key == value`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to compare.
    /// * `value` - Expected value.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline]
    #[must_use]
    pub fn or_eq<T>(self, key: &str, value: T) -> Self
    where
        T: IntoMetadataValue,
    {
        self.or_condition(Condition::Equal {
            key: key.to_string(),
            value: to_value(value),
        })
    }

    /// Appends a not-equal predicate with OR: `key != value`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to compare.
    /// * `value` - Value to reject.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline]
    #[must_use]
    pub fn or_ne<T>(self, key: &str, value: T) -> Self
    where
        T: IntoMetadataValue,
    {
        self.or_condition(Condition::NotEqual {
            key: key.to_string(),
            value: to_value(value),
        })
    }

    /// Appends a less-than predicate with OR: `key < value`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to compare.
    /// * `value` - Exclusive upper bound.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline]
    #[must_use]
    pub fn or_lt<T>(self, key: &str, value: T) -> Self
    where
        T: IntoMetadataValue,
    {
        self.or_condition(Condition::Less {
            key: key.to_string(),
            value: to_value(value),
        })
    }

    /// Appends a less-than-or-equal predicate with OR: `key <= value`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to compare.
    /// * `value` - Inclusive upper bound.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline]
    #[must_use]
    pub fn or_le<T>(self, key: &str, value: T) -> Self
    where
        T: IntoMetadataValue,
    {
        self.or_condition(Condition::LessEqual {
            key: key.to_string(),
            value: to_value(value),
        })
    }

    /// Appends a greater-than predicate with OR: `key > value`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to compare.
    /// * `value` - Exclusive lower bound.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline]
    #[must_use]
    pub fn or_gt<T>(self, key: &str, value: T) -> Self
    where
        T: IntoMetadataValue,
    {
        self.or_condition(Condition::Greater {
            key: key.to_string(),
            value: to_value(value),
        })
    }

    /// Appends a greater-than-or-equal predicate with OR: `key >= value`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to compare.
    /// * `value` - Inclusive lower bound.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline]
    #[must_use]
    pub fn or_ge<T>(self, key: &str, value: T) -> Self
    where
        T: IntoMetadataValue,
    {
        self.or_condition(Condition::GreaterEqual {
            key: key.to_string(),
            value: to_value(value),
        })
    }

    /// Appends an inclusion predicate with OR: `key` is in `values`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to compare.
    /// * `values` - Accepted candidate values.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline]
    #[must_use]
    pub fn or_in_set<I, T>(self, key: &str, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: IntoMetadataValue,
    {
        self.or_condition(Condition::In {
            key: key.to_string(),
            values: Self::collect_values(values),
        })
    }

    /// Appends an exclusion predicate with OR: `key` is not in `values`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key to compare.
    /// * `values` - Rejected candidate values.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline]
    #[must_use]
    pub fn or_not_in_set<I, T>(self, key: &str, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: IntoMetadataValue,
    {
        self.or_condition(Condition::NotIn {
            key: key.to_string(),
            values: Self::collect_values(values),
        })
    }

    /// Appends an existence predicate with OR.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key that must store a concrete value.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline]
    #[must_use]
    pub fn or_exists(self, key: &str) -> Self {
        self.or_condition(Condition::Exists {
            key: key.to_string(),
        })
    }

    /// Appends a non-existence predicate with OR.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key that must be absent or unset.
    ///
    /// # Returns
    ///
    /// This builder with the predicate appended.
    #[inline]
    #[must_use]
    pub fn or_not_exists(self, key: &str) -> Self {
        self.or_condition(Condition::NotExists {
            key: key.to_string(),
        })
    }

    /// Appends a grouped expression with AND.
    ///
    /// The closure receives a fresh builder for the group. Policies configured
    /// inside the group are ignored; configure policies on the outer builder.
    ///
    /// # Parameters
    ///
    /// * `build` - Callback that constructs the grouped expression.
    ///
    /// # Returns
    ///
    /// This builder with the group appended using logical AND.
    #[inline]
    #[must_use]
    pub fn and<F>(self, build: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        let group = build(Self::default());
        self.and_group("and", group)
    }

    /// Appends a grouped expression with OR.
    ///
    /// The closure receives a fresh builder for the group. Policies configured
    /// inside the group are ignored; configure policies on the outer builder.
    ///
    /// # Parameters
    ///
    /// * `build` - Callback that constructs the grouped expression.
    ///
    /// # Returns
    ///
    /// This builder with the group appended using logical OR.
    #[inline]
    #[must_use]
    pub fn or<F>(self, build: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        let group = build(Self::default());
        self.or_group("or", group)
    }

    /// Appends a negated grouped expression with AND.
    ///
    /// # Parameters
    ///
    /// * `build` - Callback that constructs the grouped expression.
    ///
    /// # Returns
    ///
    /// This builder with the negated group appended using logical AND.
    #[inline]
    #[must_use]
    pub fn and_not<F>(self, build: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        let group = build(Self::default());
        self.and_group("and_not", group.negate_non_empty_group())
    }

    /// Appends a negated grouped expression with OR.
    ///
    /// # Parameters
    ///
    /// * `build` - Callback that constructs the grouped expression.
    ///
    /// # Returns
    ///
    /// This builder with the negated group appended using logical OR.
    #[inline]
    #[must_use]
    pub fn or_not<F>(self, build: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        let group = build(Self::default());
        self.or_group("or_not", group.negate_non_empty_group())
    }

    /// Negates the entire builder expression.
    ///
    /// # Returns
    ///
    /// This builder with its current expression logically negated.
    #[allow(clippy::should_implement_trait)]
    #[inline(always)]
    #[must_use]
    pub fn not(mut self) -> Self {
        self.expression = Self::negate_expression(self.expression);
        self
    }

    /// Converts a sequence of typed values into stored values.
    ///
    /// # Parameters
    ///
    /// * `values` - Typed values to convert.
    ///
    /// # Returns
    ///
    /// The converted stored values in input order.
    fn collect_values<I, T>(values: I) -> Vec<Value>
    where
        I: IntoIterator<Item = T>,
        T: IntoMetadataValue,
    {
        values.into_iter().map(to_value).collect()
    }

    /// Combines the current expression with `expression` using logical AND.
    ///
    /// # Parameters
    ///
    /// * `expression` - Optional expression to combine.
    ///
    /// # Returns
    ///
    /// This builder containing the combined expression.
    fn and_expression(mut self, expression: Option<FilterExpression>) -> Self {
        self.expression = match (self.expression, expression) {
            (None, rhs) => rhs,
            (lhs, None) => lhs,
            (Some(left), Some(right)) => {
                Some(FilterExpression::and(left, right))
            }
        };
        self
    }

    /// Combines the current expression with `expression` using logical OR.
    ///
    /// # Parameters
    ///
    /// * `expression` - Optional expression to combine.
    ///
    /// # Returns
    ///
    /// This builder containing the combined expression.
    fn or_expression(mut self, expression: Option<FilterExpression>) -> Self {
        self.expression = match (self.expression, expression) {
            (None, rhs) => rhs,
            (lhs, None) => lhs,
            (Some(left), Some(right)) => {
                Some(FilterExpression::or(left, right))
            }
        };
        self
    }

    /// Appends a condition with logical AND.
    ///
    /// # Parameters
    ///
    /// * `condition` - Leaf condition to append.
    ///
    /// # Returns
    ///
    /// This builder with the condition appended.
    #[inline]
    fn and_condition(self, condition: Condition) -> Self {
        let mut builder =
            self.and_expression(Some(FilterExpression::condition(condition)));
        builder.has_condition = true;
        builder
    }

    /// Appends a condition with logical OR.
    ///
    /// # Parameters
    ///
    /// * `condition` - Leaf condition to append.
    ///
    /// # Returns
    ///
    /// This builder with the condition appended.
    #[inline]
    fn or_condition(self, condition: Condition) -> Self {
        let mut builder =
            self.or_expression(Some(FilterExpression::condition(condition)));
        builder.has_condition = true;
        builder
    }

    /// Combines a grouped expression with logical AND.
    ///
    /// # Parameters
    ///
    /// * `operator` - Stable operator name for structural diagnostics.
    /// * `group` - Grouped builder to append.
    ///
    /// # Returns
    ///
    /// This builder with the group or its structural error appended.
    #[inline(always)]
    fn and_group(self, operator: &'static str, group: Self) -> Self {
        self.combine_group(operator, group, Self::and_expression)
    }

    /// Combines a grouped expression with logical OR.
    ///
    /// # Parameters
    ///
    /// * `operator` - Stable operator name for structural diagnostics.
    /// * `group` - Grouped builder to append.
    ///
    /// # Returns
    ///
    /// This builder with the group or its structural error appended.
    #[inline(always)]
    fn or_group(self, operator: &'static str, group: Self) -> Self {
        self.combine_group(operator, group, Self::or_expression)
    }

    /// Combines a grouped expression and records empty-group errors.
    ///
    /// # Parameters
    ///
    /// * `operator` - Stable operator name for structural diagnostics.
    /// * `group` - Grouped builder to combine.
    /// * `combine` - Function that joins the group with the outer expression.
    ///
    /// # Returns
    ///
    /// The combined builder, or this builder carrying the first structural
    /// error.
    fn combine_group(
        self,
        operator: &'static str,
        group: Self,
        combine: fn(Self, Option<FilterExpression>) -> Self,
    ) -> Self {
        if let Some(error) = group.error {
            return self.with_error(error);
        }
        if !group.has_condition {
            return self.with_empty_group_error(operator);
        }
        let mut builder = combine(self, group.expression);
        builder.has_condition = true;
        builder
    }

    /// Negates the group expression while preserving empty-group detection.
    ///
    /// # Returns
    ///
    /// This group with its expression negated when one exists.
    #[inline]
    fn negate_non_empty_group(mut self) -> Self {
        if self.expression.is_some() {
            self.expression = Self::negate_expression(self.expression);
        }
        self
    }

    /// Records a structural filter expression error.
    ///
    /// # Parameters
    ///
    /// * `error` - Structural error to record when no earlier error exists.
    ///
    /// # Returns
    ///
    /// This builder carrying its first structural error.
    #[inline]
    fn with_error(mut self, error: MetadataError) -> Self {
        if self.error.is_none() {
            self.error = Some(error);
        }
        self
    }

    /// Records an empty grouped expression error.
    ///
    /// # Parameters
    ///
    /// * `operator` - Operator whose group was empty.
    ///
    /// # Returns
    ///
    /// This builder carrying an empty-group error.
    #[inline]
    fn with_empty_group_error(self, operator: &'static str) -> Self {
        self.with_error(MetadataError::InvalidFilterExpression {
            message: format!("empty '{operator}' filter group is not allowed"),
        })
    }

    /// Negates an optional expression.
    ///
    /// # Parameters
    ///
    /// * `expression` - Expression to negate; `None` represents true.
    ///
    /// # Returns
    ///
    /// The simplified negated expression.
    #[inline(always)]
    fn negate_expression(
        expression: Option<FilterExpression>,
    ) -> Option<FilterExpression> {
        Some(expression.unwrap_or_default().negated())
    }
}

/// Converts one typed value into its stored representation.
///
/// # Parameters
///
/// * `value` - Typed value to convert.
///
/// # Returns
///
/// The converted stored value.
#[inline(always)]
fn to_value<T>(value: T) -> Value
where
    T: IntoMetadataValue,
{
    value.into_metadata_value()
}
