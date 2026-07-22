// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! [`MetadataFilter`].

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use super::metadata_filter_builder::MetadataFilterBuilder;
use super::wire::MetadataFilterWire;
use crate::{
    Condition, FilterExpression, FilterLimits, FilterMatchOptions, Metadata, MetadataResult,
};

/// An expression, its matching policy, and its resource limits.
///
/// Boolean composition belongs to [`FilterExpression`]. This type only binds
/// an already-built expression to the options and limits used to evaluate it.
#[derive(Debug, Clone, PartialEq)]
pub struct MetadataFilter {
    /// Root Boolean expression.
    expression: FilterExpression,
    /// Evaluation options.
    options: FilterMatchOptions,
    /// Bounds that the expression satisfies.
    limits: FilterLimits,
}

impl MetadataFilter {
    /// Creates a builder for a metadata filter.
    #[inline(always)]
    #[must_use]
    pub const fn builder() -> MetadataFilterBuilder {
        MetadataFilterBuilder::new()
    }

    /// Creates a filter from already validated parts.
    #[inline(always)]
    pub(crate) const fn new(
        expression: FilterExpression,
        options: FilterMatchOptions,
        limits: FilterLimits,
    ) -> Self {
        Self {
            expression,
            options,
            limits,
        }
    }

    /// Returns the root expression.
    #[inline(always)]
    pub const fn expression(&self) -> &FilterExpression {
        &self.expression
    }

    /// Returns the evaluation options.
    #[inline(always)]
    #[must_use]
    pub const fn options(&self) -> FilterMatchOptions {
        self.options
    }

    /// Returns the resource limits.
    #[inline(always)]
    #[must_use]
    pub const fn limits(&self) -> FilterLimits {
        self.limits
    }

    /// Returns whether `metadata` satisfies this filter.
    #[inline(always)]
    #[must_use]
    pub fn matches(&self, metadata: &Metadata) -> bool {
        self.expression.evaluate(metadata, self.options).is_match()
    }

    /// Visits every leaf condition in the expression.
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
}

impl Serialize for MetadataFilter {
    /// Serializes this filter through its versioned wire representation.
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        MetadataFilterWire::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MetadataFilter {
    /// Deserializes a filter using library hard limits as receiver limits.
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
