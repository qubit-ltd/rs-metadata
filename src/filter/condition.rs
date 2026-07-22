// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! A single comparison predicate against one metadata key.

use std::{cmp::Ordering, fmt};

use super::internal::MatchOutcome;
use crate::{FilterLimits, Metadata, MetadataError, MetadataResult};
use qubit_datatype::NumericComparisonPolicy;
use qubit_redact::{Redact, RedactionPolicy};
use qubit_value::Value;

/// A single comparison operator applied to one metadata key.
///
/// Equality, range, and membership conditions require a concrete stored value.
/// An absent key or [`Value::Unset`] produces an unknown outcome that remains
/// unknown under logical negation and therefore fails closed when matching.
/// Existence conditions define presence in terms of a concrete value.
#[derive(Clone, PartialEq)]
#[non_exhaustive]
pub enum Condition {
    /// Key equals value.
    Equal {
        /// The metadata key.
        key: String,
        /// The expected value.
        value: Value,
    },
    /// Key does not equal value.
    NotEqual {
        /// The metadata key.
        key: String,
        /// The value to compare against.
        value: Value,
    },
    /// Key is less than value.
    Less {
        /// The metadata key.
        key: String,
        /// The upper bound (exclusive).
        value: Value,
    },
    /// Key is less than or equal to value.
    LessEqual {
        /// The metadata key.
        key: String,
        /// The upper bound (inclusive).
        value: Value,
    },
    /// Key is greater than value.
    Greater {
        /// The metadata key.
        key: String,
        /// The lower bound (exclusive).
        value: Value,
    },
    /// Key is greater than or equal to value.
    GreaterEqual {
        /// The metadata key.
        key: String,
        /// The lower bound (inclusive).
        value: Value,
    },
    /// The stored value is one of the listed candidates.
    In {
        /// The metadata key.
        key: String,
        /// The set of acceptable values.
        values: Vec<Value>,
    },
    /// The stored value is not any of the listed candidates.
    NotIn {
        /// The metadata key.
        key: String,
        /// The set of excluded values.
        values: Vec<Value>,
    },
    /// The key stores a concrete value.
    ///
    /// An absent key or a key storing [`Value::Unset`] does not exist for
    /// filter matching.
    Exists {
        /// The metadata key.
        key: String,
    },
    /// The key does not store a concrete value.
    ///
    /// An absent key or a key storing [`Value::Unset`] satisfies this
    /// condition.
    NotExists {
        /// The metadata key.
        key: String,
    },
}

impl Condition {
    /// Validates this condition against resource limits.
    ///
    /// # Parameters
    ///
    /// * `limits` - Bounds to enforce.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the key and membership values fit within `limits`.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::FilterLimitExceeded`] when a key or membership
    /// condition exceeds its configured bound.
    pub(crate) fn validate_limits(&self, limits: FilterLimits) -> MetadataResult<()> {
        let key = self.key();
        if key.len() > limits.max_key_bytes() {
            return Err(MetadataError::FilterLimitExceeded {
                limit: "key length",
                maximum: limits.max_key_bytes(),
            });
        }
        let values = match self {
            Self::In { values, .. } | Self::NotIn { values, .. } => values,
            _ => return Ok(()),
        };
        if values.len() > limits.max_set_values() {
            return Err(MetadataError::FilterLimitExceeded {
                limit: "set values",
                maximum: limits.max_set_values(),
            });
        }
        Ok(())
    }

    /// Evaluates this condition against the supplied metadata.
    ///
    /// # Parameters
    ///
    /// * `meta` - Metadata object being matched.
    /// * `numeric_comparison_policy` - Policy for mixed numeric comparisons.
    ///
    /// # Returns
    ///
    /// A definite outcome for concrete comparisons and existence predicates,
    /// or [`MatchOutcome::Unknown`] when a comparison depends on an absent or
    /// unset value.
    pub(crate) fn evaluate(
        &self,
        meta: &Metadata,
        numeric_comparison_policy: NumericComparisonPolicy,
    ) -> MatchOutcome {
        match self {
            Condition::Equal { key, value } => evaluate_concrete(meta, key, |stored| {
                values_equal(stored, value, numeric_comparison_policy)
            }),
            Condition::NotEqual { key, value } => evaluate_concrete(meta, key, |stored| {
                !values_equal(stored, value, numeric_comparison_policy)
            }),
            Condition::Less { key, value } => evaluate_concrete(meta, key, |stored| {
                compare_values(stored, value, numeric_comparison_policy) == Some(Ordering::Less)
            }),
            Condition::LessEqual { key, value } => evaluate_concrete(meta, key, |stored| {
                matches!(
                    compare_values(stored, value, numeric_comparison_policy),
                    Some(Ordering::Less) | Some(Ordering::Equal)
                )
            }),
            Condition::Greater { key, value } => evaluate_concrete(meta, key, |stored| {
                compare_values(stored, value, numeric_comparison_policy) == Some(Ordering::Greater)
            }),
            Condition::GreaterEqual { key, value } => evaluate_concrete(meta, key, |stored| {
                matches!(
                    compare_values(stored, value, numeric_comparison_policy),
                    Some(Ordering::Greater) | Some(Ordering::Equal)
                )
            }),
            Condition::In { key, values } => evaluate_concrete(meta, key, |stored| {
                values
                    .iter()
                    .any(|value| values_equal(stored, value, numeric_comparison_policy))
            }),
            Condition::NotIn { key, values } => evaluate_concrete(meta, key, |stored| {
                values
                    .iter()
                    .all(|value| !values_equal(stored, value, numeric_comparison_policy))
            }),
            Condition::Exists { key } => {
                MatchOutcome::from_bool(concrete_value(meta, key).is_some())
            }
            Condition::NotExists { key } => {
                MatchOutcome::from_bool(concrete_value(meta, key).is_none())
            }
        }
    }

