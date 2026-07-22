// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Fluent builder for [`crate::FilterExpression`].

use qubit_value::Value;

use crate::{
    Condition, FilterExpression, FilterLimits, IntoMetadataValue, MetadataError, MetadataResult,
};

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
    /// Returns [`MetadataError::InvalidFilterExpression`] for empty or invalid groups.
    #[inline]
    pub fn build(self) -> MetadataResult<FilterExpression> {
        if let Some(error) = self.error {
            return Err(error);
        }
        self.expression
            .ok_or(MetadataError::InvalidFilterExpression {
                message: "a filter expression must contain at least one condition".to_string(),
            })
    }

    /// Appends `key == value` with logical AND.
    #[inline(always)]
    #[must_use]
    pub fn eq<T>(self, key: &str, value: T) -> Self
    where
        T: IntoMetadataValue,
    {
        self.append(
            Condition::Equal {
                key: key.to_string(),
                value: to_value(value),
            },
            FilterExpression::and,
        )
    }

    /// Appends `key != value` with logical AND.
    #[inline(always)]
    #[must_use]
    pub fn ne<T>(self, key: &str, value: T) -> Self
    where
        T: IntoMetadataValue,
    {
        self.append(
            Condition::NotEqual {
                key: key.to_string(),
                value: to_value(value),
            },
            FilterExpression::and,
        )
    }

    /// Appends `key < value` with logical AND.
    #[inline(always)]
    #[must_use]
    pub fn lt<T>(self, key: &str, value: T) -> Self
    where
        T: IntoMetadataValue,
    {
        self.append(
            Condition::Less {
                key: key.to_string(),
                value: to_value(value),
            },
            FilterExpression::and,
        )
    }

    /// Appends `key <= value` with logical AND.
    #[inline(always)]
    #[must_use]
    pub fn le<T>(self, key: &str, value: T) -> Self
    where
        T: IntoMetadataValue,
    {
        self.append(
            Condition::LessEqual {
                key: key.to_string(),
                value: to_value(value),
            },
            FilterExpression::and,
        )
    }

    /// Appends `key > value` with logical AND.
    #[inline(always)]
    #[must_use]
    pub fn gt<T>(self, key: &str, value: T) -> Self
    where
        T: IntoMetadataValue,
    {
        self.append(
            Condition::Greater {
                key: key.to_string(),
                value: to_value(value),
            },
            FilterExpression::and,
        )
    }

    /// Appends `key >= value` with logical AND.
    #[inline(always)]
    #[must_use]
    pub fn ge<T>(self, key: &str, value: T) -> Self
    where
        T: IntoMetadataValue,
    {
        self.append(
            Condition::GreaterEqual {
                key: key.to_string(),
                value: to_value(value),
            },
            FilterExpression::and,
        )
    }

    /// Appends a membership predicate with logical AND.
    #[inline(always)]
    #[must_use]
    pub fn in_set<I, T>(self, key: &str, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: IntoMetadataValue,
    {
        self.append(
            Condition::In {
                key: key.to_string(),
                values: collect_values(values),
            },
            FilterExpression::and,
        )
    }

    /// Appends a non-membership predicate with logical AND.
    #[inline(always)]
    #[must_use]
    pub fn not_in_set<I, T>(self, key: &str, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: IntoMetadataValue,
    {
        self.append(
            Condition::NotIn {
                key: key.to_string(),
                values: collect_values(values),
            },
            FilterExpression::and,
        )
    }

    /// Appends an existence predicate with logical AND.
    #[inline(always)]
    #[must_use]
    pub fn exists(self, key: &str) -> Self {
        self.append(
            Condition::Exists {
                key: key.to_string(),
            },
            FilterExpression::and,
        )
    }

    /// Appends a non-existence predicate with logical AND.
    #[inline(always)]
    #[must_use]
    pub fn not_exists(self, key: &str) -> Self {
        self.append(
            Condition::NotExists {
                key: key.to_string(),
            },
            FilterExpression::and,
        )
    }

    /// Appends a grouped expression with logical AND.
    #[inline]
    #[must_use]
    pub fn and_group<F>(self, build: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        self.combine_group("AND", build(Self::new()), FilterExpression::and)
    }

    /// Appends a grouped expression with logical OR.
    #[inline]
    #[must_use]
    pub fn or_group<F>(self, build: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        self.combine_group("OR", build(Self::new()), FilterExpression::or)
    }

    /// Negates the expression built so far.
    #[allow(clippy::should_implement_trait)]
    #[inline]
    #[must_use]
    pub fn not(mut self) -> Self {
        if let Some(expression) = self.expression.take() {
            self.expression = Some(expression.negated());
        } else if self.error.is_none() {
            self.error = Some(MetadataError::InvalidFilterExpression {
                message: "cannot negate an empty filter expression".to_string(),
            });
        }
        self
    }

    /// Adds one condition with hard-limit validation.
    fn append(
        mut self,
        condition: Condition,
        combine: fn(FilterExpression, FilterExpression) -> FilterExpression,
    ) -> Self {
        if self.error.is_some() {
            return self;
        }
        let next = FilterExpression::condition(condition);
        let expression = match self.expression.take() {
            Some(previous) => combine(previous, next),
            None => next,
        };
        if let Err(error) = expression.validate_limits(FilterLimits::MAX) {
            self.error = Some(error);
        } else {
            self.expression = Some(expression);
        }
        self
    }

    /// Combines a non-empty nested group with hard-limit validation.
    fn combine_group(
        mut self,
        operator: &'static str,
        group: Self,
        combine: fn(FilterExpression, FilterExpression) -> FilterExpression,
    ) -> Self {
        if self.error.is_some() {
            return self;
        }
        let group = match group.build() {
            Ok(expression) => expression,
            Err(_) => {
                self.error = Some(MetadataError::InvalidFilterExpression {
                    message: if operator == "AND" {
                        "AND group must contain at least one condition".to_string()
                    } else {
                        "OR group must contain at least one condition".to_string()
                    },
                });
                return self;
            }
        };
        let expression = match self.expression.take() {
            Some(previous) => combine(previous, group),
            None => group,
        };
        if let Err(error) = expression.validate_limits(FilterLimits::MAX) {
            self.error = Some(error);
        } else {
            self.expression = Some(expression);
        }
        self
    }
}

/// Converts a typed value into its stored representation.
#[inline(always)]
fn to_value<T>(value: T) -> Value
where
    T: IntoMetadataValue,
{
    value.into_metadata_value()
}

/// Converts typed values into stored representations.
#[inline]
fn collect_values<I, T>(values: I) -> Vec<Value>
where
    I: IntoIterator<Item = T>,
    T: IntoMetadataValue,
{
    values.into_iter().map(to_value).collect()
}
