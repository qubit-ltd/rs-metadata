// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Expression variant tag in the V1 wire representation.

use serde::Deserialize;

/// Expression variant tag in the V1 wire representation.
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ExpressionKind {
    /// Constant true expression.
    All,
    /// Constant false expression.
    None,
    /// Equality condition.
    Eq,
    /// Inequality condition.
    Ne,
    /// Less-than condition.
    Lt,
    /// Less-or-equal condition.
    Le,
    /// Greater-than condition.
    Gt,
    /// Greater-or-equal condition.
    Ge,
    /// Inclusion condition.
    In,
    /// Exclusion condition.
    NotIn,
    /// Existence condition.
    Exists,
    /// Non-existence condition.
    NotExists,
    /// Logical AND.
    And,
    /// Logical OR.
    Or,
    /// Logical negation.
    Not,
}
