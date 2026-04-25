/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! [`NumberComparisonPolicy`] — mixed integer/float handling for range predicates.

use serde::{Deserialize, Serialize};

/// Policy that controls mixed integer/float number comparisons.
///
/// This policy only applies to range predicates (`greater`, `greater_equal`,
/// `less`, `less_equal`) when operands are numeric.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NumberComparisonPolicy {
    /// Preserve precision and return "incomparable" for risky mixed comparisons.
    ///
    /// This is the historical behavior and therefore the default.
    #[default]
    Conservative,
    /// Allow lossy `f64` fallback for mixed comparisons that are otherwise
    /// incomparable under [`NumberComparisonPolicy::Conservative`].
    Approximate,
}
