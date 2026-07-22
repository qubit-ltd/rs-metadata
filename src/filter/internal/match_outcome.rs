// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Three-valued result produced while evaluating a filter expression.

/// The internal truth value of a filter expression.
///
/// [`MatchOutcome::Unknown`] represents an expression that depends on a
/// missing key or a key storing `Value::Unset`. It remains unknown through
/// logical negation and is treated as a non-match at the public API boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub(crate) enum MatchOutcome {
    /// The expression is true.
    True,
    /// The expression is false.
    False,
    /// The expression cannot be decided because a concrete value is absent.
    Unknown,
}

impl MatchOutcome {
    /// Converts a Boolean value into a match outcome.
    ///
    /// # Parameters
    ///
    /// * `value` - Boolean value to convert.
    ///
    /// # Returns
    ///
    /// [`MatchOutcome::True`] for `true`; otherwise,
    /// [`MatchOutcome::False`].
    #[inline]
    pub(crate) const fn from_bool(value: bool) -> Self {
        if value { Self::True } else { Self::False }
    }

    /// Returns the three-valued logical negation of this outcome.
    ///
    /// # Returns
    ///
    /// The negated outcome. Unknown remains unknown.
    #[inline]
    pub(crate) const fn not(self) -> Self {
        match self {
            Self::True => Self::False,
            Self::False => Self::True,
            Self::Unknown => Self::Unknown,
        }
    }

    /// Combines outcomes with three-valued logical AND.
    ///
    /// # Parameters
    ///
    /// * `outcomes` - Outcomes to combine.
    ///
    /// # Returns
    ///
    /// False when any child is false, unknown when no child is false and at
    /// least one is unknown, or true otherwise.
    pub(crate) fn and<I>(outcomes: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        let mut result = Self::True;
        for outcome in outcomes {
            match outcome {
                Self::False => return Self::False,
                Self::Unknown => result = Self::Unknown,
                Self::True => {}
            }
        }
        result
    }

    /// Combines outcomes with three-valued logical OR.
    ///
    /// # Parameters
    ///
    /// * `outcomes` - Outcomes to combine.
    ///
    /// # Returns
    ///
    /// True when any child is true, unknown when no child is true and at
    /// least one is unknown, or false otherwise.
    pub(crate) fn or<I>(outcomes: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        let mut result = Self::False;
        for outcome in outcomes {
            match outcome {
                Self::True => return Self::True,
                Self::Unknown => result = Self::Unknown,
                Self::False => {}
            }
        }
        result
    }

    /// Reports whether this outcome is a definite match.
    ///
    /// # Returns
    ///
    /// `true` only for [`MatchOutcome::True`].
    #[inline(always)]
    pub(crate) const fn is_match(self) -> bool {
        matches!(self, Self::True)
    }
}
