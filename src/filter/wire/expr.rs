/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! [`FilterExprWire`].
use serde::{Deserialize, Serialize};

use super::super::filter_expr::FilterExpr;
use super::condition::ConditionWire;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum FilterExprWire {
    Condition { condition: ConditionWire },
    And { children: Vec<FilterExprWire> },
    Or { children: Vec<FilterExprWire> },
    Not { expr: Box<FilterExprWire> },
    False,
}

impl From<&FilterExpr> for FilterExprWire {
    fn from(expr: &FilterExpr) -> Self {
        match expr {
            FilterExpr::Condition(condition) => Self::Condition {
                condition: ConditionWire::from(condition),
            },
            FilterExpr::And(children) => Self::And {
                children: children.iter().map(Self::from).collect(),
            },
            FilterExpr::Or(children) => Self::Or {
                children: children.iter().map(Self::from).collect(),
            },
            FilterExpr::Not(inner) => Self::Not {
                expr: Box::new(Self::from(inner.as_ref())),
            },
            FilterExpr::False => Self::False,
        }
    }
}

impl FilterExprWire {
    pub(crate) fn into_expr(self) -> FilterExpr {
        match self {
            Self::Condition { condition } => FilterExpr::Condition(condition.into_condition()),
            Self::And { children } => {
                FilterExpr::And(children.into_iter().map(Self::into_expr).collect())
            }
            Self::Or { children } => {
                FilterExpr::Or(children.into_iter().map(Self::into_expr).collect())
            }
            Self::Not { expr } => FilterExpr::Not(Box::new(expr.into_expr())),
            Self::False => FilterExpr::False,
        }
    }
}
