/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! [`NumberComparisonPolicy`] — mixed numeric comparison handling.

use serde::{Deserialize, Serialize};

/// Policy that controls mixed numeric comparisons.
///
/// This policy applies to numeric equality, membership, and range predicates
/// when operands use different numeric representations.
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
