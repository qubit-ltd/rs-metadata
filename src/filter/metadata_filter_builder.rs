/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! [`MetadataFilterBuilder`] — fluent builder for composable filters.
use qubit_value::{Value, ValueConstructor};

use super::condition::Condition;
use super::filter_expr::FilterExpr;
use super::metadata_filter::MetadataFilter;
use crate::{
    FilterMatchOptions, MetadataError, MetadataResult, MetadataSchema, MissingKeyPolicy,
    NumberComparisonPolicy,
};

/// Builder for [`MetadataFilter`].
///
/// Predicates without an explicit connector (`eq`, `gt`, `exists`, and so on)
/// are appended with logical AND. Use `or_*` methods or group methods for more
/// complex expressions.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MetadataFilterBuilder {
    /// Root expression being built. `None` means match all.
    pub(crate) expr: Option<FilterExpr>,
    /// Match policies copied into the built filter.
    pub(crate) options: FilterMatchOptions,
    /// First structural error found while building grouped expressions.
    pub(crate) error: Option<MetadataError>,
    /// Whether at least one leaf condition has been added to this builder.
    pub(crate) has_condition: bool,
}

impl MetadataFilterBuilder {
    /// Builds an immutable [`MetadataFilter`].
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::InvalidFilterExpression`] when the builder contains
    /// a structurally invalid expression such as an empty grouped expression.
    #[inline]
    pub fn build(self) -> MetadataResult<MetadataFilter> {
        if let Some(error) = self.error {
            return Err(error);
        }
        Ok(MetadataFilter::new(self.expr, self.options))
    }

    /// Builds an immutable filter and validates it against `schema`.
    ///
    /// # Errors
    ///
    /// Returns an error when the filter expression is structurally invalid, the
    /// filter references unknown schema fields, uses range operators on
    /// non-comparable field types, or compares a field with an incompatible value
    /// type.
    #[inline]
    pub fn build_checked(self, schema: &MetadataSchema) -> MetadataResult<MetadataFilter> {
        let filter = self.build()?;
        schema.validate_filter(&filter)?;
        Ok(filter)
    }

    /// Replaces the match options used by the built filter.
    #[inline]
    #[must_use]
    pub fn with_options(mut self, options: FilterMatchOptions) -> Self {
        self.options = options;
        self
    }

    /// Sets how the built filter treats missing keys in negative predicates.
    #[inline]
    #[must_use]
    pub fn missing_key_policy(mut self, missing_key_policy: MissingKeyPolicy) -> Self {
        self.options.missing_key_policy = missing_key_policy;
        self
    }

    /// Sets how the built filter handles mixed numeric comparisons.
    #[inline]
    #[must_use]
    pub fn number_comparison_policy(
        mut self,
        number_comparison_policy: NumberComparisonPolicy,
    ) -> Self {
        self.options.number_comparison_policy = number_comparison_policy;
        self
    }

    /// Appends an equality predicate with AND: `key == value`.
    #[inline]
    #[must_use]
    pub fn eq<T>(self, key: &str, value: T) -> Self
    where
        Value: ValueConstructor<T>,
    {
        self.and_eq(key, value)
    }

    /// Appends a not-equal predicate with AND: `key != value`.
    #[inline]
    #[must_use]
    pub fn ne<T>(self, key: &str, value: T) -> Self
    where
        Value: ValueConstructor<T>,
    {
        self.and_ne(key, value)
    }

    /// Appends a less-than predicate with AND: `key < value`.
    #[inline]
    #[must_use]
    pub fn lt<T>(self, key: &str, value: T) -> Self
    where
        Value: ValueConstructor<T>,
    {
        self.and_lt(key, value)
    }

    /// Appends a less-than-or-equal predicate with AND: `key <= value`.
    #[inline]
    #[must_use]
    pub fn le<T>(self, key: &str, value: T) -> Self
    where
        Value: ValueConstructor<T>,
    {
        self.and_le(key, value)
    }

    /// Appends a greater-than predicate with AND: `key > value`.
    #[inline]
    #[must_use]
    pub fn gt<T>(self, key: &str, value: T) -> Self
    where
        Value: ValueConstructor<T>,
    {
        self.and_gt(key, value)
    }

    /// Appends a greater-than-or-equal predicate with AND: `key >= value`.
    #[inline]
    #[must_use]
    pub fn ge<T>(self, key: &str, value: T) -> Self
    where
        Value: ValueConstructor<T>,
    {
        self.and_ge(key, value)
    }

    /// Appends an inclusion predicate with AND: `key` is in `values`.
    #[inline]
    #[must_use]
    pub fn in_set<I, T>(self, key: &str, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        Value: ValueConstructor<T>,
    {
        self.and_in_set(key, values)
    }

    /// Appends an exclusion predicate with AND: `key` is not in `values`.
    #[inline]
    #[must_use]
    pub fn not_in_set<I, T>(self, key: &str, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        Value: ValueConstructor<T>,
    {
        self.and_not_in_set(key, values)
    }

    /// Appends an existence predicate with AND.
    #[inline]
    #[must_use]
    pub fn exists(self, key: &str) -> Self {
        self.and_exists(key)
    }

    /// Appends a non-existence predicate with AND.
    #[inline]
    #[must_use]
    pub fn not_exists(self, key: &str) -> Self {
        self.and_not_exists(key)
    }

    /// Appends an equality predicate with AND: `key == value`.
    #[inline]
    #[must_use]
    pub fn and_eq<T>(self, key: &str, value: T) -> Self
    where
        Value: ValueConstructor<T>,
    {
        self.and_condition(Condition::Equal {
            key: key.to_string(),
            value: to_value(value),
        })
    }

    /// Appends a not-equal predicate with AND: `key != value`.
    #[inline]
    #[must_use]
    pub fn and_ne<T>(self, key: &str, value: T) -> Self
    where
        Value: ValueConstructor<T>,
    {
        self.and_condition(Condition::NotEqual {
            key: key.to_string(),
            value: to_value(value),
        })
    }

    /// Appends a less-than predicate with AND: `key < value`.
    #[inline]
    #[must_use]
    pub fn and_lt<T>(self, key: &str, value: T) -> Self
    where
        Value: ValueConstructor<T>,
    {
        self.and_condition(Condition::Less {
            key: key.to_string(),
            value: to_value(value),
        })
    }

    /// Appends a less-than-or-equal predicate with AND: `key <= value`.
    #[inline]
    #[must_use]
    pub fn and_le<T>(self, key: &str, value: T) -> Self
    where
        Value: ValueConstructor<T>,
    {
        self.and_condition(Condition::LessEqual {
            key: key.to_string(),
            value: to_value(value),
        })
    }

    /// Appends a greater-than predicate with AND: `key > value`.
    #[inline]
    #[must_use]
    pub fn and_gt<T>(self, key: &str, value: T) -> Self
    where
        Value: ValueConstructor<T>,
    {
        self.and_condition(Condition::Greater {
            key: key.to_string(),
            value: to_value(value),
        })
    }

    /// Appends a greater-than-or-equal predicate with AND: `key >= value`.
    #[inline]
    #[must_use]
    pub fn and_ge<T>(self, key: &str, value: T) -> Self
    where
        Value: ValueConstructor<T>,
    {
        self.and_condition(Condition::GreaterEqual {
            key: key.to_string(),
            value: to_value(value),
        })
    }

    /// Appends an inclusion predicate with AND: `key` is in `values`.
    #[inline]
    #[must_use]
    pub fn and_in_set<I, T>(self, key: &str, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        Value: ValueConstructor<T>,
    {
        self.and_condition(Condition::In {
            key: key.to_string(),
            values: Self::collect_values(values),
        })
    }

    /// Appends an exclusion predicate with AND: `key` is not in `values`.
    #[inline]
    #[must_use]
    pub fn and_not_in_set<I, T>(self, key: &str, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        Value: ValueConstructor<T>,
    {
        self.and_condition(Condition::NotIn {
            key: key.to_string(),
            values: Self::collect_values(values),
        })
    }

    /// Appends an existence predicate with AND.
    #[inline]
    #[must_use]
    pub fn and_exists(self, key: &str) -> Self {
        self.and_condition(Condition::Exists {
            key: key.to_string(),
        })
    }

    /// Appends a non-existence predicate with AND.
    #[inline]
    #[must_use]
    pub fn and_not_exists(self, key: &str) -> Self {
        self.and_condition(Condition::NotExists {
            key: key.to_string(),
        })
    }

    /// Appends an equality predicate with OR: `key == value`.
    #[inline]
    #[must_use]
    pub fn or_eq<T>(self, key: &str, value: T) -> Self
    where
        Value: ValueConstructor<T>,
    {
        self.or_condition(Condition::Equal {
            key: key.to_string(),
            value: to_value(value),
        })
    }

    /// Appends a not-equal predicate with OR: `key != value`.
    #[inline]
    #[must_use]
    pub fn or_ne<T>(self, key: &str, value: T) -> Self
    where
        Value: ValueConstructor<T>,
    {
        self.or_condition(Condition::NotEqual {
            key: key.to_string(),
            value: to_value(value),
        })
    }

    /// Appends a less-than predicate with OR: `key < value`.
    #[inline]
    #[must_use]
    pub fn or_lt<T>(self, key: &str, value: T) -> Self
    where
        Value: ValueConstructor<T>,
    {
        self.or_condition(Condition::Less {
            key: key.to_string(),
            value: to_value(value),
        })
    }

    /// Appends a less-than-or-equal predicate with OR: `key <= value`.
    #[inline]
    #[must_use]
    pub fn or_le<T>(self, key: &str, value: T) -> Self
    where
        Value: ValueConstructor<T>,
    {
        self.or_condition(Condition::LessEqual {
            key: key.to_string(),
            value: to_value(value),
        })
    }

    /// Appends a greater-than predicate with OR: `key > value`.
    #[inline]
    #[must_use]
    pub fn or_gt<T>(self, key: &str, value: T) -> Self
    where
        Value: ValueConstructor<T>,
    {
        self.or_condition(Condition::Greater {
            key: key.to_string(),
            value: to_value(value),
        })
    }

    /// Appends a greater-than-or-equal predicate with OR: `key >= value`.
    #[inline]
    #[must_use]
    pub fn or_ge<T>(self, key: &str, value: T) -> Self
    where
        Value: ValueConstructor<T>,
    {
        self.or_condition(Condition::GreaterEqual {
            key: key.to_string(),
            value: to_value(value),
        })
    }

    /// Appends an inclusion predicate with OR: `key` is in `values`.
    #[inline]
    #[must_use]
    pub fn or_in_set<I, T>(self, key: &str, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        Value: ValueConstructor<T>,
    {
        self.or_condition(Condition::In {
            key: key.to_string(),
            values: Self::collect_values(values),
        })
    }

    /// Appends an exclusion predicate with OR: `key` is not in `values`.
    #[inline]
    #[must_use]
    pub fn or_not_in_set<I, T>(self, key: &str, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        Value: ValueConstructor<T>,
    {
        self.or_condition(Condition::NotIn {
            key: key.to_string(),
            values: Self::collect_values(values),
        })
    }

    /// Appends an existence predicate with OR.
    #[inline]
    #[must_use]
    pub fn or_exists(self, key: &str) -> Self {
        self.or_condition(Condition::Exists {
            key: key.to_string(),
        })
    }

    /// Appends a non-existence predicate with OR.
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
    #[allow(clippy::should_implement_trait)]
    #[inline]
    #[must_use]
    pub fn not(mut self) -> Self {
        self.expr = Self::negate_expr(self.expr);
        self
    }

    /// Converts a sequence of typed values into stored values.
    #[inline]
    fn collect_values<I, T>(values: I) -> Vec<Value>
    where
        I: IntoIterator<Item = T>,
        Value: ValueConstructor<T>,
    {
        values.into_iter().map(to_value).collect()
    }

    /// Combines the current expression with `expr` using logical AND.
    #[inline]
    fn and_expr(mut self, expr: Option<FilterExpr>) -> Self {
        self.expr = match (self.expr, expr) {
            (None, rhs) => rhs,
            (lhs, None) => lhs,
            (Some(lhs), Some(rhs)) => Some(FilterExpr::and(lhs, rhs)),
        };
        self
    }

    /// Combines the current expression with `expr` using logical OR.
    #[inline]
    fn or_expr(mut self, expr: Option<FilterExpr>) -> Self {
        self.expr = match (self.expr, expr) {
            (None, rhs) => rhs,
            (lhs, None) => lhs,
            (Some(lhs), Some(rhs)) => Some(FilterExpr::or(lhs, rhs)),
        };
        self
    }

    /// Appends a condition with logical AND.
    #[inline]
    fn and_condition(self, condition: Condition) -> Self {
        let mut builder = self.and_expr(Some(FilterExpr::Condition(condition)));
        builder.has_condition = true;
        builder
    }

    /// Appends a condition with logical OR.
    #[inline]
    fn or_condition(self, condition: Condition) -> Self {
        let mut builder = self.or_expr(Some(FilterExpr::Condition(condition)));
        builder.has_condition = true;
        builder
    }

    /// Combines a grouped expression with logical AND.
    #[inline]
    fn and_group(self, operator: &'static str, group: Self) -> Self {
        self.combine_group(operator, group, Self::and_expr)
    }

    /// Combines a grouped expression with logical OR.
    #[inline]
    fn or_group(self, operator: &'static str, group: Self) -> Self {
        self.combine_group(operator, group, Self::or_expr)
    }

    /// Combines a grouped expression and records empty-group errors.
    #[inline]
    fn combine_group(
        self,
        operator: &'static str,
        group: Self,
        combine: fn(Self, Option<FilterExpr>) -> Self,
    ) -> Self {
        if let Some(error) = group.error {
            return self.with_error(error);
        }
        if !group.has_condition {
            return self.with_empty_group_error(operator);
        }
        let mut builder = combine(self, group.expr);
        builder.has_condition = true;
        builder
    }

    /// Negates the group expression while preserving empty-group detection.
    #[inline]
    fn negate_non_empty_group(mut self) -> Self {
        if self.expr.is_some() {
            self.expr = Self::negate_expr(self.expr);
        }
        self
    }

    /// Records a structural filter expression error.
    #[inline]
    fn with_error(mut self, error: MetadataError) -> Self {
        if self.error.is_none() {
            self.error = Some(error);
        }
        self
    }

    /// Records an empty grouped expression error.
    #[inline]
    fn with_empty_group_error(self, operator: &'static str) -> Self {
        self.with_error(MetadataError::InvalidFilterExpression {
            message: format!("empty '{operator}' filter group is not allowed"),
        })
    }

    /// Negates an optional expression.
    #[inline]
    pub(crate) fn negate_expr(expr: Option<FilterExpr>) -> Option<FilterExpr> {
        match expr {
            None => Some(FilterExpr::False),
            Some(FilterExpr::False) => None,
            Some(FilterExpr::Not(inner)) => Some(*inner),
            Some(other) => Some(FilterExpr::Not(Box::new(other))),
        }
    }
}

#[inline]
fn to_value<T>(value: T) -> Value
where
    Value: ValueConstructor<T>,
{
    <Value as ValueConstructor<T>>::from_type(value)
}
