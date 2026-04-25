/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Provides [`MetadataFilter`] and [`MetadataFilterBuilder`].

use qubit_value::{Value, ValueConstructor};
use serde::{Deserialize, Serialize};

use crate::{
    Condition, Metadata, MetadataResult, MetadataSchema, MissingKeyPolicy, NumberComparisonPolicy,
};

/// Match policies used when evaluating a [`MetadataFilter`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FilterMatchOptions {
    /// Policy for missing keys in negative predicates.
    pub missing_key_policy: MissingKeyPolicy,
    /// Policy for mixed numeric comparisons.
    pub number_comparison_policy: NumberComparisonPolicy,
}

/// Internal expression tree used by [`MetadataFilter`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum FilterExpr {
    /// A leaf condition.
    Condition(Condition),
    /// All child expressions must match.
    And(Vec<FilterExpr>),
    /// At least one child expression must match.
    Or(Vec<FilterExpr>),
    /// Negates the child expression.
    Not(Box<FilterExpr>),
    /// Constant false expression.
    False,
}

impl FilterExpr {
    /// Appends one expression to an AND node, flattening nested AND nodes.
    #[inline]
    fn append_and_child(children: &mut Vec<FilterExpr>, expr: FilterExpr) {
        match expr {
            FilterExpr::And(mut nested) => children.append(&mut nested),
            other => children.push(other),
        }
    }

    /// Appends one expression to an OR node, flattening nested OR nodes.
    #[inline]
    fn append_or_child(children: &mut Vec<FilterExpr>, expr: FilterExpr) {
        match expr {
            FilterExpr::Or(mut nested) => children.append(&mut nested),
            other => children.push(other),
        }
    }

    /// Builds an optimized AND expression from two child expressions.
    #[inline]
    fn and(lhs: FilterExpr, rhs: FilterExpr) -> FilterExpr {
        if matches!(lhs, FilterExpr::False) || matches!(rhs, FilterExpr::False) {
            return FilterExpr::False;
        }
        let mut children = Vec::new();
        Self::append_and_child(&mut children, lhs);
        Self::append_and_child(&mut children, rhs);
        FilterExpr::And(children)
    }

    /// Builds an optimized OR expression from two child expressions.
    #[inline]
    fn or(lhs: FilterExpr, rhs: FilterExpr) -> FilterExpr {
        if matches!(lhs, FilterExpr::False) {
            return rhs;
        }
        if matches!(rhs, FilterExpr::False) {
            return lhs;
        }
        let mut children = Vec::new();
        Self::append_or_child(&mut children, lhs);
        Self::append_or_child(&mut children, rhs);
        FilterExpr::Or(children)
    }

    /// Evaluates this expression tree against one metadata object.
    #[inline]
    fn matches(&self, meta: &Metadata, options: FilterMatchOptions) -> bool {
        match self {
            FilterExpr::Condition(condition) => condition.matches(
                meta,
                options.missing_key_policy,
                options.number_comparison_policy,
            ),
            FilterExpr::And(children) => children.iter().all(|expr| expr.matches(meta, options)),
            FilterExpr::Or(children) => children.iter().any(|expr| expr.matches(meta, options)),
            FilterExpr::Not(inner) => !inner.matches(meta, options),
            FilterExpr::False => false,
        }
    }

    /// Visits all leaf conditions in this expression tree.
    fn visit_conditions<F>(&self, visitor: &mut F) -> MetadataResult<()>
    where
        F: FnMut(&Condition) -> MetadataResult<()>,
    {
        match self {
            FilterExpr::Condition(condition) => visitor(condition),
            FilterExpr::And(children) | FilterExpr::Or(children) => {
                for child in children {
                    child.visit_conditions(visitor)?;
                }
                Ok(())
            }
            FilterExpr::Not(inner) => inner.visit_conditions(visitor),
            FilterExpr::False => Ok(()),
        }
    }
}

/// An immutable, composable filter expression over [`Metadata`].
///
/// Construct filters with [`MetadataFilter::builder`]. An empty builder builds a
/// match-all filter, which makes the default behavior explicit while keeping the
/// built filter immutable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MetadataFilter {
    /// Root expression tree. `None` means match all.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    expr: Option<FilterExpr>,
    /// Match policies used by [`MetadataFilter::matches`].
    #[serde(default)]
    options: FilterMatchOptions,
}

impl MetadataFilter {
    /// Creates a builder for a metadata filter.
    #[inline]
    #[must_use]
    pub fn builder() -> MetadataFilterBuilder {
        MetadataFilterBuilder::default()
    }

    /// Creates a filter that matches every metadata object.
    #[inline]
    #[must_use]
    pub fn all() -> Self {
        Self::default()
    }

