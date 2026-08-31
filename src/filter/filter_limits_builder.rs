// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Fluent construction of validated filter resource limits.

use super::FilterLimits;
use crate::MetadataResult;

/// Fluent builder for validated filter limits.
#[must_use]
pub struct FilterLimitsBuilder {
    /// Requested maximum nesting depth.
    max_depth: Option<usize>,
    /// Requested maximum node count.
    max_nodes: Option<usize>,
    /// Requested maximum set-membership candidate count.
    max_set_values: Option<usize>,
    /// Requested maximum metadata-key byte length.
    max_key_bytes: Option<usize>,
}

impl FilterLimitsBuilder {
    /// Creates an empty builder that inherits every hard maximum by default.
    ///
    /// # Returns
    ///
    /// A builder with no explicit overrides.
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            max_depth: None,
            max_nodes: None,
            max_set_values: None,
            max_key_bytes: None,
        }
    }

    /// Sets the maximum nested expression depth.
    ///
    /// # Parameters
    ///
    /// * max_depth - Requested root-inclusive expression depth.
    ///
    /// # Returns
    ///
    /// The updated builder.
    #[inline(always)]
    pub const fn max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = Some(max_depth);
        self
    }

    /// Sets the maximum expression node count.
    ///
    /// # Parameters
    ///
    /// * max_nodes - Requested count including constants and logical nodes.
    ///
    /// # Returns
    ///
    /// The updated builder.
    #[inline(always)]
    pub const fn max_nodes(mut self, max_nodes: usize) -> Self {
        self.max_nodes = Some(max_nodes);
        self
    }

    /// Sets the maximum candidate count in one membership condition.
    ///
    /// # Parameters
    ///
    /// * max_set_values - Requested candidate-value count.
    ///
    /// # Returns
    ///
    /// The updated builder.
    #[inline(always)]
    pub const fn max_set_values(mut self, max_set_values: usize) -> Self {
        self.max_set_values = Some(max_set_values);
        self
    }

    /// Sets the maximum UTF-8 byte length of one metadata key.
    ///
    /// # Parameters
    ///
    /// * max_key_bytes - Requested byte bound.
    ///
    /// # Returns
    ///
    /// The updated builder.
    #[inline(always)]
    pub const fn max_key_bytes(mut self, max_key_bytes: usize) -> Self {
        self.max_key_bytes = Some(max_key_bytes);
        self
    }

    /// Builds validated resource limits.
    ///
    /// # Errors
    ///
    /// Returns a metadata error when a requested bound is zero or exceeds the
    /// library hard maximum.
    ///
    /// # Returns
    ///
    /// Limits using explicit values and hard maximums for unset properties.
    pub fn build(self) -> MetadataResult<FilterLimits> {
        FilterLimits::try_new(
            self.max_depth.unwrap_or(FilterLimits::MAX.max_depth()),
            self.max_nodes.unwrap_or(FilterLimits::MAX.max_nodes()),
            self.max_set_values.unwrap_or(FilterLimits::MAX.max_set_values()),
            self.max_key_bytes.unwrap_or(FilterLimits::MAX.max_key_bytes()),
        )
    }
}
