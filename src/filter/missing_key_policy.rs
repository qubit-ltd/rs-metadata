/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! [`MissingKeyPolicy`] — how filters treat missing keys for negative predicates.

use serde::{Deserialize, Serialize};

/// Policy that controls how filters treat missing keys for negative predicates.
///
/// The policy only affects [`crate::Condition::NotEqual`] and [`crate::Condition::NotIn`].
/// Other predicates keep their existing semantics (`equal` requires presence,
/// `exists` / `not_exists` check presence directly, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MissingKeyPolicy {
    /// Missing keys satisfy negative predicates (`not_equal`, `not_in_values`).
    ///
    /// This is the historical behavior and therefore the default.
    #[default]
    Match,
    /// Missing keys do not satisfy negative predicates.
    NoMatch,
}

impl MissingKeyPolicy {
    #[inline]
    pub(crate) const fn matches_negative_predicates(self) -> bool {
        matches!(self, Self::Match)
    }
}
