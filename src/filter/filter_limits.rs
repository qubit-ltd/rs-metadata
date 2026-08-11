// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! [`FilterLimits`] — resource bounds for metadata filters.

use crate::constants::{
    FILTER_MAX_DEPTH, FILTER_MAX_KEY_BYTES, FILTER_MAX_NODES, FILTER_MAX_SET_VALUES,
};
use crate::{MetadataError, MetadataResult};

use super::{FilterLimitKind, FilterLimitsBuilder};

/// Resource bounds enforced for every constructed or deserialized filter.
///
/// The defaults keep filters small enough for predictable allocation and
/// recursive evaluation while leaving room for realistic application queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub struct FilterLimits {
    /// Maximum nesting depth of an expression tree.
    max_depth: usize,
    /// Maximum number of expression nodes, including constants and groups.
    max_nodes: usize,
    /// Maximum number of candidate values in one membership condition.
    max_set_values: usize,
    /// Maximum UTF-8 byte length of one metadata key.
    max_key_length: usize,
}

impl FilterLimits {
    /// Library-wide hard maximums for every resource bound.
    pub const MAX: Self = Self {
        max_depth: FILTER_MAX_DEPTH,
        max_nodes: FILTER_MAX_NODES,
        max_set_values: FILTER_MAX_SET_VALUES,
        max_key_length: FILTER_MAX_KEY_BYTES,
    };

    /// Creates a fluent builder for validated resource limits.
    ///
    /// # Returns
    ///
    /// A builder whose omitted properties use library hard maximums.
    #[inline(always)]
    pub const fn builder() -> FilterLimitsBuilder {
        FilterLimitsBuilder::new()
    }

    /// Validates and creates resource limits.
    ///
    /// # Parameters
    ///
    /// * `max_depth` - Maximum root-inclusive expression depth.
    /// * `max_nodes` - Maximum expression node count.
    /// * `max_set_values` - Maximum membership candidate count.
    /// * `max_key_bytes` - Maximum UTF-8 byte length of one metadata key.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::InvalidFilterLimit`] when a value is zero or
    /// exceeds the corresponding hard maximum.
    ///
    /// # Returns
    ///
    /// Validated resource limits.
    pub(crate) fn try_new(
        max_depth: usize,
        max_nodes: usize,
        max_set_values: usize,
        max_key_bytes: usize,
    ) -> MetadataResult<Self> {
        Self::validate_limit(FilterLimitKind::Depth, max_depth, Self::MAX.max_depth)?;
        Self::validate_limit(FilterLimitKind::Nodes, max_nodes, Self::MAX.max_nodes)?;
        Self::validate_limit(
            FilterLimitKind::SetValues,
            max_set_values,
            Self::MAX.max_set_values,
        )?;
        Self::validate_limit(
            FilterLimitKind::KeyBytes,
            max_key_bytes,
            Self::MAX.max_key_length,
        )?;
        Ok(Self {
            max_depth,
            max_nodes,
            max_set_values,
            max_key_length: max_key_bytes,
        })
    }

    /// Returns the maximum nesting depth.
    #[inline(always)]
    #[must_use]
    pub const fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Returns the maximum number of expression nodes.
    #[inline(always)]
    #[must_use]
    pub const fn max_nodes(&self) -> usize {
        self.max_nodes
    }

    /// Returns the maximum candidate values in one membership condition.
    #[inline(always)]
    #[must_use]
    pub const fn max_set_values(&self) -> usize {
        self.max_set_values
    }

    /// Returns the maximum UTF-8 byte length of one metadata key.
    ///
    /// # Returns
    ///
    /// The configured key byte bound.
    #[inline(always)]
    #[must_use]
    pub const fn max_key_bytes(&self) -> usize {
        self.max_key_length
    }

    /// Validates one configured resource bound.
    ///
    /// # Parameters
    ///
    /// * `kind` - Category of the bounded resource.
    /// * `value` - Requested bound.
    /// * `maximum` - Library hard maximum.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::InvalidFilterLimit`] when `value` is zero or
    /// greater than `maximum`.
    fn validate_limit(kind: FilterLimitKind, value: usize, maximum: usize) -> MetadataResult<()> {
        if value == 0 || value > maximum {
            return Err(MetadataError::InvalidFilterLimit {
                kind,
                value,
                maximum,
            });
        }
        Ok(())
    }
}

impl Default for FilterLimits {
    #[inline]
    fn default() -> Self {
        Self::MAX
    }
}
