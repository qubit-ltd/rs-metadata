/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! A single comparison predicate against one metadata key.

use std::cmp::Ordering;

use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use qubit_value::Value;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::missing_key_policy::MissingKeyPolicy;
use super::number_comparison_policy::NumberComparisonPolicy;
use super::wire::ConditionWire;
use crate::Metadata;

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
        number_comparison_policy: NumberComparisonPolicy,
    ) -> bool {
        match self {
            Condition::Equal { key, value } => meta
                .get_raw(key)
                .is_some_and(|stored| values_equal(stored, value, number_comparison_policy)),
            Condition::NotEqual { key, value } => match meta.get_raw(key) {
                Some(stored) => !values_equal(stored, value, number_comparison_policy),
                None => missing_key_policy.matches_negative_predicates(),
            },
            Condition::Less { key, value } => meta.get_raw(key).is_some_and(|stored| {
                compare_values(stored, value, number_comparison_policy) == Some(Ordering::Less)
            }),
            Condition::LessEqual { key, value } => meta.get_raw(key).is_some_and(|stored| {
                matches!(
                    compare_values(stored, value, number_comparison_policy),
                    Some(Ordering::Less) | Some(Ordering::Equal)
                )
            }),
            Condition::Greater { key, value } => meta.get_raw(key).is_some_and(|stored| {
                compare_values(stored, value, number_comparison_policy) == Some(Ordering::Greater)
            }),
            Condition::GreaterEqual { key, value } => meta.get_raw(key).is_some_and(|stored| {
                matches!(
                    compare_values(stored, value, number_comparison_policy),
                    Some(Ordering::Greater) | Some(Ordering::Equal)
                )
            }),
            Condition::In { key, values } => meta.get_raw(key).is_some_and(|stored| {
                values
                    .iter()
                    .any(|value| values_equal(stored, value, number_comparison_policy))
            }),
            Condition::NotIn { key, values } => match meta.get_raw(key) {
                Some(stored) => values
                    .iter()
                    .all(|value| !values_equal(stored, value, number_comparison_policy)),
                None => missing_key_policy.matches_negative_predicates(),
            },
            Condition::Exists { key } => meta.contains_key(key),
            Condition::NotExists { key } => !meta.contains_key(key),
        }
    }
}

/// Compares two values for equality, treating numeric variants by numeric value.
#[inline]
fn values_equal(a: &Value, b: &Value, number_comparison_policy: NumberComparisonPolicy) -> bool {
    if is_numeric_value(a) && is_numeric_value(b) {
        return compare_numbers(a, b, number_comparison_policy) == Some(Ordering::Equal);
    }
    a == b
}

/// Compares two values where both are compatible numeric or string variants.
#[inline]
fn compare_values(
    a: &Value,
    b: &Value,
    number_comparison_policy: NumberComparisonPolicy,
) -> Option<Ordering> {
    if is_numeric_value(a) && is_numeric_value(b) {
        return compare_numbers(a, b, number_comparison_policy);
    }
    match (a, b) {
        (Value::String(x), Value::String(y)) => x.partial_cmp(y),
        _ => None,
    }
}

/// Internal normalized representation for scalar numeric comparisons.
#[derive(Debug, Clone, Copy)]
enum NumberValue {
    /// Signed integer value.
    Signed(i128),
    /// Unsigned integer value.
    Unsigned(u128),
    /// Floating-point value.
    Float(f64),
}

/// Returns `true` when `value` is one of the numeric `Value` variants.
#[inline]
fn is_numeric_value(value: &Value) -> bool {
    matches!(
        value,
        Value::Int8(_)
            | Value::Int16(_)
            | Value::Int32(_)
            | Value::Int64(_)
            | Value::Int128(_)
            | Value::UInt8(_)
            | Value::UInt16(_)
            | Value::UInt32(_)
            | Value::UInt64(_)
            | Value::UInt128(_)
            | Value::IntSize(_)
            | Value::UIntSize(_)
            | Value::Float32(_)
            | Value::Float64(_)
            | Value::BigInteger(_)
            | Value::BigDecimal(_)
    )
}

