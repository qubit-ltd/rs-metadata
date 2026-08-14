// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private storage for one filter expression node.

use crate::Condition;
use crate::FilterExpression;

/// Private representation of a filter expression.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FilterExpressionNode {
    /// A leaf condition.
    Condition(
        /// Stored comparison predicate.
        Condition,
    ),
    /// All child expressions must be true.
    And(
        /// Child expressions evaluated with logical AND.
        Vec<FilterExpression>,
    ),
    /// At least one child expression must be true.
    Or(
        /// Child expressions evaluated with logical OR.
        Vec<FilterExpression>,
    ),
    /// Three-valued logical negation.
    Not(
        /// Expression whose outcome is negated.
        Box<FilterExpression>,
    ),
    /// Constant true expression.
    True,
    /// Constant false expression.
    False,
}