    /// Creates a filter that matches no metadata object.
    #[inline]
    #[must_use]
    pub fn none() -> Self {
        Self {
            expr: Some(FilterExpr::False),
            options: FilterMatchOptions::default(),
        }
    }

    /// Returns the current match options.
    #[inline]
    #[must_use]
    pub fn options(&self) -> FilterMatchOptions {
        self.options
    }

    /// Replaces the current match options and returns a new filter.
    #[inline]
    #[must_use]
    pub fn with_options(mut self, options: FilterMatchOptions) -> Self {
        self.options = options;
        self
    }

    /// Returns a new filter with the supplied missing-key policy.
    #[inline]
    #[must_use]
    pub fn with_missing_key_policy(mut self, missing_key_policy: MissingKeyPolicy) -> Self {
        self.options.missing_key_policy = missing_key_policy;
        self
    }

    /// Returns a new filter with the supplied number-comparison policy.
    #[inline]
    #[must_use]
    pub fn with_number_comparison_policy(
        mut self,
        number_comparison_policy: NumberComparisonPolicy,
    ) -> Self {
        self.options.number_comparison_policy = number_comparison_policy;
        self
    }

    /// Returns a new filter that negates this filter.
    #[allow(clippy::should_implement_trait)]
    #[inline]
    #[must_use]
    pub fn not(mut self) -> Self {
        self.expr = MetadataFilterBuilder::negate_expr(self.expr);
        self
    }

    /// Returns `true` if `meta` satisfies this filter.
    #[inline]
    #[must_use]
    pub fn matches(&self, meta: &Metadata) -> bool {
        self.matches_with_options(meta, self.options)
    }

    /// Returns `true` if `meta` satisfies this filter with explicit options.
    #[inline]
    #[must_use]
    pub fn matches_with_options(&self, meta: &Metadata, options: FilterMatchOptions) -> bool {
        self.expr
            .as_ref()
            .is_none_or(|expr| expr.matches(meta, options))
    }

    /// Visits all leaf conditions in this filter.
    pub(crate) fn visit_conditions<F>(&self, mut visitor: F) -> MetadataResult<()>
    where
        F: FnMut(&Condition) -> MetadataResult<()>,
    {
        if let Some(expr) = &self.expr {
            expr.visit_conditions(&mut visitor)?;
        }
        Ok(())
    }
}

impl std::ops::Not for MetadataFilter {
    type Output = MetadataFilter;

    #[inline]
    fn not(self) -> Self::Output {
        MetadataFilter::not(self)
    }
}

/// Builder for [`MetadataFilter`].
///
/// Predicates without an explicit connector (`eq`, `gt`, `exists`, and so on)
/// are appended with logical AND. Use `or_*` methods or group methods for more
/// complex expressions.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MetadataFilterBuilder {
    /// Root expression being built. `None` means match all.
    expr: Option<FilterExpr>,
    /// Match policies copied into the built filter.
    options: FilterMatchOptions,
}

impl MetadataFilterBuilder {
    /// Builds an immutable [`MetadataFilter`].
    #[inline]
    #[must_use]
    pub fn build(self) -> MetadataFilter {
        MetadataFilter {
            expr: self.expr,
            options: self.options,
        }
    }

    /// Builds an immutable filter and validates it against `schema`.
    ///
    /// # Errors
    ///
    /// Returns an error when the filter references unknown schema fields, uses
    /// range operators on non-comparable field types, or compares a field with an
    /// incompatible value type.
    #[inline]
    pub fn build_checked(self, schema: &MetadataSchema) -> MetadataResult<MetadataFilter> {
        let filter = self.build();
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
        let group = build(Self::default()).expr;
        self.and_expr(group)
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
        let group = build(Self::default()).expr;
        self.or_expr(group)
    }

    /// Appends a negated grouped expression with AND.
    #[inline]
    #[must_use]
    pub fn and_not<F>(self, build: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        let group = build(Self::default()).expr;
        self.and_expr(Self::negate_expr(group))
    }

    /// Appends a negated grouped expression with OR.
    #[inline]
    #[must_use]
    pub fn or_not<F>(self, build: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        let group = build(Self::default()).expr;
        self.or_expr(Self::negate_expr(group))
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
        self.and_expr(Some(FilterExpr::Condition(condition)))
    }

    /// Appends a condition with logical OR.
    #[inline]
    fn or_condition(self, condition: Condition) -> Self {
        self.or_expr(Some(FilterExpr::Condition(condition)))
    }

    /// Negates an optional expression.
    #[inline]
    fn negate_expr(expr: Option<FilterExpr>) -> Option<FilterExpr> {
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
