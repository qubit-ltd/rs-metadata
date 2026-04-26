/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! [`ConditionWire`].
use qubit_value::Value;
use serde::{Deserialize, Serialize};

use super::super::condition::Condition;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum ConditionWire {
    Eq { key: String, value: Value },
    Ne { key: String, value: Value },
    Lt { key: String, value: Value },
    Le { key: String, value: Value },
    Gt { key: String, value: Value },
    Ge { key: String, value: Value },
    In { key: String, values: Vec<Value> },
    NotIn { key: String, values: Vec<Value> },
    Exists { key: String },
    NotExists { key: String },
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
