// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! [`MetadataFilter`].

use qubit_datatype::NumericComparisonPolicy;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use super::filter_expression::FilterExpression;
use super::metadata_filter_builder::MetadataFilterBuilder;
use super::wire::MetadataFilterWire;
use crate::metadata::Metadata;
use crate::{Condition, FilterLimits, FilterMatchOptions, MetadataError, MetadataResult};

/// An immutable, composable filter expression over [`Metadata`].
///
/// Construct filters with [`MetadataFilter::builder`]. An empty builder builds
/// a match-all filter, while structurally invalid expressions such as empty
/// groups are rejected by [`MetadataFilterBuilder::build`]. Comparison
/// predicates over missing or unset values remain unknown through negation and
/// fail closed at the public matching boundary. Use
/// [`MetadataFilter::expression`] to inspect the immutable Boolean structure.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MetadataFilter {
    /// Root expression tree.
    expression: FilterExpression,
    /// Match policies used by [`MetadataFilter::matches`].
    options: FilterMatchOptions,
}

impl MetadataFilter {
    /// Creates a builder for a metadata filter.
    ///
    /// # Returns
    ///
    /// An empty builder whose default result matches all metadata.
    #[inline(always)]
    #[must_use]
    pub fn builder() -> MetadataFilterBuilder {
        MetadataFilterBuilder::default()
    }

    /// Creates a filter that matches every metadata object.
    ///
    /// # Returns
    ///
    /// A constant match-all filter.
    #[inline(always)]
    #[must_use]
    pub fn all() -> Self {
        Self::default()
    }

    /// Creates a filter that matches no metadata object.
    ///
    /// # Returns
    ///
    /// A constant match-none filter.
    #[inline]
    #[must_use]
    pub fn none() -> Self {
        Self {
            expression: FilterExpression::false_expression(),
            options: FilterMatchOptions::default(),
        }
    }

    /// Creates a filter from an expression and match options.
    ///
    /// # Parameters
    ///
    /// * `expression` - Root filter expression.
    /// * `options` - Match options stored with the filter.
    ///
    /// # Returns
    ///
    /// A new immutable filter.
    #[inline]
    pub(crate) const fn new(expression: FilterExpression, options: FilterMatchOptions) -> Self {
        Self {
            expression,
            options,
        }
    }

    /// Returns the root filter expression.
    ///
    /// # Returns
    ///
    /// A borrowed, immutable expression tree.
    #[inline(always)]
    pub fn expression(&self) -> &FilterExpression {
        &self.expression
    }

    /// Returns the current match options.
    ///
    /// # Returns
    ///
    /// A copy of the configured match options.
    #[inline(always)]
    #[must_use]
    pub fn options(&self) -> FilterMatchOptions {
        self.options
    }

    /// Replaces the current match options and returns a new filter.
    ///
    /// # Parameters
    ///
    /// * `options` - Replacement match options.
    ///
    /// # Returns
    ///
    /// This filter with the replacement options.
    #[inline(always)]
    #[must_use]
    pub fn with_options(mut self, options: FilterMatchOptions) -> Self {
        self.options = options;
        self
    }

    /// Returns a new filter with the supplied number-comparison policy.
    ///
    /// # Parameters
    ///
    /// * `numeric_comparison_policy` - Policy for mixed numeric comparisons.
    ///
    /// # Returns
    ///
    /// This filter with the replacement policy.
    #[inline(always)]
    #[must_use]
    pub fn with_numeric_comparison_policy(
        mut self,
        numeric_comparison_policy: NumericComparisonPolicy,
    ) -> Self {
        self.options
            .set_numeric_comparison_policy(numeric_comparison_policy);
        self
    }

    /// Returns a new filter that negates this filter.
    ///
    /// # Returns
    ///
    /// This filter with its root expression logically negated.
    #[allow(clippy::should_implement_trait)]
    #[inline(always)]
    #[must_use]
    pub fn not(mut self) -> Self {
        self.expression = self.expression.negated();
        self
    }

