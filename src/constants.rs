// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Shared resource-limit constants for the metadata crate.

/// Hard maximum number of entries accepted by strict metadata maps.
pub(crate) const STRICT_STRING_MAP_MAX_ENTRIES: usize = 4_096;

/// Hard maximum UTF-8 byte length accepted for strict metadata map keys.
pub(crate) const STRICT_STRING_MAP_MAX_KEY_BYTES: usize = 256;

/// Hard maximum filter-expression nesting depth.
#[cfg(feature = "filter")]
pub(crate) const FILTER_MAX_DEPTH: usize = 64;

/// Hard maximum number of nodes in a filter expression.
#[cfg(feature = "filter")]
pub(crate) const FILTER_MAX_NODES: usize = 256;

/// Hard maximum membership values in one filter condition.
#[cfg(feature = "filter")]
pub(crate) const FILTER_MAX_SET_VALUES: usize = 128;

/// Hard maximum UTF-8 byte length of one filter key.
#[cfg(feature = "filter")]
pub(crate) const FILTER_MAX_KEY_BYTES: usize = 256;
