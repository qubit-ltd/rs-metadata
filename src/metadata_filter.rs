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
//! let filter = MetadataFilter::equal("status", "active")
//!     .and(MetadataFilter::greater_equal("score", 10_i64));
//!
//! assert!(filter.matches(&meta));
//! ```

use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    Condition,
    Metadata,
};

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
/// let f = MetadataFilter::equal("env", "prod")
///     .and(MetadataFilter::greater_equal("version", 1_i64));
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
    pub fn equal<T: Serialize>(key: impl Into<String>, value: T) -> Self {
        Self::Condition(Condition::Equal {
            key: key.into(),
            value: serde_json::to_value(value)
                .expect("MetadataFilter::equal: value must be serializable"),
        })
    }

    /// Creates a not-equal filter: `key != value`.
    pub fn not_equal<T: Serialize>(key: impl Into<String>, value: T) -> Self {
        Self::Condition(Condition::NotEqual {
            key: key.into(),
            value: serde_json::to_value(value)
                .expect("MetadataFilter::not_equal: value must be serializable"),
        })
    }

    /// Creates a greater-than filter: `key > value`.
    pub fn greater<T: Serialize>(key: impl Into<String>, value: T) -> Self {
        Self::Condition(Condition::Greater {
            key: key.into(),
            value: serde_json::to_value(value)
                .expect("MetadataFilter::greater: value must be serializable"),
        })
    }

    /// Creates a greater-than-or-equal filter: `key >= value`.
    pub fn greater_equal<T: Serialize>(key: impl Into<String>, value: T) -> Self {
        Self::Condition(Condition::GreaterEqual {
            key: key.into(),
            value: serde_json::to_value(value)
                .expect("MetadataFilter::greater_equal: value must be serializable"),
        })
    }

    /// Creates a less-than filter: `key < value`.
    pub fn less<T: Serialize>(key: impl Into<String>, value: T) -> Self {
        Self::Condition(Condition::Less {
            key: key.into(),
            value: serde_json::to_value(value)
                .expect("MetadataFilter::less: value must be serializable"),
        })
    }

    /// Creates a less-than-or-equal filter: `key <= value`.
    pub fn less_equal<T: Serialize>(key: impl Into<String>, value: T) -> Self {
        Self::Condition(Condition::LessEqual {
            key: key.into(),
            value: serde_json::to_value(value)
                .expect("MetadataFilter::less_equal: value must be serializable"),
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
