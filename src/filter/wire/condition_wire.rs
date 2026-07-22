// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! [`ConditionWire`].

use qubit_value::Value;
use serde::{Deserialize, Serialize};

use super::super::condition::Condition;

/// Serializable representation of one [`Condition`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum ConditionWire {
    /// Equality condition.
    Eq {
        /// Metadata key.
        key: String,
        /// Expected value.
        value: Value,
    },
    /// Inequality condition.
    Ne {
        /// Metadata key.
        key: String,
        /// Rejected value.
        value: Value,
    },
    /// Exclusive upper-bound condition.
    Lt {
        /// Metadata key.
        key: String,
        /// Exclusive upper bound.
        value: Value,
    },
    /// Inclusive upper-bound condition.
    Le {
        /// Metadata key.
        key: String,
        /// Inclusive upper bound.
        value: Value,
    },
    /// Exclusive lower-bound condition.
    Gt {
        /// Metadata key.
        key: String,
        /// Exclusive lower bound.
        value: Value,
    },
    /// Inclusive lower-bound condition.
    Ge {
        /// Metadata key.
        key: String,
        /// Inclusive lower bound.
        value: Value,
    },
    /// Set-membership condition.
    In {
        /// Metadata key.
        key: String,
        /// Accepted values.
        values: Vec<Value>,
    },
    /// Set-exclusion condition.
    NotIn {
        /// Metadata key.
        key: String,
        /// Rejected values.
        values: Vec<Value>,
    },
    /// Concrete-value existence condition.
    Exists {
        /// Metadata key.
        key: String,
    },
    /// Concrete-value non-existence condition.
    NotExists {
        /// Metadata key.
        key: String,
    },
}

impl From<&Condition> for ConditionWire {
    fn from(condition: &Condition) -> Self {
        match condition {
            Condition::Equal { key, value } => Self::Eq {
                key: key.clone(),
                value: value.clone(),
            },
            Condition::NotEqual { key, value } => Self::Ne {
                key: key.clone(),
                value: value.clone(),
            },
            Condition::Less { key, value } => Self::Lt {
                key: key.clone(),
                value: value.clone(),
            },
            Condition::LessEqual { key, value } => Self::Le {
                key: key.clone(),
                value: value.clone(),
            },
            Condition::Greater { key, value } => Self::Gt {
                key: key.clone(),
                value: value.clone(),
            },
            Condition::GreaterEqual { key, value } => Self::Ge {
                key: key.clone(),
                value: value.clone(),
            },
            Condition::In { key, values } => Self::In {
                key: key.clone(),
                values: values.clone(),
            },
            Condition::NotIn { key, values } => Self::NotIn {
                key: key.clone(),
                values: values.clone(),
            },
            Condition::Exists { key } => Self::Exists { key: key.clone() },
            Condition::NotExists { key } => Self::NotExists { key: key.clone() },
        }
    }
}

impl ConditionWire {
    /// Converts this wire value into its public condition representation.
    ///
    /// # Returns
    ///
    /// The decoded condition.
    pub(crate) fn into_condition(self) -> Condition {
        match self {
            Self::Eq { key, value } => Condition::Equal { key, value },
            Self::Ne { key, value } => Condition::NotEqual { key, value },
            Self::Lt { key, value } => Condition::Less { key, value },
            Self::Le { key, value } => Condition::LessEqual { key, value },
            Self::Gt { key, value } => Condition::Greater { key, value },
            Self::Ge { key, value } => Condition::GreaterEqual { key, value },
            Self::In { key, values } => Condition::In { key, values },
            Self::NotIn { key, values } => Condition::NotIn { key, values },
            Self::Exists { key } => Condition::Exists { key },
            Self::NotExists { key } => Condition::NotExists { key },
        }
    }
}
