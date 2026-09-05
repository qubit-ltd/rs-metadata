// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Fluent builder for [`crate::FilterExpression`].

use qubit_value::Value;

use crate::Condition;
use crate::FilterExpression;
use crate::FilterLimitKind;
use crate::FilterLimits;
use crate::MetadataError;
use crate::MetadataResult;

/// Fluent builder for a non-empty [`FilterExpression`].
///
/// Predicates without an explicit connector are joined by logical AND.
/// Construction stops at the first invalid operand or resource limit. Later
/// operands are not converted, iterators are not consumed, and group callbacks
/// are not invoked. Argument expressions themselves are still evaluated by
/// Rust.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FilterExpressionBuilder {
    expression: Option<FilterExpression>,
    error: Option<MetadataError>,
}

impl FilterExpressionBuilder {
    /// Creates an empty expression builder.
    #[inline(always)]
    pub(crate) const fn new() -> Self {
        Self {
            expression: None,
            error: None,
        }
    }

    /// Builds the assembled expression.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::InvalidFilterExpression`] for empty or invalid
    /// groups, or [`MetadataError::FilterLimitExceeded`] for oversized trees.
    #[inline]
    pub fn build(self) -> MetadataResult<FilterExpression> {
        if let Some(error) = self.error {
            return Err(error);
        }
        let expression = self.expression.ok_or(MetadataError::InvalidFilterExpression {
            message: "a filter expression must contain at least one condition".to_string(),
        })?;
        expression.validate_limits(FilterLimits::MAX)?;
        Ok(expression)
    }

    /// Appends `key == value` with logical AND.
    #[inline]
    #[must_use]
    pub fn eq<T>(self, key: &str, value: T) -> Self
    where
        T: Into<Value>,
    {
        self.append(
            || Condition::Equal {
                key: key.to_string(),
                value: to_value(value),
            },
            FilterExpression::and_unchecked,
        )
    }

    /// Appends `key != value` with logical AND.
    #[inline]
    #[must_use]
    pub fn ne<T>(self, key: &str, value: T) -> Self
    where
        T: Into<Value>,
    {
        self.append(
            || Condition::NotEqual {
                key: key.to_string(),
                value: to_value(value),
            },
            FilterExpression::and_unchecked,
        )
    }

    /// Appends `key < value` with logical AND.
    #[inline]
    #[must_use]
    pub fn lt<T>(self, key: &str, value: T) -> Self
    where
        T: Into<Value>,
    {
        self.append(
            || Condition::Less {
                key: key.to_string(),
                value: to_value(value),
            },
            FilterExpression::and_unchecked,
        )
    }

    /// Appends `key <= value` with logical AND.
    #[inline]
    #[must_use]
    pub fn le<T>(self, key: &str, value: T) -> Self
    where
        T: Into<Value>,
    {
        self.append(
            || Condition::LessEqual {
                key: key.to_string(),
                value: to_value(value),
            },
            FilterExpression::and_unchecked,
        )
    }

    /// Appends `key > value` with logical AND.
    #[inline]
    #[must_use]
    pub fn gt<T>(self, key: &str, value: T) -> Self
    where
        T: Into<Value>,
    {
        self.append(
            || Condition::Greater {
                key: key.to_string(),
                value: to_value(value),
            },
            FilterExpression::and_unchecked,
        )
    }

    /// Appends `key >= value` with logical AND.
    #[inline]
    #[must_use]
    pub fn ge<T>(self, key: &str, value: T) -> Self
    where
        T: Into<Value>,
    {
        self.append(
            || Condition::GreaterEqual {
                key: key.to_string(),
                value: to_value(value),
            },
            FilterExpression::and_unchecked,
        )
    }

    /// Appends a membership predicate with logical AND.
    #[inline]
    #[must_use]
    pub fn in_set<I, T>(mut self, key: &str, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Value>,
    {
        if self.error.is_some() {
            return self;
        }
        if key.len() > FilterLimits::MAX.max_key_bytes() {
            self.error = Some(MetadataError::FilterLimitExceeded {
                kind: FilterLimitKind::KeyBytes,
                value: key.len(),
                maximum: FilterLimits::MAX.max_key_bytes(),
            });
            return self;
        }
        let values = match collect_values(values) {
            Ok(values) => values,
            Err(error) => {
                self.error = Some(error);
                return self;
            }
        };
        self.append(
            || Condition::In {
                key: key.to_string(),
                values,
            },
            FilterExpression::and_unchecked,
        )
    }

    /// Appends a non-membership predicate with logical AND.
    #[inline]
    #[must_use]
    pub fn not_in_set<I, T>(mut self, key: &str, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Value>,
    {
        if self.error.is_some() {
            return self;
        }
        if key.len() > FilterLimits::MAX.max_key_bytes() {
            self.error = Some(MetadataError::FilterLimitExceeded {
                kind: FilterLimitKind::KeyBytes,
                value: key.len(),
                maximum: FilterLimits::MAX.max_key_bytes(),
            });
            return self;
        }
        let values = match collect_values(values) {
            Ok(values) => values,
            Err(error) => {
                self.error = Some(error);
                return self;
            }
        };
        self.append(
            || Condition::NotIn {
                key: key.to_string(),
                values,
            },
            FilterExpression::and_unchecked,
        )
    }

    /// Appends an existence predicate with logical AND.
    #[inline]
    #[must_use]
    pub fn exists(self, key: &str) -> Self {
        self.append(
            || Condition::Exists { key: key.to_string() },
            FilterExpression::and_unchecked,
        )
    }

    /// Appends a non-existence predicate with logical AND.
    #[inline]
    #[must_use]
    pub fn not_exists(self, key: &str) -> Self {
        self.append(
            || Condition::NotExists { key: key.to_string() },
            FilterExpression::and_unchecked,
        )
    }

    /// Appends a grouped expression with logical AND.
    ///
    /// # Examples
    ///
    /// ```
    /// use qubit_metadata::FilterExpression;
    ///
    /// let expression = FilterExpression::builder()
    ///     .eq("tenant", "acme")
    ///     .and_group(|group| group.exists("region").ne("region", "blocked"))
    ///     .build()?;
    /// # Ok::<(), qubit_metadata::MetadataError>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn and_group<F>(self, build: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        if self.error.is_some() {
            return self;
        }
        self.combine_group("AND", build(Self::new()), FilterExpression::and_unchecked)
    }