/// Converts a `Value` into the normalized numeric representation when supported.
#[inline]
fn number_value(value: &Value) -> Option<NumberValue> {
    match value {
        Value::Int8(v) => Some(NumberValue::Signed(i128::from(*v))),
        Value::Int16(v) => Some(NumberValue::Signed(i128::from(*v))),
        Value::Int32(v) => Some(NumberValue::Signed(i128::from(*v))),
        Value::Int64(v) => Some(NumberValue::Signed(i128::from(*v))),
        Value::Int128(v) => Some(NumberValue::Signed(*v)),
        Value::UInt8(v) => Some(NumberValue::Unsigned(u128::from(*v))),
        Value::UInt16(v) => Some(NumberValue::Unsigned(u128::from(*v))),
        Value::UInt32(v) => Some(NumberValue::Unsigned(u128::from(*v))),
        Value::UInt64(v) => Some(NumberValue::Unsigned(u128::from(*v))),
        Value::UInt128(v) => Some(NumberValue::Unsigned(*v)),
        Value::IntSize(v) => Some(NumberValue::Signed(*v as i128)),
        Value::UIntSize(v) => Some(NumberValue::Unsigned(*v as u128)),
        Value::Float32(v) => Some(NumberValue::Float(f64::from(*v))),
        Value::Float64(v) => Some(NumberValue::Float(*v)),
        _ => None,
    }
}

/// Compares two numeric `Value` variants with the configured precision policy.
#[inline]
fn compare_numbers(
    a: &Value,
    b: &Value,
    number_comparison_policy: NumberComparisonPolicy,
) -> Option<Ordering> {
    if contains_big_number(a, b) {
        return compare_big_numbers(a, b, number_comparison_policy);
    }
    match (number_value(a)?, number_value(b)?) {
        (NumberValue::Signed(x), NumberValue::Signed(y)) => Some(x.cmp(&y)),
        (NumberValue::Unsigned(x), NumberValue::Unsigned(y)) => Some(x.cmp(&y)),
        (NumberValue::Signed(x), NumberValue::Unsigned(y)) => Some(compare_i128_u128(x, y)),
        (NumberValue::Unsigned(x), NumberValue::Signed(y)) => {
            Some(compare_i128_u128(y, x).reverse())
        }
        (NumberValue::Signed(x), NumberValue::Float(y)) => {
            compare_i128_f64(x, y, number_comparison_policy)
        }
        (NumberValue::Float(x), NumberValue::Signed(y)) => {
            compare_i128_f64(y, x, number_comparison_policy).map(Ordering::reverse)
        }
        (NumberValue::Unsigned(x), NumberValue::Float(y)) => {
            compare_u128_f64(x, y, number_comparison_policy)
        }
        (NumberValue::Float(x), NumberValue::Unsigned(y)) => {
            compare_u128_f64(y, x, number_comparison_policy).map(Ordering::reverse)
        }
        (NumberValue::Float(x), NumberValue::Float(y)) => x.partial_cmp(&y),
    }
}

/// Returns `true` if either value is a big-number variant.
#[inline]
fn contains_big_number(a: &Value, b: &Value) -> bool {
    matches!(a, Value::BigInteger(_) | Value::BigDecimal(_))
        || matches!(b, Value::BigInteger(_) | Value::BigDecimal(_))
}

/// Compares values when at least one side is `BigInteger` or `BigDecimal`.
fn compare_big_numbers(
    a: &Value,
    b: &Value,
    number_comparison_policy: NumberComparisonPolicy,
) -> Option<Ordering> {
    if let (Some(x), Some(y)) = (big_integer_value(a), big_integer_value(b)) {
        return Some(x.cmp(&y));
    }
    if let (Some(x), Some(y)) = (big_decimal_value(a), big_decimal_value(b)) {
        return Some(x.cmp(&y));
    }
    if matches!(
        number_comparison_policy,
        NumberComparisonPolicy::Approximate
    ) {
        return compare_as_f64(a, b);
    }
    None
}

