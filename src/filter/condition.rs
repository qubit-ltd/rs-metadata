// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! A single comparison predicate against one metadata key.

use std::cmp::Ordering;
use std::fmt;

use qubit_datatype::NumericComparisonPolicy;
use qubit_redact::Redact;
use qubit_redact::RedactionWriter;
use qubit_redact::Redactor;
use qubit_redact::Sensitivity;
use qubit_value::Value;
use qubit_value::ValueRef;
use qubit_value::ValueWirePayloadRefV1;

use super::internal::MatchOutcome;
use crate::FilterLimitKind;
use crate::FilterLimits;
use crate::Metadata;
use crate::MetadataError;
use crate::MetadataResult;

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
    /// Validates that every comparison operand has stable matching and wire
    /// serialization semantics.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::InvalidFilterOperand`] when an operand is
    /// unset or cannot be represented by the V1 wire format.
    pub(crate) fn validate_operands(&self) -> MetadataResult<()> {
        match self {
            Self::Equal { value, .. } => validate_operand("eq", value),
            Self::NotEqual { value, .. } => validate_operand("ne", value),
            Self::Less { value, .. } => validate_operand("lt", value),
            Self::LessEqual { value, .. } => validate_operand("le", value),
            Self::Greater { value, .. } => validate_operand("gt", value),
            Self::GreaterEqual { value, .. } => validate_operand("ge", value),
            Self::In { values, .. } => validate_operands("in_set", values),
            Self::NotIn { values, .. } => validate_operands("not_in_set", values),
            Self::Exists { .. } | Self::NotExists { .. } => Ok(()),
        }
    }

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
                kind: FilterLimitKind::KeyBytes,
                value: key.len(),
                maximum: limits.max_key_bytes(),
            });
        }
        let values = match self {
            Self::In { values, .. } | Self::NotIn { values, .. } => values,
            _ => return Ok(()),
        };
        if values.len() > limits.max_set_values() {
            return Err(MetadataError::FilterLimitExceeded {
                kind: FilterLimitKind::SetValues,
                value: values.len(),
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
    pub(crate) fn evaluate(&self, meta: &Metadata, numeric_comparison_policy: NumericComparisonPolicy) -> MatchOutcome {
        match self {
            Condition::Equal { key, value } => evaluate_concrete(meta, key, |stored| {
                values_equal(stored, value, numeric_comparison_policy)
                    .map_or(MatchOutcome::Unknown, MatchOutcome::from_bool)
            }),
            Condition::NotEqual { key, value } => evaluate_concrete(meta, key, |stored| {
                values_equal(stored, value, numeric_comparison_policy)
                    .map_or(MatchOutcome::Unknown, |equal| MatchOutcome::from_bool(!equal))
            }),
            Condition::Less { key, value } => evaluate_concrete(meta, key, |stored| {
                compare_values(stored, value, numeric_comparison_policy).map_or(MatchOutcome::Unknown, |ordering| {
                    MatchOutcome::from_bool(ordering == Ordering::Less)
                })
            }),
            Condition::LessEqual { key, value } => evaluate_concrete(meta, key, |stored| {
                compare_values(stored, value, numeric_comparison_policy).map_or(MatchOutcome::Unknown, |ordering| {
                    MatchOutcome::from_bool(matches!(ordering, Ordering::Less | Ordering::Equal))
                })
            }),
            Condition::Greater { key, value } => evaluate_concrete(meta, key, |stored| {
                compare_values(stored, value, numeric_comparison_policy).map_or(MatchOutcome::Unknown, |ordering| {
                    MatchOutcome::from_bool(ordering == Ordering::Greater)
                })
            }),
            Condition::GreaterEqual { key, value } => evaluate_concrete(meta, key, |stored| {
                compare_values(stored, value, numeric_comparison_policy).map_or(MatchOutcome::Unknown, |ordering| {
                    MatchOutcome::from_bool(matches!(ordering, Ordering::Greater | Ordering::Equal))
                })
            }),
            Condition::In { key, values } => evaluate_concrete(meta, key, |stored| {
                evaluate_membership(stored, values, numeric_comparison_policy, false)
            }),
            Condition::NotIn { key, values } => evaluate_concrete(meta, key, |stored| {
                evaluate_membership(stored, values, numeric_comparison_policy, true)
            }),
            Condition::Exists { key } => MatchOutcome::from_bool(concrete_value(meta, key).is_some()),
            Condition::NotExists { key } => MatchOutcome::from_bool(concrete_value(meta, key).is_none()),
        }
    }

    /// Returns the metadata key referenced by this condition.
    #[inline]
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

/// Validates one filter comparison operand.
///
/// # Parameters
///
/// * `operator` - Stable name of the operator using the operand.
/// * `value` - Operand to validate.
///
/// # Errors
///
/// Returns [`MetadataError::InvalidFilterOperand`] when `value` is unset or
/// cannot be represented by the V1 wire format.
fn validate_operand(operator: &'static str, value: &Value) -> MetadataResult<()> {
    if value.is_unset() {
        return Err(MetadataError::InvalidFilterOperand {
            operator,
            data_type: value.data_type(),
            message: "filter operands must be concrete values".to_owned(),
        });
    }
    if ValueWirePayloadRefV1::try_from(value).is_err() {
        return Err(MetadataError::InvalidFilterOperand {
            operator,
            data_type: value.data_type(),
            message: "filter operands must be representable by the V1 wire format".to_owned(),
        });
    }
    Ok(())
}

/// Validates every operand in a set-membership condition.
///
/// # Parameters
///
/// * `operator` - Stable name of the set operator.
/// * `values` - Candidate values to validate.
///
/// # Errors
///
/// Returns the first invalid operand error.
fn validate_operands(operator: &'static str, values: &[Value]) -> MetadataResult<()> {
    values.iter().try_for_each(|value| validate_operand(operator, value))
}

impl Redact for Condition {
    /// Writes a diagnostic condition representation under the active policy.
    ///
    /// The condition node is entered before its discriminant is inspected. The
    /// synthetic operator label is derived from that discriminant and does not
    /// consume a field node. The source key and optional operand fields are
    /// admitted in source order. Accessors are invoked only after admission;
    /// operand access is skipped for opaque masking but remains available to a
    /// disabled policy that intentionally restores source values. A node-budget
    /// rejection adds one structural truncation field and terminates the branch
    /// without touching the rejected field.
    ///
    /// # Parameters
    ///
    /// * `writer` - Structured destination carrying the active policy and
    ///   cumulative domain budgets.
    fn write_redacted(&self, writer: &mut RedactionWriter<'_>) {
        let (operator, operand): (&str, Option<&dyn fmt::Debug>) = match self {
            Self::Equal { value, .. } => ("equal", Some(value)),
            Self::NotEqual { value, .. } => ("not_equal", Some(value)),
            Self::Less { value, .. } => ("less", Some(value)),
            Self::LessEqual { value, .. } => ("less_equal", Some(value)),
            Self::Greater { value, .. } => ("greater", Some(value)),
            Self::GreaterEqual { value, .. } => ("greater_equal", Some(value)),
            Self::In { values, .. } => ("in", Some(values)),
            Self::NotIn { values, .. } => ("not_in", Some(values)),
            Self::Exists { .. } => ("exists", None),
            Self::NotExists { .. } => ("not_exists", None),
        };
        writer.record("Condition", |fields| {
            fields.unredacted("operator", || operator);
            fields.unredacted("key", || self.key());
            if let Some(operand) = operand {
                fields.sensitive(Sensitivity::Secret, "value", || operand);
            }
        });
    }
}

impl fmt::Debug for Condition {
    /// Writes the strict-policy diagnostic representation.
    #[inline(always)]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = Redactor::strict().redact_text(self);
        let text = output.text_or_marker("<redaction incomplete>");
        formatter.write_str(text.as_ref())
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
    F: FnOnce(&Value) -> MatchOutcome,
{
    concrete_value(meta, key).map_or(MatchOutcome::Unknown, predicate)
}

/// Evaluates one membership condition while preserving unknown comparisons.
///
/// # Parameters
///
/// * `stored` - Concrete metadata value being matched.
/// * `candidates` - Values accepted or excluded by the condition.
/// * `numeric_comparison_policy` - Policy for mixed numeric comparisons.
/// * `negated` - Whether a matching candidate means the condition is false.
///
/// # Returns
///
/// `Unknown` when no candidate matches but at least one candidate cannot be
/// compared to `stored`; otherwise the normal inclusion or exclusion result.
fn evaluate_membership(
    stored: &Value,
    candidates: &[Value],
    numeric_comparison_policy: NumericComparisonPolicy,
    negated: bool,
) -> MatchOutcome {
    let mut unknown = false;
    for candidate in candidates {
        match values_equal(stored, candidate, numeric_comparison_policy) {
            Some(true) => return MatchOutcome::from_bool(!negated),
            Some(false) => {}
            None => unknown = true,
        }
    }
    if unknown {
        MatchOutcome::Unknown
    } else {
        MatchOutcome::from_bool(negated)
    }
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
fn values_equal(left: &Value, right: &Value, numeric_comparison_policy: NumericComparisonPolicy) -> Option<bool> {
    if left.is_numeric() && right.is_numeric() {
        return left
            .numeric_cmp(right, numeric_comparison_policy)
            .ok()
            .map(|ordering| ordering == Ordering::Equal);
    }
    if left.data_type() != right.data_type() {
        return None;
    }
    Some(left == right)
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
fn compare_values(left: &Value, right: &Value, numeric_comparison_policy: NumericComparisonPolicy) -> Option<Ordering> {
    if left.is_numeric() && right.is_numeric() {
        return left.numeric_cmp(right, numeric_comparison_policy).ok();
    }
    match (left.view(), right.view()) {
        (ValueRef::String(left), ValueRef::String(right)) => left.partial_cmp(right),
        _ => None,
    }
}
