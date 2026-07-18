// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! [`FilterMatchOptions`] — policies for filter evaluation.

use qubit_datatype::NumericComparisonPolicy;
use serde::{
    Deserialize,
    Serialize,
};

/// Match policies used when evaluating a [`crate::MetadataFilter`].
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default,
)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct FilterMatchOptions {
    /// Policy for mixed numeric comparisons.
    numeric_comparison_policy: NumericComparisonPolicy,
}

impl FilterMatchOptions {
    /// Creates default filter match options.
    ///
    /// # Returns
    ///
    /// Options using [`NumericComparisonPolicy::Exact`].
    #[inline]
    pub const fn new() -> Self {
        Self {
            numeric_comparison_policy: NumericComparisonPolicy::Exact,
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
