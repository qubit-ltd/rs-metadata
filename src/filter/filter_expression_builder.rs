// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Fluent builder for [`crate::FilterExpression`].

use qubit_value::Value;

use crate::{Condition, FilterExpression, FilterLimits, MetadataError, MetadataResult};

/// Fluent builder for a non-empty [`FilterExpression`].
///
/// Predicates without an explicit connector are joined by logical AND.
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
        let expression = self
            .expression
            .ok_or(MetadataError::InvalidFilterExpression {
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
            Condition::Equal {
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
            Condition::NotEqual {
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
            Condition::Less {
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
            Condition::LessEqual {
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
            Condition::Greater {
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
            Condition::GreaterEqual {
                key: key.to_string(),
                value: to_value(value),
            },
            FilterExpression::and_unchecked,
        )
    }

    /// Appends a membership predicate with logical AND.
    #[inline]
    #[must_use]
    pub fn in_set<I, T>(self, key: &str, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Value>,
    {
        self.append(
            Condition::In {
                key: key.to_string(),
                values: collect_values(values),
            },
            FilterExpression::and_unchecked,
        )
    }

    /// Appends a non-membership predicate with logical AND.
    #[inline]
    #[must_use]
    pub fn not_in_set<I, T>(self, key: &str, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Value>,
    {
        self.append(
            Condition::NotIn {
                key: key.to_string(),
                values: collect_values(values),
            },
            FilterExpression::and_unchecked,
        )
    }

    /// Appends an existence predicate with logical AND.
    #[inline]
    #[must_use]
    pub fn exists(self, key: &str) -> Self {
        self.append(
            Condition::Exists {
                key: key.to_string(),
            },
            FilterExpression::and_unchecked,
        )
    }

    /// Appends a non-existence predicate with logical AND.
    #[inline]
    #[must_use]
    pub fn not_exists(self, key: &str) -> Self {
        self.append(
            Condition::NotExists {
                key: key.to_string(),
            },
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

    /// Adds one condition, deferring hard-limit validation to [`Self::build`].
    fn append(
        mut self,
        condition: Condition,
        combine: fn(FilterExpression, FilterExpression) -> FilterExpression,
    ) -> Self {
        if self.error.is_some() {
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
        self.expression = Some(expression);
        self
    }

    /// Combines a non-empty nested group and defers hard-limit validation to
    /// [`Self::build`].
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
        self.expression = Some(expression);
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

/// Converts typed values into stored representations.
fn collect_values<I, T>(values: I) -> Vec<Value>
where
    I: IntoIterator<Item = T>,
    T: Into<Value>,
{
    values.into_iter().map(to_value).collect()
}
