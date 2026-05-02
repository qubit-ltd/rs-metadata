/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! [`FilterExpr`].
use super::condition::Condition;
use crate::metadata::Metadata;
use crate::{FilterMatchOptions, MetadataResult};

/// Internal expression tree used by [`crate::MetadataFilter`].
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FilterExpr {
    /// A leaf condition.
    Condition(Condition),
    /// All child expressions must match.
    And(Vec<FilterExpr>),
    /// At least one child expression must match.
    Or(Vec<FilterExpr>),
    /// Negates the child expression.
    Not(Box<FilterExpr>),
    /// Constant false expression.
    False,
}

impl FilterExpr {
    /// Appends one expression to an AND node, flattening nested AND nodes.
    #[inline]
    fn append_and_child(children: &mut Vec<FilterExpr>, expr: FilterExpr) {
        match expr {
            FilterExpr::And(mut nested) => children.append(&mut nested),
            other => children.push(other),
        }
    }

    /// Appends one expression to an OR node, flattening nested OR nodes.
    #[inline]
    fn append_or_child(children: &mut Vec<FilterExpr>, expr: FilterExpr) {
        match expr {
            FilterExpr::Or(mut nested) => children.append(&mut nested),
            other => children.push(other),
        }
    }

    /// Builds an optimized AND expression from two child expressions.
    #[inline]
    pub(crate) fn and(lhs: FilterExpr, rhs: FilterExpr) -> FilterExpr {
        if matches!(lhs, FilterExpr::False) || matches!(rhs, FilterExpr::False) {
            return FilterExpr::False;
        }
        let mut children = Vec::new();
        Self::append_and_child(&mut children, lhs);
        Self::append_and_child(&mut children, rhs);
        FilterExpr::And(children)
    }

    /// Builds an optimized OR expression from two child expressions.
    #[inline]
    pub(crate) fn or(lhs: FilterExpr, rhs: FilterExpr) -> FilterExpr {
        if matches!(lhs, FilterExpr::False) {
            return rhs;
        }
        if matches!(rhs, FilterExpr::False) {
            return lhs;
        }
        let mut children = Vec::new();
        Self::append_or_child(&mut children, lhs);
        Self::append_or_child(&mut children, rhs);
        FilterExpr::Or(children)
    }

    /// Evaluates this expression tree against one metadata object.
    #[inline]
    pub(crate) fn matches(&self, meta: &Metadata, options: FilterMatchOptions) -> bool {
        match self {
            FilterExpr::Condition(condition) => condition.matches(
                meta,
                options.missing_key_policy,
                options.number_comparison_policy,
            ),
            FilterExpr::And(children) => children.iter().all(|expr| expr.matches(meta, options)),
            FilterExpr::Or(children) => children.iter().any(|expr| expr.matches(meta, options)),
            FilterExpr::Not(inner) => !inner.matches(meta, options),
            FilterExpr::False => false,
        }
    }

    /// Visits all leaf conditions in this expression tree.
    pub(crate) fn visit_conditions<F>(&self, visitor: &mut F) -> MetadataResult<()>
    where
        F: FnMut(&Condition) -> MetadataResult<()>,
    {
        match self {
            FilterExpr::Condition(condition) => visitor(condition),
            FilterExpr::And(children) | FilterExpr::Or(children) => {
                for child in children {
                    child.visit_conditions(visitor)?;
                }
                Ok(())
            }
            FilterExpr::Not(inner) => inner.visit_conditions(visitor),
            FilterExpr::False => Ok(()),
        }
    }
}
