// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! V4 wire representation of [`crate::FilterExpression`].

use qubit_value::Value;
use serde::{Deserialize, Serialize};

use crate::{Condition, FilterExpression, FilterExpressionView, MetadataError, MetadataResult};

/// One v4 expression node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum FilterExpressionWire {
    /// Constant true.
    All,
    /// Constant false.
    None,
    /// Equality condition.
    Eq { key: String, value: Value },
    /// Inequality condition.
    Ne { key: String, value: Value },
    /// Less-than condition.
    Lt { key: String, value: Value },
    /// Less-or-equal condition.
    Le { key: String, value: Value },
    /// Greater-than condition.
    Gt { key: String, value: Value },
    /// Greater-or-equal condition.
    Ge { key: String, value: Value },
    /// Inclusion condition.
    In { key: String, values: Vec<Value> },
    /// Exclusion condition.
    NotIn { key: String, values: Vec<Value> },
    /// Existence condition.
    Exists { key: String },
    /// Non-existence condition.
    NotExists { key: String },
    /// Logical AND with at least two children.
    And { children: Vec<FilterExpressionWire> },
    /// Logical OR with at least two children.
    Or { children: Vec<FilterExpressionWire> },
    /// Logical negation.
    Not {
        expression: Box<FilterExpressionWire>,
    },
}

impl FilterExpressionWire {
    /// Converts a v4 node into an expression.
    ///
    /// # Errors
    ///
    /// Returns an invalid-expression error for AND/OR groups with fewer than
    /// two children, or a hard-limit error for oversized trees.
    pub(crate) fn into_expression(self) -> MetadataResult<FilterExpression> {
        match self {
            Self::All => Ok(FilterExpression::match_all()),
            Self::None => Ok(FilterExpression::match_none()),
            Self::Eq { key, value } => {
                Ok(FilterExpression::condition(Condition::Equal { key, value }))
            }
            Self::Ne { key, value } => Ok(FilterExpression::condition(Condition::NotEqual {
                key,
                value,
            })),
            Self::Lt { key, value } => {
                Ok(FilterExpression::condition(Condition::Less { key, value }))
            }
            Self::Le { key, value } => Ok(FilterExpression::condition(Condition::LessEqual {
                key,
                value,
            })),
            Self::Gt { key, value } => Ok(FilterExpression::condition(Condition::Greater {
                key,
                value,
            })),
            Self::Ge { key, value } => Ok(FilterExpression::condition(Condition::GreaterEqual {
                key,
                value,
            })),
            Self::In { key, values } => {
                Ok(FilterExpression::condition(Condition::In { key, values }))
            }
            Self::NotIn { key, values } => Ok(FilterExpression::condition(Condition::NotIn {
                key,
                values,
            })),
            Self::Exists { key } => Ok(FilterExpression::condition(Condition::Exists { key })),
            Self::NotExists { key } => {
                Ok(FilterExpression::condition(Condition::NotExists { key }))
            }
            Self::Not { expression } => expression.into_expression()?.try_not(),
            Self::And { children } => Self::combine(children, FilterExpression::try_and, "and"),
            Self::Or { children } => Self::combine(children, FilterExpression::try_or, "or"),
        }
    }

    /// Folds a non-trivial Boolean group into an expression.
    fn combine(
        children: Vec<Self>,
        combine: fn(FilterExpression, FilterExpression) -> MetadataResult<FilterExpression>,
        operator: &'static str,
    ) -> MetadataResult<FilterExpression> {
        let mut children = children.into_iter();
        let Some(first) = children.next() else {
            return Err(MetadataError::InvalidFilterExpression {
                message: format!("'{operator}' group requires at least two children"),
            });
        };
        let Some(second) = children.next() else {
            return Err(MetadataError::InvalidFilterExpression {
                message: format!("'{operator}' group requires at least two children"),
            });
        };
        let mut expression = combine(first.into_expression()?, second.into_expression()?)?;
        for child in children {
            expression = combine(expression, child.into_expression()?)?;
        }
        Ok(expression)
    }
}

impl From<&FilterExpression> for FilterExpressionWire {
    /// Converts an expression into its v4 node representation.
    fn from(expression: &FilterExpression) -> Self {
        match expression.view() {
            FilterExpressionView::True => Self::All,
            FilterExpressionView::False => Self::None,
            FilterExpressionView::Condition(Condition::Equal { key, value }) => Self::Eq {
                key: key.clone(),
                value: value.clone(),
            },
            FilterExpressionView::Condition(Condition::NotEqual { key, value }) => Self::Ne {
                key: key.clone(),
                value: value.clone(),
            },
            FilterExpressionView::Condition(Condition::Less { key, value }) => Self::Lt {
                key: key.clone(),
                value: value.clone(),
            },
            FilterExpressionView::Condition(Condition::LessEqual { key, value }) => Self::Le {
                key: key.clone(),
                value: value.clone(),
            },
            FilterExpressionView::Condition(Condition::Greater { key, value }) => Self::Gt {
                key: key.clone(),
                value: value.clone(),
            },
            FilterExpressionView::Condition(Condition::GreaterEqual { key, value }) => Self::Ge {
                key: key.clone(),
                value: value.clone(),
            },
            FilterExpressionView::Condition(Condition::In { key, values }) => Self::In {
                key: key.clone(),
                values: values.clone(),
            },
            FilterExpressionView::Condition(Condition::NotIn { key, values }) => Self::NotIn {
                key: key.clone(),
                values: values.clone(),
            },
            FilterExpressionView::Condition(Condition::Exists { key }) => {
                Self::Exists { key: key.clone() }
            }
            FilterExpressionView::Condition(Condition::NotExists { key }) => {
                Self::NotExists { key: key.clone() }
            }
            FilterExpressionView::And(children) => Self::And {
                children: children.iter().map(Self::from).collect(),
            },
            FilterExpressionView::Or(children) => Self::Or {
                children: children.iter().map(Self::from).collect(),
            },
            FilterExpressionView::Not(expression) => Self::Not {
                expression: Box::new(Self::from(expression)),
            },
        }
    }
}
