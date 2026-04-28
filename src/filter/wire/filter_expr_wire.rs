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
use super::condition_wire::ConditionWire;

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
    pub(crate) fn into_expr(self) -> Result<FilterExpr, String> {
        match self {
            Self::Condition { condition } => Ok(FilterExpr::Condition(condition.into_condition())),
            Self::And { children } => {
                if children.is_empty() {
                    return Err("empty 'and' filter group is not allowed".to_string());
                }
                children
                    .into_iter()
                    .map(Self::into_expr)
                    .collect::<Result<Vec<_>, _>>()
                    .map(FilterExpr::And)
            }
            Self::Or { children } => {
                if children.is_empty() {
                    return Err("empty 'or' filter group is not allowed".to_string());
                }
                children
                    .into_iter()
                    .map(Self::into_expr)
                    .collect::<Result<Vec<_>, _>>()
                    .map(FilterExpr::Or)
            }
            Self::Not { expr } => expr.into_expr().map(|expr| FilterExpr::Not(Box::new(expr))),
            Self::False => Ok(FilterExpr::False),
        }
    }
}