    /// Returns the metadata key referenced by this condition.
    #[inline(always)]
    fn key(&self) -> &str {
        match self {
            Self::Equal { key, .. }
            | Self::NotEqual { key, .. }
            | Self::Less { key, .. }
            | Self::LessEqual { key, .. }
            | Self::Greater { key, .. }
            | Self::GreaterEqual { key, .. }
            | Self::In { key, .. }
            | Self::NotIn { key, .. }
            | Self::Exists { key }
            | Self::NotExists { key } => key,
        }
    }
}

impl Redact for Condition {
    /// Writes a diagnostic condition representation without exposing operands.
    fn fmt_redacted(
        &self,
        _policy: &RedactionPolicy,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(
            formatter,
            "Condition {{ key: {:?}, value: <redacted> }}",
            self.key()
        )
    }
}

impl fmt::Debug for Condition {
    /// Writes the default-policy diagnostic representation.
    #[inline(always)]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_redacted(&RedactionPolicy::default(), formatter)
    }
}

/// Evaluates a predicate that requires a concrete stored value.
///
/// # Parameters
///
/// * `meta` - Metadata object being matched.
/// * `key` - Metadata key to inspect.
/// * `predicate` - Comparison to apply to a concrete value.
///
/// # Returns
///
/// The predicate result, or [`MatchOutcome::Unknown`] when the key is absent
/// or stores [`Value::Unset`].
#[inline]
fn evaluate_concrete<F>(meta: &Metadata, key: &str, predicate: F) -> MatchOutcome
where
    F: FnOnce(&Value) -> bool,
{
    concrete_value(meta, key).map_or(MatchOutcome::Unknown, |value| {
        MatchOutcome::from_bool(predicate(value))
    })
}

/// Returns the concrete metadata value stored under `key`.
///
/// # Parameters
///
/// * `meta` - Metadata object being matched.
/// * `key` - Metadata key to inspect.
///
/// # Returns
///
/// The stored value when it is concrete, or `None` when the key is absent or
/// stores [`Value::Unset`].
#[inline(always)]
fn concrete_value<'a>(meta: &'a Metadata, key: &str) -> Option<&'a Value> {
    meta.get_raw(key).filter(|value| !value.is_unset())
}

/// Compares two values for equality, treating numeric variants by numeric
/// value.
///
/// # Parameters
///
/// * `left` - Left comparison operand.
/// * `right` - Right comparison operand.
/// * `numeric_comparison_policy` - Policy for mixed numeric variants.
///
/// # Returns
///
/// `true` when both values compare equal under the supplied policy.
#[inline]
fn values_equal(
    left: &Value,
    right: &Value,
    numeric_comparison_policy: NumericComparisonPolicy,
) -> bool {
    if left.is_numeric() && right.is_numeric() {
        return left
            .numeric_cmp(right, numeric_comparison_policy)
            .is_ok_and(|ordering| ordering == Ordering::Equal);
    }
    left == right
}

/// Compares two numeric values or two strings.
///
/// # Parameters
///
/// * `left` - Left comparison operand.
/// * `right` - Right comparison operand.
/// * `numeric_comparison_policy` - Policy for mixed numeric variants.
///
/// # Returns
///
/// The relative ordering, or `None` when the values cannot be compared.
#[inline]
fn compare_values(
    left: &Value,
    right: &Value,
    numeric_comparison_policy: NumericComparisonPolicy,
) -> Option<Ordering> {
    if left.is_numeric() && right.is_numeric() {
        return left.numeric_cmp(right, numeric_comparison_policy).ok();
    }
    match (left, right) {
        (Value::String(left), Value::String(right)) => left.partial_cmp(right),
        _ => None,
    }
}
