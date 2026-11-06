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
use serde::Deserializer;
use serde::Serialize;

use super::FilterMatchOptionsBuilder;
use super::internal::deserialize_filter_match_options;

/// Match policies used when evaluating a [`crate::MetadataFilter`].
///
/// # Examples
///
/// ```
/// use qubit_datatype::NumericComparisonPolicy;
/// use qubit_metadata::FilterMatchOptions;
///
/// let options = FilterMatchOptions::builder()
///     .numeric_comparison_policy(NumericComparisonPolicy::Exact)
///     .build();
/// assert_eq!(options.numeric_comparison_policy(), NumericComparisonPolicy::Exact);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
#[non_exhaustive]
pub struct FilterMatchOptions {
    /// Policy for mixed numeric comparisons.
    numeric_comparison_policy: NumericComparisonPolicy,
}

impl<'de> Deserialize<'de> for FilterMatchOptions {
    /// Decodes options from an object with one numeric comparison policy.
    ///
    /// # Type Parameters
    ///
    /// * `D` - The deserializer supplying the options representation.
    ///
    /// # Parameters
    ///
    /// * `deserializer` - The source representation to decode.
    ///
    /// # Returns
    ///
    /// The decoded filter match options.
    ///
    /// # Errors
    ///
    /// Returns an error if the representation is not a strict options object.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_filter_match_options(deserializer)
    }
}

impl FilterMatchOptions {
    /// Creates a fluent builder for match options.
    ///
    /// # Returns
    ///
    /// A builder configured with exact numeric comparison by default.
    #[inline]
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
    #[inline]
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
    #[inline]
    #[must_use = "the numeric comparison policy should be inspected"]
    pub const fn numeric_comparison_policy(&self) -> NumericComparisonPolicy {
        self.numeric_comparison_policy
    }
}