    /// Appends a grouped expression with logical OR.
    ///
    /// # Examples
    ///
    /// ```
    /// use qubit_metadata::{
    ///     FilterExpression,
    ///     Metadata,
    ///     MetadataFilter,
    /// };
    ///
    /// let expression = FilterExpression::builder()
    ///     .eq("tenant", "acme")
    ///     .or_group(|group| {
    ///         group
    ///             .eq("priority", 1_i64)
    ///             .or_group(|alternative| alternative.eq("priority", 2_i64))
    ///     })
    ///     .build()?;
    /// let filter = MetadataFilter::builder()
    ///     .expression(expression)
    ///     .build()?;
    /// let metadata = Metadata::new()
    ///     .with("tenant", "other")
    ///     .with("priority", 2_i64);
    ///
    /// assert!(filter.matches(&metadata));
    /// # Ok::<(), qubit_metadata::MetadataError>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn or_group<F>(self, build: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        if self.error.is_some() {
            return self;
        }
        self.combine_group("OR", build(Self::new()), FilterExpression::or_unchecked)
    }

    /// Negates the expression built so far.
    ///
    /// # Examples
    ///
    /// ```
    /// use qubit_metadata::FilterExpression;
    ///
    /// let expression = FilterExpression::builder()
    ///     .eq("state", "deleted")
    ///     .not()
    ///     .build()?;
    /// # Ok::<(), qubit_metadata::MetadataError>(())
    /// ```
    #[allow(clippy::should_implement_trait)]
    #[inline]
    #[must_use]
    pub fn not(mut self) -> Self {
        if self.error.is_some() {
            return self;
        }
        let Some(expression) = self.expression.take() else {
            self.error = Some(MetadataError::InvalidFilterExpression {
                message: "cannot negate an empty filter expression".to_string(),
            });
            return self;
        };
        match expression.try_not() {
            Ok(expression) => self.expression = Some(expression),
            Err(error) => self.error = Some(error),
        }
        self
    }

    /// Adds a lazily constructed condition and enforces hard limits
    /// immediately.
    fn append(
        mut self,
        condition: impl FnOnce() -> Condition,
        combine: fn(FilterExpression, FilterExpression) -> FilterExpression,
    ) -> Self {
        if self.error.is_some() {
            return self;
        }
        let condition = condition();
        if let Err(error) = condition.validate_limits(FilterLimits::MAX) {
            self.error = Some(error);
            return self;
        }
        let next = match FilterExpression::condition(condition) {
            Ok(expression) => expression,
            Err(error) => {
                self.error = Some(error);
                return self;
            }
        };
        let expression = match self.expression.take() {
            Some(previous) => combine(previous, next),
            None => next,
        };
        match expression.validate_limits(FilterLimits::MAX) {
            Ok(()) => self.expression = Some(expression),
            Err(error) => self.error = Some(error),
        }
        self
    }

    /// Combines a non-empty nested group and immediately enforces hard limits.
    fn combine_group(
        mut self,
        operator: &'static str,
        group: Self,
        combine: fn(FilterExpression, FilterExpression) -> FilterExpression,
    ) -> Self {
        if self.error.is_some() {
            return self;
        }
        let group = match (group.error, group.expression) {
            (Some(error), _) => {
                self.error = Some(error);
                return self;
            }
            (None, Some(expression)) => expression,
            (None, None) => {
                self.error = Some(empty_group_error(operator));
                return self;
            }
        };
        let expression = match self.expression.take() {
            Some(previous) => combine(previous, group),
            None => group,
        };
        match expression.validate_limits(FilterLimits::MAX) {
            Ok(()) => self.expression = Some(expression),
            Err(error) => self.error = Some(error),
        }
        self
    }
}

/// Creates the validation error for an empty logical group.
///
/// # Parameters
///
/// * `operator` - Logical operator naming the empty group.
///
/// # Returns
///
/// An invalid-expression error whose message identifies `operator`.
fn empty_group_error(operator: &'static str) -> MetadataError {
    MetadataError::InvalidFilterExpression {
        message: format!("{operator} group must contain at least one condition"),
    }
}

/// Converts a typed value into its stored representation.
#[inline(always)]
fn to_value<T>(value: T) -> Value
where
    T: Into<Value>,
{
    value.into()
}

/// Collects at most the hard maximum, consuming one extra item to detect
/// overflow.
///
/// Returns a set-limit error without converting the rejected item.
fn collect_values<I, T>(values: I) -> MetadataResult<Vec<Value>>
where
    I: IntoIterator<Item = T>,
    T: Into<Value>,
{
    let maximum = FilterLimits::MAX.max_set_values();
    let mut result = Vec::new();
    for value in values {
        if result.len() == maximum {
            return Err(MetadataError::FilterLimitExceeded {
                kind: FilterLimitKind::SetValues,
                value: maximum + 1,
                maximum,
            });
        }
        result.push(value.into());
    }
    Ok(result)
}
