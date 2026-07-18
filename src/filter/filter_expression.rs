// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Immutable filter expressions and their read-only views.

use crate::filter::internal::{
    FilterExpressionNode,
    MatchOutcome,
};
use crate::{
    Condition,
    FilterExpressionView,
    FilterLimits,
    FilterMatchOptions,
    Metadata,
    MetadataError,
    MetadataResult,
};

/// An immutable Boolean expression in a [`crate::MetadataFilter`].
///
/// Expressions are constructed by [`crate::MetadataFilterBuilder`] and can be
/// inspected without allocation through [`FilterExpression::view`]. Their
/// private representation prevents callers from constructing structurally
/// invalid expression trees. The structure is Boolean, while evaluation uses
/// a private three-valued outcome so missing data stays unknown through NOT,
/// AND, and OR.
#[derive(Debug, Clone, PartialEq)]
#[must_use]
pub struct FilterExpression {
    /// Private expression node.
    node: FilterExpressionNode,
}

impl FilterExpression {
    /// Creates a condition expression.
    ///
    /// # Parameters
    ///
    /// * `condition` - Leaf condition to store.
    ///
    /// # Returns
    ///
    /// A new condition expression.
    #[inline]
    pub(crate) fn condition(condition: Condition) -> Self {
        Self {
            node: FilterExpressionNode::Condition(condition),
        }
    }

    /// Creates a constant true expression.
    ///
    /// # Returns
    ///
    /// A new constant true expression.
    #[inline]
    pub(crate) const fn true_expression() -> Self {
        Self {
            node: FilterExpressionNode::True,
        }
    }

    /// Creates a constant false expression.
    ///
    /// # Returns
    ///
    /// A new constant false expression.
    #[inline]
    pub(crate) const fn false_expression() -> Self {
        Self {
            node: FilterExpressionNode::False,
        }
    }

    /// Combines two expressions with logical AND.
    ///
    /// # Parameters
    ///
    /// * `left` - Left child expression.
    /// * `right` - Right child expression.
    ///
    /// # Returns
    ///
    /// A simplified AND expression.
    pub(crate) fn and(left: Self, right: Self) -> Self {
        if left.is_false() || right.is_false() {
            return Self::false_expression();
        }
        if left.is_true() {
            return right;
        }
        if right.is_true() {
            return left;
        }
        Self::combine_and(left, right)
    }

    /// Combines two expressions with logical OR.
    ///
    /// # Parameters
    ///
    /// * `left` - Left child expression.
    /// * `right` - Right child expression.
    ///
    /// # Returns
    ///
    /// A simplified OR expression.
    pub(crate) fn or(left: Self, right: Self) -> Self {
        if left.is_true() || right.is_true() {
            return Self::true_expression();
        }
        if left.is_false() {
            return right;
        }
        if right.is_false() {
            return left;
        }
        Self::combine_or(left, right)
    }

    /// Creates an AND expression from a non-empty child list.
    ///
    /// # Parameters
    ///
    /// * `children` - Child expressions to combine.
    ///
    /// # Returns
    ///
    /// An AND expression containing the supplied children.
    #[inline]
    pub(crate) fn and_children(children: Vec<Self>) -> Self {
        Self {
            node: FilterExpressionNode::And(children),
        }
    }

    /// Creates an OR expression from a non-empty child list.
    ///
    /// # Parameters
    ///
    /// * `children` - Child expressions to combine.
    ///
    /// # Returns
    ///
    /// An OR expression containing the supplied children.
    #[inline]
    pub(crate) fn or_children(children: Vec<Self>) -> Self {
        Self {
            node: FilterExpressionNode::Or(children),
        }
    }

    /// Creates a NOT expression without simplifying its child.
    ///
    /// # Parameters
    ///
    /// * `expression` - Child expression to negate.
    ///
    /// # Returns
    ///
    /// A NOT expression containing the supplied child.
    #[inline]
    pub(crate) fn not_expression(expression: Self) -> Self {
        Self {
            node: FilterExpressionNode::Not(Box::new(expression)),
        }
    }

