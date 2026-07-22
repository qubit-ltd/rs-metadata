// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! [`FilterMatchOptions`] — policies for filter evaluation.

use qubit_datatype::NumericComparisonPolicy;
use serde::{Deserialize, Serialize};

use super::FilterMatchOptionsBuilder;

/// Match policies used when evaluating a [`crate::MetadataFilter`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct FilterMatchOptions {
    /// Policy for mixed numeric comparisons.
    numeric_comparison_policy: NumericComparisonPolicy,
}

impl FilterMatchOptions {
    /// Creates a fluent builder for match options.
    ///
    /// # Returns
    ///
    /// A builder configured with exact numeric comparison by default.
    #[inline]
    #[must_use]
    pub const fn builder() -> FilterMatchOptionsBuilder {
        FilterMatchOptionsBuilder::new()
    }

    /// Creates default filter match options.
    ///
    /// # Returns
    ///
    /// Options using [`NumericComparisonPolicy::Exact`].
    #[inline]
    pub const fn new() -> Self {
        Self::from_numeric_comparison_policy(NumericComparisonPolicy::Exact)
    }

    /// Creates options from a numeric comparison policy.
    ///
    /// # Parameters
    ///
    /// * `numeric_comparison_policy` - Policy applied to mixed numeric values.
    ///
    /// # Returns
    ///
    /// Options containing the supplied policy.
    #[inline(always)]
    pub(crate) const fn from_numeric_comparison_policy(
        numeric_comparison_policy: NumericComparisonPolicy,
    ) -> Self {
        Self {
            numeric_comparison_policy,
        }
    }

    /// Returns the mixed numeric comparison policy.
    ///
    /// # Returns
    ///
    /// The configured comparison policy.
    #[inline(always)]
    pub const fn numeric_comparison_policy(&self) -> NumericComparisonPolicy {
        self.numeric_comparison_policy
    }

    /// Replaces the mixed numeric comparison policy.
    ///
    /// # Parameters
    ///
    /// * `numeric_comparison_policy` - New comparison policy.
    ///
    /// # Returns
    ///
    /// This options object for mutable chaining.
    #[inline(always)]
    pub fn set_numeric_comparison_policy(
        &mut self,
        numeric_comparison_policy: NumericComparisonPolicy,
    ) -> &mut Self {
        self.numeric_comparison_policy = numeric_comparison_policy;
        self
    }

    /// Returns options using the supplied mixed numeric comparison policy.
    ///
    /// # Parameters
    ///
    /// * `numeric_comparison_policy` - New comparison policy.
    ///
    /// # Returns
    ///
    /// Updated options.
    #[inline(always)]
    pub const fn with_numeric_comparison_policy(
        mut self,
        numeric_comparison_policy: NumericComparisonPolicy,
    ) -> Self {
        self.numeric_comparison_policy = numeric_comparison_policy;
        self
    }
}