/// Converts integral numeric values to `BigInt` for exact comparison.
fn big_integer_value(value: &Value) -> Option<BigInt> {
    match value {
        Value::Int8(v) => Some(BigInt::from(*v)),
        Value::Int16(v) => Some(BigInt::from(*v)),
        Value::Int32(v) => Some(BigInt::from(*v)),
        Value::Int64(v) => Some(BigInt::from(*v)),
        Value::Int128(v) => Some(BigInt::from(*v)),
        Value::UInt8(v) => Some(BigInt::from(*v)),
        Value::UInt16(v) => Some(BigInt::from(*v)),
        Value::UInt32(v) => Some(BigInt::from(*v)),
        Value::UInt64(v) => Some(BigInt::from(*v)),
        Value::UInt128(v) => Some(BigInt::from(*v)),
        Value::IntSize(v) => Some(BigInt::from(*v)),
        Value::UIntSize(v) => Some(BigInt::from(*v)),
        Value::BigInteger(v) => Some(v.clone()),
        _ => None,
    }
}

/// Converts integral and decimal numeric values to `BigDecimal`.
fn big_decimal_value(value: &Value) -> Option<BigDecimal> {
    match value {
        Value::BigDecimal(v) => Some(v.clone()),
        _ => big_integer_value(value).map(BigDecimal::from),
    }
}

/// Compares two numeric values through the approximate `f64` fallback.
#[inline]
fn compare_as_f64(a: &Value, b: &Value) -> Option<Ordering> {
    a.to::<f64>().ok()?.partial_cmp(&b.to::<f64>().ok()?)
}

const MAX_SAFE_INTEGER_F64_U64: u64 = 9_007_199_254_740_992;
const I64_MIN_F64: f64 = -9_223_372_036_854_775_808.0;
const I64_EXCLUSIVE_MAX_F64: f64 = 9_223_372_036_854_775_808.0;
const U64_EXCLUSIVE_MAX_F64: f64 = 18_446_744_073_709_551_616.0;

/// Compares a signed integer and an unsigned integer without lossy casts.
#[inline]
fn compare_i128_u128(x: i128, y: u128) -> Ordering {
    if x < 0 {
        Ordering::Less
    } else {
        (x as u128).cmp(&y)
    }
}

/// Compares a signed integer and a float, returning `None` for risky cases.
fn compare_i128_f64(
    x: i128,
    y: f64,
    number_comparison_policy: NumberComparisonPolicy,
) -> Option<Ordering> {
    if let Ok(x64) = i64::try_from(x) {
        return compare_i64_f64(x64, y, number_comparison_policy);
    }
    if matches!(
        number_comparison_policy,
        NumberComparisonPolicy::Approximate
    ) {
        return (x as f64).partial_cmp(&y);
    }
    None
}

/// Compares an unsigned integer and a float, returning `None` for risky cases.
fn compare_u128_f64(
    x: u128,
    y: f64,
    number_comparison_policy: NumberComparisonPolicy,
) -> Option<Ordering> {
    if let Ok(x64) = u64::try_from(x) {
        return compare_u64_f64(x64, y, number_comparison_policy);
    }
    if matches!(
        number_comparison_policy,
        NumberComparisonPolicy::Approximate
    ) {
        return (x as f64).partial_cmp(&y);
    }
    None
}

/// Compares an `i64` and a float using the conservative JSON-era rules.
fn compare_i64_f64(
    x: i64,
    y: f64,
    number_comparison_policy: NumberComparisonPolicy,
) -> Option<Ordering> {
    if y.fract() == 0.0 && (I64_MIN_F64..I64_EXCLUSIVE_MAX_F64).contains(&y) {
        return Some(x.cmp(&(y as i64)));
    }

    if x.unsigned_abs() <= MAX_SAFE_INTEGER_F64_U64 {
        return (x as f64).partial_cmp(&y);
    }

    if matches!(
        number_comparison_policy,
        NumberComparisonPolicy::Approximate
    ) {
        return (x as f64).partial_cmp(&y);
    }

    None
}

/// Compares a `u64` and a float using the conservative JSON-era rules.
fn compare_u64_f64(
    x: u64,
    y: f64,
    number_comparison_policy: NumberComparisonPolicy,
) -> Option<Ordering> {
    if y < 0.0 {
        return Some(Ordering::Greater);
    }

    if y.fract() == 0.0 && (0.0..U64_EXCLUSIVE_MAX_F64).contains(&y) {
        return Some(x.cmp(&(y as u64)));
    }

    if x <= MAX_SAFE_INTEGER_F64_U64 {
        return (x as f64).partial_cmp(&y);
    }

    if matches!(
        number_comparison_policy,
        NumberComparisonPolicy::Approximate
    ) {
        return (x as f64).partial_cmp(&y);
    }

    None
}
