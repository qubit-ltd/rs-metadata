/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! A single comparison predicate against one metadata key.

use serde::{
    Deserialize,
    Serialize,
};
use std::cmp::Ordering;

use serde_json::{
    Number,
    Value,
};

use crate::Metadata;

/// A single comparison operator applied to one metadata key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// Key is greater than value (numeric / string comparison).
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
    /// Key exists in the metadata (regardless of its value).
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

impl Condition {
    #[inline]
    pub(crate) fn matches(&self, meta: &Metadata) -> bool {
        match self {
            Condition::Equal { key, value } => meta.get_raw(key) == Some(value),
            Condition::NotEqual { key, value } => meta.get_raw(key) != Some(value),
            Condition::Less { key, value } => meta
                .get_raw(key)
                .is_some_and(|v| compare_values(v, value) == Some(Ordering::Less)),
            Condition::LessEqual { key, value } => meta.get_raw(key).is_some_and(|v| {
                matches!(
                    compare_values(v, value),
                    Some(Ordering::Less) | Some(Ordering::Equal)
                )
            }),
            Condition::Greater { key, value } => meta
                .get_raw(key)
                .is_some_and(|v| compare_values(v, value) == Some(Ordering::Greater)),
            Condition::GreaterEqual { key, value } => meta.get_raw(key).is_some_and(|v| {
                matches!(
                    compare_values(v, value),
                    Some(Ordering::Greater) | Some(Ordering::Equal)
                )
            }),
            Condition::In { key, values } => meta.get_raw(key).is_some_and(|v| values.contains(v)),
            Condition::NotIn { key, values } => {
                meta.get_raw(key).map_or(true, |v| !values.contains(v))
            }
            Condition::Exists { key } => meta.contains_key(key),
            Condition::NotExists { key } => !meta.contains_key(key),
        }
    }
}

/// Compares two [`Value`]s where both are the same numeric or string variant.
/// Returns `None` when the values are incomparable (different types).
#[inline]
fn compare_values(a: &Value, b: &Value) -> Option<Ordering> {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => compare_numbers(x, y),
        (Value::String(x), Value::String(y)) => x.partial_cmp(y),
        _ => None,
    }
}

const MAX_SAFE_INTEGER_F64_U64: u64 = 9_007_199_254_740_992; // 2^53
const I64_MIN_F64: f64 = -9_223_372_036_854_775_808.0; // -2^63
const I64_EXCLUSIVE_MAX_F64: f64 = 9_223_372_036_854_775_808.0; // 2^63
const U64_EXCLUSIVE_MAX_F64: f64 = 18_446_744_073_709_551_616.0; // 2^64

fn compare_numbers(a: &Number, b: &Number) -> Option<Ordering> {
    if let (Some(xi), Some(yi)) = (a.as_i64(), b.as_i64()) {
        return Some(xi.cmp(&yi));
    }
    if let (Some(xi), Some(yu)) = (a.as_i64(), b.as_u64()) {
        return Some(compare_i64_u64(xi, yu));
    }
    if let (Some(xu), Some(yi)) = (a.as_u64(), b.as_i64()) {
        return Some(compare_i64_u64(yi, xu).reverse());
    }
    if let (Some(xu), Some(yu)) = (a.as_u64(), b.as_u64()) {
        return Some(xu.cmp(&yu));
    }
    if let (Some(xi), Some(yf)) = (a.as_i64(), b.as_f64()) {
        return compare_i64_f64(xi, yf);
    }
    if let (Some(xf), Some(yi)) = (a.as_f64(), b.as_i64()) {
        return compare_i64_f64(yi, xf).map(Ordering::reverse);
    }
    if let (Some(xu), Some(yf)) = (a.as_u64(), b.as_f64()) {
        return compare_u64_f64(xu, yf);
    }
    if let (Some(xf), Some(yu)) = (a.as_f64(), b.as_u64()) {
        return compare_u64_f64(yu, xf).map(Ordering::reverse);
    }
    if let (Some(xf), Some(yf)) = (a.as_f64(), b.as_f64()) {
        return xf.partial_cmp(&yf);
    }

    // serde_json::Number always represents one of i64/u64/f64.
    unreachable!("Number must be representable as i64/u64/f64")
}

#[inline]
fn compare_i64_u64(x: i64, y: u64) -> Ordering {
    if x < 0 {
        Ordering::Less
    } else {
        (x as u64).cmp(&y)
    }
}

#[inline]
fn compare_i64_f64(x: i64, y: f64) -> Option<Ordering> {
    if y.fract() == 0.0 && (I64_MIN_F64..I64_EXCLUSIVE_MAX_F64).contains(&y) {
        // Integer-vs-integer path avoids precision loss for values > 2^53.
        return Some(x.cmp(&(y as i64)));
    }

    if x.unsigned_abs() <= MAX_SAFE_INTEGER_F64_U64 {
        return (x as f64).partial_cmp(&y);
    }

    None
}

#[inline]
fn compare_u64_f64(x: u64, y: f64) -> Option<Ordering> {
    if y < 0.0 {
        return Some(Ordering::Greater);
    }

    if y.fract() == 0.0 && (0.0..U64_EXCLUSIVE_MAX_F64).contains(&y) {
        // Integer-vs-integer path avoids precision loss for values > 2^53.
        return Some(x.cmp(&(y as u64)));
    }

    None
}
