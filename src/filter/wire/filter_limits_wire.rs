// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Versioned-wire representation of [`crate::FilterLimits`].

use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    FilterLimits,
    MetadataResult,
};

/// Serializable filter resource limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FilterLimitsWire {
    /// Maximum expression depth.
    max_depth: usize,
    /// Maximum expression nodes.
    max_nodes: usize,
    /// Maximum values in one membership condition.
    max_set_values: usize,
    /// Maximum key bytes.
    max_key_bytes: usize,
}

impl FilterLimitsWire {
    /// Converts this wire value into validated limits.
    ///
    /// # Errors
    ///
    /// Returns a limit-configuration error when any value is invalid.
    pub(crate) fn into_limits(self) -> MetadataResult<FilterLimits> {
        FilterLimits::builder()
            .max_depth(self.max_depth)
            .max_nodes(self.max_nodes)
            .max_set_values(self.max_set_values)
            .max_key_bytes(self.max_key_bytes)
            .build()
    }
}

impl From<FilterLimits> for FilterLimitsWire {
    /// Converts validated limits into their wire representation.
    #[inline(always)]
    fn from(limits: FilterLimits) -> Self {
        Self {
            max_depth: limits.max_depth(),
            max_nodes: limits.max_nodes(),
            max_set_values: limits.max_set_values(),
            max_key_bytes: limits.max_key_bytes(),
        }
    }
}
