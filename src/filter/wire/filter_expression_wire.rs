// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Serializable representation of [`crate::FilterExpression`].

use serde::{
    Deserialize,
    Serialize,
};

use super::condition_wire::ConditionWire;
use crate::{
    FilterExpression,
    FilterExpressionView,
};

/// Versioned wire node for one filter expression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum FilterExpressionWire {
    /// A leaf condition.
    Condition {
        /// Serializable condition.
        condition: ConditionWire,
    },
    /// A non-empty AND group.
    And {
        /// Child expressions.
        children: Vec<FilterExpressionWire>,
    },
    /// A non-empty OR group.
    Or {
        /// Child expressions.
        children: Vec<FilterExpressionWire>,
    },
    /// A negated expression.
    Not {
        /// Child expression.
        expression: Box<FilterExpressionWire>,
    },
    /// Constant true expression.
    True,
    /// Constant false expression.
    False,
}

impl FilterExpressionWire {
    /// Converts this wire node into an immutable filter expression.
    ///
    /// # Returns
    ///
    /// The decoded filter expression.
    ///
    /// # Errors
    ///
    /// Returns an error when an AND or OR group is empty.
    pub(crate) fn into_expression(self) -> Result<FilterExpression, String> {
        match self {
            Self::Condition { condition } => {
                Ok(FilterExpression::condition(condition.into_condition()))
            }
            Self::And { children } => {
                if children.is_empty() {
                    return Err(
                        "empty 'and' filter group is not allowed".to_string()
                    );
                }
                children
                    .into_iter()
                    .map(Self::into_expression)
                    .collect::<Result<Vec<_>, _>>()
                    .map(FilterExpression::and_children)
            }
            Self::Or { children } => {
                if children.is_empty() {
                    return Err(
                        "empty 'or' filter group is not allowed".to_string()
                    );
                }
                children
                    .into_iter()
                    .map(Self::into_expression)
                    .collect::<Result<Vec<_>, _>>()
                    .map(FilterExpression::or_children)
            }
            Self::Not { expression } => expression
                .into_expression()
                .map(FilterExpression::not_expression),
            Self::True => Ok(FilterExpression::true_expression()),
            Self::False => Ok(FilterExpression::false_expression()),
        }
    }
}

impl From<&FilterExpression> for FilterExpressionWire {
    fn from(expression: &FilterExpression) -> Self {
        match expression.view() {
            FilterExpressionView::Condition(condition) => Self::Condition {
                condition: ConditionWire::from(condition),
            },
            FilterExpressionView::And(children) => Self::And {
                children: children.iter().map(Self::from).collect(),
            },
            FilterExpressionView::Or(children) => Self::Or {
                children: children.iter().map(Self::from).collect(),
            },
            FilterExpressionView::Not(expression) => Self::Not {
                expression: Box::new(Self::from(expression)),
            },
            FilterExpressionView::True => Self::True,
            FilterExpressionView::False => Self::False,
        }
    }
}
