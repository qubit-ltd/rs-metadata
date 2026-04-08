/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Provides [`MetadataFilter`] — composable filter expressions for
//! metadata-based queries.
//!
//! A [`MetadataFilter`] can be used to select [`Metadata`] instances that
//! satisfy a set of conditions.  Conditions can be combined with logical
//! operators (`and`, `or`, `not`) to form arbitrarily complex predicates.
//!
//! # Examples
//!
//! ```rust
//! use qubit_metadata::{Metadata, MetadataFilter};
//!
//! let mut meta = Metadata::new();
//! meta.set("status", "active");
//! meta.set("score", 42_i64);
//!
//! let filter = MetadataFilter::eq("status", "active")
//!     .and(MetadataFilter::gte("score", 10_i64));
//!
//! assert!(filter.matches(&meta));
//! ```

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
    Eq {
        /// The metadata key.
        key: String,
        /// The expected value.
        value: Value,
    },
    /// Key does not equal value.
    Ne {
        /// The metadata key.
        key: String,
        /// The value to compare against.
        value: Value,
    },
    /// Key is greater than value (numeric / string comparison).
    Gt {
        /// The metadata key.
        key: String,
        /// The lower bound (exclusive).
        value: Value,
    },
    /// Key is greater than or equal to value.
    Gte {
        /// The metadata key.
        key: String,
        /// The lower bound (inclusive).
        value: Value,
    },
    /// Key is less than value.
    Lt {
        /// The metadata key.
        key: String,
        /// The upper bound (exclusive).
        value: Value,
    },
    /// Key is less than or equal to value.
    Lte {
        /// The metadata key.
        key: String,
        /// The upper bound (inclusive).
        value: Value,
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
}

impl Condition {
    fn matches(&self, meta: &Metadata) -> bool {
        match self {
            Condition::Eq { key, value } => meta.get_raw(key) == Some(value),
            Condition::Ne { key, value } => meta.get_raw(key) != Some(value),
            Condition::Gt { key, value } => meta
                .get_raw(key)
                .is_some_and(|v| compare_values(v, value) == Some(Ordering::Greater)),
            Condition::Gte { key, value } => meta.get_raw(key).is_some_and(|v| {
                matches!(
                    compare_values(v, value),
                    Some(Ordering::Greater) | Some(Ordering::Equal)
                )
            }),
            Condition::Lt { key, value } => meta
                .get_raw(key)
                .is_some_and(|v| compare_values(v, value) == Some(Ordering::Less)),
            Condition::Lte { key, value } => meta.get_raw(key).is_some_and(|v| {
                matches!(
                    compare_values(v, value),
                    Some(Ordering::Less) | Some(Ordering::Equal)
                )
            }),
            Condition::Exists { key } => meta.contains_key(key),
            Condition::NotExists { key } => !meta.contains_key(key),
            Condition::In { key, values } => meta.get_raw(key).is_some_and(|v| values.contains(v)),
            Condition::NotIn { key, values } => {
                meta.get_raw(key).map_or(true, |v| !values.contains(v))
            }
        }
    }
}

/// Compares two [`Value`]s where both are the same numeric or string variant.
/// Returns `None` when the values are incomparable (different types).
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

/// A composable filter expression over [`Metadata`].
///
/// Filters can be built from primitive [`Condition`]s and combined with
/// [`MetadataFilter::and`], [`MetadataFilter::or`], and [`MetadataFilter::not`].
///
/// # Examples
///
/// ```rust
/// use qubit_metadata::{Metadata, MetadataFilter};
///
/// let mut meta = Metadata::new();
/// meta.set("env", "prod");
/// meta.set("version", 2_i64);
///
/// let f = MetadataFilter::eq("env", "prod")
///     .and(MetadataFilter::gte("version", 1_i64));
///
/// assert!(f.matches(&meta));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetadataFilter {
    /// A leaf condition.
    Condition(Condition),
    /// All child filters must match.
    And(Vec<MetadataFilter>),
    /// At least one child filter must match.
    Or(Vec<MetadataFilter>),
    /// The child filter must not match.
    Not(Box<MetadataFilter>),
}

impl MetadataFilter {
    // ── Leaf constructors ────────────────────────────────────────────────────

    /// Creates an equality filter: `key == value`.
    pub fn eq<T: Serialize>(key: impl Into<String>, value: T) -> Self {
        Self::Condition(Condition::Eq {
            key: key.into(),
            value: serde_json::to_value(value)
                .expect("MetadataFilter::eq: value must be serializable"),
        })
    }

    /// Creates a not-equal filter: `key != value`.
    pub fn ne<T: Serialize>(key: impl Into<String>, value: T) -> Self {
        Self::Condition(Condition::Ne {
            key: key.into(),
            value: serde_json::to_value(value)
                .expect("MetadataFilter::ne: value must be serializable"),
        })
    }