    /// Returns a borrowed view of this expression node.
    ///
    /// # Returns
    ///
    /// A zero-copy view preserving the node's Boolean structure.
    #[inline(always)]
    pub fn view(&self) -> FilterExpressionView<'_> {
        match &self.node {
            FilterExpressionNode::Condition(condition) => {
                FilterExpressionView::Condition(condition)
            }
            FilterExpressionNode::And(children) => {
                FilterExpressionView::And(children)
            }
            FilterExpressionNode::Or(children) => {
                FilterExpressionView::Or(children)
            }
            FilterExpressionNode::Not(inner) => {
                FilterExpressionView::Not(inner)
            }
            FilterExpressionNode::True => FilterExpressionView::True,
            FilterExpressionNode::False => FilterExpressionView::False,
        }
    }

    /// Returns the three-valued logical negation of this expression.
    ///
    /// # Returns
    ///
    /// A simplified negated expression.
    pub(crate) fn negated(self) -> Self {
        match self.node {
            FilterExpressionNode::True => Self::false_expression(),
            FilterExpressionNode::False => Self::true_expression(),
            FilterExpressionNode::Not(inner) => *inner,
            node => Self::not_expression(Self { node }),
        }
    }

    /// Reports whether this is a constant true expression.
    ///
    /// # Returns
    ///
    /// `true` only for the constant true node.
    #[inline(always)]
    pub(crate) const fn is_true(&self) -> bool {
        matches!(&self.node, FilterExpressionNode::True)
    }

    /// Reports whether this is a constant false expression.
    ///
    /// # Returns
    ///
    /// `true` only for the constant false node.
    #[inline(always)]
    pub(crate) const fn is_false(&self) -> bool {
        matches!(&self.node, FilterExpressionNode::False)
    }

    /// Evaluates this expression against one metadata object.
    ///
    /// # Parameters
    ///
    /// * `metadata` - Metadata object being matched.
    /// * `options` - Match options to apply.
    ///
    /// # Returns
    ///
    /// The three-valued expression outcome.
    pub(crate) fn evaluate(
        &self,
        metadata: &Metadata,
        options: FilterMatchOptions,
    ) -> MatchOutcome {
        match &self.node {
            FilterExpressionNode::Condition(condition) => condition
                .evaluate(metadata, options.numeric_comparison_policy()),
            FilterExpressionNode::And(children) => MatchOutcome::and(
                children
                    .iter()
                    .map(|child| child.evaluate(metadata, options)),
            ),
            FilterExpressionNode::Or(children) => MatchOutcome::or(
                children
                    .iter()
                    .map(|child| child.evaluate(metadata, options)),
            ),
            FilterExpressionNode::Not(inner) => {
                inner.evaluate(metadata, options).not()
            }
            FilterExpressionNode::True => MatchOutcome::True,
            FilterExpressionNode::False => MatchOutcome::False,
        }
    }

    /// Visits every leaf condition in this expression.
    ///
    /// # Parameters
    ///
    /// * `visitor` - Callback invoked for each condition.
    ///
    /// # Returns
    ///
    /// `Ok(())` after all conditions have been visited.
    ///
    /// # Errors
    ///
    /// Returns the first error produced by `visitor`.
    pub(crate) fn visit_conditions<F>(
        &self,
        visitor: &mut F,
    ) -> MetadataResult<()>
    where
        F: FnMut(&Condition) -> MetadataResult<()>,
    {
        match &self.node {
            FilterExpressionNode::Condition(condition) => visitor(condition),
            FilterExpressionNode::And(children)
            | FilterExpressionNode::Or(children) => {
                for child in children {
                    child.visit_conditions(visitor)?;
                }
                Ok(())
            }
            FilterExpressionNode::Not(inner) => inner.visit_conditions(visitor),
            FilterExpressionNode::True | FilterExpressionNode::False => Ok(()),
        }
    }

    /// Validates this expression against resource limits.
    ///
    /// # Parameters
    ///
    /// * `limits` - Bounds to enforce.
    ///
    /// # Returns
    ///
    /// `Ok(())` when every node and condition fits within `limits`.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::FilterLimitExceeded`] when depth, node count,
    /// key length, or membership values exceed a configured bound.
    pub(crate) fn validate_limits(
        &self,
        limits: FilterLimits,
    ) -> MetadataResult<()> {
        let mut node_count = 0;
        self.validate_limits_at(limits, 1, &mut node_count)
    }

    /// Recursively validates one expression node at `depth`.
    fn validate_limits_at(
        &self,
        limits: FilterLimits,
        depth: usize,
        node_count: &mut usize,
    ) -> MetadataResult<()> {
        if depth > limits.max_depth() {
            return Err(MetadataError::FilterLimitExceeded {
                limit: "depth",
                maximum: limits.max_depth(),
            });
        }
        *node_count += 1;
        if *node_count > limits.max_nodes() {
            return Err(MetadataError::FilterLimitExceeded {
                limit: "node count",
                maximum: limits.max_nodes(),
            });
        }
        match &self.node {
            FilterExpressionNode::Condition(condition) => {
                condition.validate_limits(limits)
            }
            FilterExpressionNode::And(children)
            | FilterExpressionNode::Or(children) => {
                for child in children {
                    child.validate_limits_at(limits, depth + 1, node_count)?;
                }
                Ok(())
            }
            FilterExpressionNode::Not(inner) => {
                inner.validate_limits_at(limits, depth + 1, node_count)
            }
            FilterExpressionNode::True | FilterExpressionNode::False => Ok(()),
        }
    }

    /// Combines two non-constant expressions with logical AND while reusing a
    /// left AND group.
    fn combine_and(left: Self, right: Self) -> Self {
        let mut children = match left.node {
            FilterExpressionNode::And(children) => children,
            node => vec![Self { node }],
        };
        match right.node {
            FilterExpressionNode::And(mut nested) => {
                children.append(&mut nested)
            }
            node => children.push(Self { node }),
        }
        Self {
            node: FilterExpressionNode::And(children),
        }
    }

    /// Combines two non-constant expressions with logical OR while reusing a
    /// left OR group.
    fn combine_or(left: Self, right: Self) -> Self {
        let mut children = match left.node {
            FilterExpressionNode::Or(children) => children,
            node => vec![Self { node }],
        };
        match right.node {
            FilterExpressionNode::Or(mut nested) => {
                children.append(&mut nested)
            }
            node => children.push(Self { node }),
        }
        Self {
            node: FilterExpressionNode::Or(children),
        }
    }
}

impl Default for FilterExpression {
    #[inline(always)]
    fn default() -> Self {
        Self::true_expression()
    }
}
