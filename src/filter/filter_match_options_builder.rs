// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Fluent construction of filter match options.

use qubit_datatype::NumericComparisonPolicy;

use super::FilterMatchOptions;

/// Fluent builder for filter match options.
#[must_use]
pub struct FilterMatchOptionsBuilder {
    /// Mixed numeric comparison policy.
    numeric_comparison_policy: NumericComparisonPolicy,
}

impl FilterMatchOptionsBuilder {
    /// Creates a builder using exact numeric comparison by default.
    ///
    /// # Returns
    ///
    /// A builder with the default match policy.
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            numeric_comparison_policy: NumericComparisonPolicy::Exact,
        }
    }

    /// Sets the mixed numeric comparison policy.
    ///
    /// # Parameters
    ///
    /// * numeric_comparison_policy - Policy applied to mixed numeric values.
    ///
    /// # Returns
    ///
    /// The updated builder.
    #[inline(always)]
    pub const fn numeric_comparison_policy(mut self, numeric_comparison_policy: NumericComparisonPolicy) -> Self {
        self.numeric_comparison_policy = numeric_comparison_policy;
        self
    }

    /// Builds immutable match options.
    ///
    /// # Returns
    ///
    /// Options containing the selected comparison policy.
    #[inline(always)]
    pub const fn build(self) -> FilterMatchOptions {
        FilterMatchOptions::from_numeric_comparison_policy(self.numeric_comparison_policy)
    }
}