    /// Creates a greater-than filter: `key > value`.
    pub fn gt<T: Serialize>(key: impl Into<String>, value: T) -> Self {
        Self::Condition(Condition::Gt {
            key: key.into(),
            value: serde_json::to_value(value)
                .expect("MetadataFilter::gt: value must be serializable"),
        })
    }

    /// Creates a greater-than-or-equal filter: `key >= value`.
    pub fn gte<T: Serialize>(key: impl Into<String>, value: T) -> Self {
        Self::Condition(Condition::Gte {
            key: key.into(),
            value: serde_json::to_value(value)
                .expect("MetadataFilter::gte: value must be serializable"),
        })
    }

    /// Creates a less-than filter: `key < value`.
    pub fn lt<T: Serialize>(key: impl Into<String>, value: T) -> Self {
        Self::Condition(Condition::Lt {
            key: key.into(),
            value: serde_json::to_value(value)
                .expect("MetadataFilter::lt: value must be serializable"),
        })
    }

    /// Creates a less-than-or-equal filter: `key <= value`.
    pub fn lte<T: Serialize>(key: impl Into<String>, value: T) -> Self {
        Self::Condition(Condition::Lte {
            key: key.into(),
            value: serde_json::to_value(value)
                .expect("MetadataFilter::lte: value must be serializable"),
        })
    }

    /// Creates an existence filter: the key must be present.
    pub fn exists(key: impl Into<String>) -> Self {
        Self::Condition(Condition::Exists { key: key.into() })
    }

    /// Creates a non-existence filter: the key must be absent.
    pub fn not_exists(key: impl Into<String>) -> Self {
        Self::Condition(Condition::NotExists { key: key.into() })
    }

    /// Creates an in-set filter: `key ∈ values`.
    pub fn in_values<T, I>(key: impl Into<String>, values: I) -> Self
    where
        T: Serialize,
        I: IntoIterator<Item = T>,
    {
        let values = values
            .into_iter()
            .map(|v| {
                serde_json::to_value(v)
                    .expect("MetadataFilter::in_values: each value must be serializable")
            })
            .collect();
        Self::Condition(Condition::In {
            key: key.into(),
            values,
        })
    }

    /// Creates a not-in-set filter: `key ∉ values`.
    pub fn not_in_values<T, I>(key: impl Into<String>, values: I) -> Self
    where
        T: Serialize,
        I: IntoIterator<Item = T>,
    {
        let values = values
            .into_iter()
            .map(|v| {
                serde_json::to_value(v)
                    .expect("MetadataFilter::not_in_values: each value must be serializable")
            })
            .collect();
        Self::Condition(Condition::NotIn {
            key: key.into(),
            values,
        })
    }

    // ── Logical combinators ──────────────────────────────────────────────────

    /// Combines `self` and `other` with a logical AND.
    ///
    /// If `self` is already an `And` node the new filter is appended to its
    /// children rather than creating a new nested node.
    #[must_use]
    pub fn and(self, other: MetadataFilter) -> Self {
        match self {
            MetadataFilter::And(mut children) => {
                children.push(other);
                MetadataFilter::And(children)
            }
            _ => MetadataFilter::And(vec![self, other]),
        }
    }

    /// Combines `self` and `other` with a logical OR.
    ///
    /// If `self` is already an `Or` node the new filter is appended to its
    /// children rather than creating a new nested node.
    #[must_use]
    pub fn or(self, other: MetadataFilter) -> Self {
        match self {
            MetadataFilter::Or(mut children) => {
                children.push(other);
                MetadataFilter::Or(children)
            }
            _ => MetadataFilter::Or(vec![self, other]),
        }
    }

    /// Wraps `self` in a logical NOT.
    #[allow(clippy::should_implement_trait)]
    #[must_use]
    pub fn not(self) -> Self {
        !self
    }

    // ── Evaluation ───────────────────────────────────────────────────────────

    /// Returns `true` if `meta` satisfies this filter.
    pub fn matches(&self, meta: &Metadata) -> bool {
        match self {
            MetadataFilter::Condition(cond) => cond.matches(meta),
            MetadataFilter::And(children) => children.iter().all(|f| f.matches(meta)),
            MetadataFilter::Or(children) => children.iter().any(|f| f.matches(meta)),
            MetadataFilter::Not(inner) => !inner.matches(meta),
        }
    }
}

impl std::ops::Not for MetadataFilter {
    type Output = MetadataFilter;

    fn not(self) -> Self::Output {
        MetadataFilter::Not(Box::new(self))
    }
}