    /// Combines this filter with `other` using logical AND.
    ///
    /// # Parameters
    ///
    /// * `other` - Filter to combine with this filter.
    ///
    /// # Returns
    ///
    /// A filter that requires both operands to match.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::IncompatibleFilterOptions`] when the filters
    /// use different match options, or [`MetadataError::FilterLimitExceeded`]
    /// when their combined expression exceeds the default resource bounds.
    pub fn try_and(self, other: Self) -> MetadataResult<Self> {
        self.try_combine(other, FilterExpression::and)
    }

    /// Combines this filter with `other` using logical OR.
    ///
    /// # Parameters
    ///
    /// * `other` - Filter to combine with this filter.
    ///
    /// # Returns
    ///
    /// A filter that matches when either operand matches.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::IncompatibleFilterOptions`] when the filters
    /// use different match options, or [`MetadataError::FilterLimitExceeded`]
    /// when their combined expression exceeds the default resource bounds.
    pub fn try_or(self, other: Self) -> MetadataResult<Self> {
        self.try_combine(other, FilterExpression::or)
    }

    /// Returns `true` if `meta` satisfies this filter.
    ///
    /// # Parameters
    ///
    /// * `meta` - Metadata object to evaluate.
    ///
    /// # Returns
    ///
    /// `true` only when evaluation produces a definite true outcome.
    #[inline(always)]
    #[must_use]
    pub fn matches(&self, meta: &Metadata) -> bool {
        self.matches_with_options(meta, self.options)
    }

    /// Returns `true` if `meta` satisfies this filter with explicit options.
    ///
    /// # Parameters
    ///
    /// * `meta` - Metadata object to evaluate.
    /// * `options` - Match options for this evaluation.
    ///
    /// # Returns
    ///
    /// `true` only when evaluation produces a definite true outcome.
    #[inline(always)]
    #[must_use]
    pub fn matches_with_options(&self, meta: &Metadata, options: FilterMatchOptions) -> bool {
        self.expression.evaluate(meta, options).is_match()
    }

    /// Visits all leaf conditions in this filter.
    ///
    /// # Parameters
    ///
    /// * `visitor` - Callback invoked for each leaf condition.
    ///
    /// # Returns
    ///
    /// `Ok(())` after every condition has been visited.
    ///
    /// # Errors
    ///
    /// Returns the first error produced by `visitor`.
    #[inline(always)]
    pub(crate) fn visit_conditions<F>(&self, mut visitor: F) -> MetadataResult<()>
    where
        F: FnMut(&Condition) -> MetadataResult<()>,
    {
        self.expression.visit_conditions(&mut visitor)
    }

    /// Validates this filter against resource limits.
    ///
    /// # Parameters
    ///
    /// * `limits` - Bounds to enforce.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the expression fits within `limits`.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::FilterLimitExceeded`] when the expression
    /// exceeds a configured resource bound.
    pub fn validate_limits(&self, limits: FilterLimits) -> MetadataResult<()> {
        self.expression.validate_limits(limits)
    }

    /// Combines two filters that must share identical match options.
    fn try_combine(
        self,
        other: Self,
        combine: fn(FilterExpression, FilterExpression) -> FilterExpression,
    ) -> MetadataResult<Self> {
        if self.options != other.options {
            return Err(MetadataError::IncompatibleFilterOptions {
                left: self.options,
                right: other.options,
            });
        }
        let filter = Self::new(combine(self.expression, other.expression), self.options);
        filter.validate_limits(FilterLimits::default())?;
        Ok(filter)
    }
}

impl Serialize for MetadataFilter {
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        MetadataFilterWire::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MetadataFilter {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        MetadataFilterWire::deserialize(deserializer)?
            .into_filter()
            .map_err(de::Error::custom)
    }
}

impl std::ops::Not for MetadataFilter {
    type Output = MetadataFilter;

    #[inline(always)]
    fn not(self) -> Self::Output {
        MetadataFilter::not(self)
    }
}
