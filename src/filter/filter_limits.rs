// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! [`FilterLimits`] — resource bounds for metadata filters.

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
    /// Creates filter limits from explicit bounds.
    ///
    /// # Parameters
    ///
    /// * `max_depth` - Maximum nesting depth.
    /// * `max_nodes` - Maximum expression-node count.
    /// * `max_set_values` - Maximum values in one membership condition.
    /// * `max_key_length` - Maximum UTF-8 byte length of one key.
    ///
    /// # Returns
    ///
    /// The configured resource bounds.
    #[inline]
    pub const fn new(
        max_depth: usize,
        max_nodes: usize,
        max_set_values: usize,
        max_key_length: usize,
    ) -> Self {
        Self {
            max_depth,
            max_nodes,
            max_set_values,
            max_key_length,
        }
    }

    /// Returns the maximum nesting depth.
    #[inline(always)]
    pub const fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Returns the maximum number of expression nodes.
    #[inline(always)]
    pub const fn max_nodes(&self) -> usize {
        self.max_nodes
    }

    /// Returns the maximum candidate values in one membership condition.
    #[inline(always)]
    pub const fn max_set_values(&self) -> usize {
        self.max_set_values
    }

    /// Returns the maximum UTF-8 byte length of one metadata key.
    #[inline(always)]
    pub const fn max_key_length(&self) -> usize {
        self.max_key_length
    }
}

impl Default for FilterLimits {
    #[inline]
    fn default() -> Self {
        Self::new(64, 256, 128, 256)
    }
}
