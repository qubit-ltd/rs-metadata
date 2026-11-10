// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Filter resource-bound categories.

/// Identifies one resource bounded while constructing or decoding a filter.
///
/// # Examples
///
/// ```
/// use qubit_metadata::FilterLimitKind;
///
/// let kind = FilterLimitKind::Depth;
/// let description = match kind {
///     FilterLimitKind::Depth => "expression depth",
///     FilterLimitKind::Nodes => "expression node count",
///     FilterLimitKind::SetValues => "set-membership values",
///     FilterLimitKind::KeyBytes => "metadata key bytes",
///     _ => "other filter resource",
/// };
/// assert_eq!(description, "expression depth");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FilterLimitKind {
    /// Nested expression depth.
    Depth,
    /// Total expression node count.
    Nodes,
    /// Candidate values in one set-membership condition.
    SetValues,
    /// UTF-8 bytes in one metadata key.
    KeyBytes,
}
