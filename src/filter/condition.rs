// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! A single comparison predicate against one metadata key.

use std::cmp::Ordering;

use qubit_value::Value;
use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
};

use super::missing_key_policy::MissingKeyPolicy;
use super::wire::ConditionWire;
use crate::Metadata;
use qubit_datatype::NumericComparisonPolicy;

/// A single comparison operator applied to one metadata key.
#[derive(Debug, Clone, PartialEq)]
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
    /// Key exists in the metadata regardless of its value.
    Exists {
        /// The metadata key.
        key: String,
    },
    /// Key does not exist in the metadata.
    NotExists {
        /// The metadata key.
        key: String,
    },
}

impl Serialize for Condition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ConditionWire::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Condition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(ConditionWire::deserialize(deserializer)?.into_condition())
    }
}

impl Condition {
    /// Evaluates this condition against `meta` using the supplied policies.
    #[inline]
    pub(crate) fn matches(
        &self,
        meta: &Metadata,
        missing_key_policy: MissingKeyPolicy,
        numeric_comparison_policy: NumericComparisonPolicy,
    ) -> bool {
        match self {
            Condition::Equal { key, value } => {
                meta.get_raw(key).is_some_and(|stored| {
                    values_equal(stored, value, numeric_comparison_policy)
                })
            }
            Condition::NotEqual { key, value } => match meta.get_raw(key) {
                Some(stored) => {
                    !values_equal(stored, value, numeric_comparison_policy)
                }
                None => missing_key_policy.matches_negative_predicates(),
            },
            Condition::Less { key, value } => {
                meta.get_raw(key).is_some_and(|stored| {
                    compare_values(stored, value, numeric_comparison_policy)
                        == Some(Ordering::Less)
                })
            }
            Condition::LessEqual { key, value } => {
                meta.get_raw(key).is_some_and(|stored| {
                    matches!(
                        compare_values(
                            stored,
                            value,
                            numeric_comparison_policy
                        ),
                        Some(Ordering::Less) | Some(Ordering::Equal)
                    )
                })
            }
            Condition::Greater { key, value } => {
                meta.get_raw(key).is_some_and(|stored| {
                    compare_values(stored, value, numeric_comparison_policy)
                        == Some(Ordering::Greater)
                })
            }
            Condition::GreaterEqual { key, value } => {
                meta.get_raw(key).is_some_and(|stored| {
                    matches!(
                        compare_values(
                            stored,
                            value,
                            numeric_comparison_policy
                        ),
                        Some(Ordering::Greater) | Some(Ordering::Equal)
                    )
                })
            }
            Condition::In { key, values } => {
                meta.get_raw(key).is_some_and(|stored| {
                    values.iter().any(|value| {
                        values_equal(stored, value, numeric_comparison_policy)
                    })
                })
            }
            Condition::NotIn { key, values } => match meta.get_raw(key) {
                Some(stored) => values.iter().all(|value| {
                    !values_equal(stored, value, numeric_comparison_policy)
                }),
                None => missing_key_policy.matches_negative_predicates(),
            },
            Condition::Exists { key } => meta.contains_key(key),
            Condition::NotExists { key } => !meta.contains_key(key),
        }
    }
}

/// Compares two values for equality, treating numeric variants by numeric
/// value.
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
