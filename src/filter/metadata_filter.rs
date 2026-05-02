/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! [`MetadataFilter`].
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use super::filter_expr::FilterExpr;
use super::metadata_filter_builder::MetadataFilterBuilder;
use super::wire::MetadataFilterWire;
use crate::metadata::Metadata;
use crate::{
    Condition, FilterMatchOptions, MetadataResult, MissingKeyPolicy, NumberComparisonPolicy,
};

/// An immutable, composable filter expression over [`Metadata`].
///
/// Construct filters with [`MetadataFilter::builder`]. An empty builder builds a
/// match-all filter, while structurally invalid expressions such as empty groups
/// are rejected by [`MetadataFilterBuilder::build`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MetadataFilter {
    /// Root expression tree. `None` means match all.
    pub(crate) expr: Option<FilterExpr>,
    /// Match policies used by [`MetadataFilter::matches`].
    pub(crate) options: FilterMatchOptions,
}

impl MetadataFilter {
    /// Creates a filter from expression and options.
    #[inline]
    pub(crate) fn new(expr: Option<FilterExpr>, options: FilterMatchOptions) -> Self {
        Self { expr, options }
    }

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

impl Serialize for MetadataFilter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        MetadataFilterWire::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MetadataFilter {
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

    #[inline]
    fn not(self) -> Self::Output {
        MetadataFilter::not(self)
    }
}
