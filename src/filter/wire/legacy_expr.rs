/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! [`LegacyFilterExpr`].
use serde::Deserialize;

use super::super::condition::Condition;
use super::super::filter_expr::FilterExpr;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) enum LegacyFilterExpr {
    Condition(Condition),
    And(Vec<LegacyFilterExpr>),
    Or(Vec<LegacyFilterExpr>),
    Not(Box<LegacyFilterExpr>),
    False,
}

impl LegacyFilterExpr {
    pub(crate) fn into_expr(self) -> FilterExpr {
        match self {
            Self::Condition(condition) => FilterExpr::Condition(condition),
            Self::And(children) => {
                FilterExpr::And(children.into_iter().map(Self::into_expr).collect())
            }
            Self::Or(children) => {
                FilterExpr::Or(children.into_iter().map(Self::into_expr).collect())
            }
            Self::Not(inner) => FilterExpr::Not(Box::new(inner.into_expr())),
            Self::False => FilterExpr::False,
        }
    }
}
