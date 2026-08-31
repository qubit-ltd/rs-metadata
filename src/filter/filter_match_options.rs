// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! [`FilterMatchOptions`] — policies for filter evaluation.

use qubit_datatype::NumericComparisonPolicy;
use serde::Deserialize;
use serde::Serialize;

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
    #[inline(always)]
    pub const fn builder() -> FilterMatchOptionsBuilder {
        FilterMatchOptionsBuilder::new()
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
    pub(crate) const fn from_numeric_comparison_policy(numeric_comparison_policy: NumericComparisonPolicy) -> Self {
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
    #[must_use = "the numeric comparison policy should be inspected"]
    pub const fn numeric_comparison_policy(&self) -> NumericComparisonPolicy {
        self.numeric_comparison_policy
    }
}
