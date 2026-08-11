// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Read-only views over filter expression nodes.

use crate::{
    Condition,
    FilterExpression,
};

/// A borrowed, read-only view of one [`FilterExpression`] node.
///
/// This enum preserves the complete Boolean structure without allowing
/// callers to construct or mutate the filter's private representation.
#[derive(Debug, Clone, Copy, PartialEq)]
#[must_use]
#[non_exhaustive]
pub enum FilterExpressionView<'a> {
    /// A leaf condition.
    Condition(
        /// Borrowed comparison predicate.
        &'a Condition,
    ),
    /// A logical AND over child expressions.
    And(
        /// Borrowed child expressions.
        &'a [FilterExpression],
    ),
    /// A logical OR over child expressions.
    Or(
        /// Borrowed child expressions.
        &'a [FilterExpression],
    ),
    /// A logical NOT around one expression.
    Not(
        /// Borrowed expression whose outcome is negated.
        &'a FilterExpression,
    ),
    /// A constant true expression.
    True,
    /// A constant false expression.
    False,
}
