// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Immutable filter expressions and their read-only views.

use std::fmt;

use crate::Condition;
use crate::FilterExpressionBuilder;
use crate::FilterExpressionView;
use crate::FilterLimitKind;
use crate::FilterLimits;
use crate::FilterMatchOptions;
use crate::Metadata;
use crate::MetadataError;
use crate::MetadataResult;
use crate::filter::internal::FilterExpressionNode;
use crate::filter::internal::MatchOutcome;

/// An immutable Boolean expression in a [`crate::MetadataFilter`].
///
/// Expressions are constructed by [`FilterExpressionBuilder`] and can be
/// inspected without allocation through [`FilterExpression::view`]. Their
/// private representation prevents callers from constructing structurally
/// invalid expression trees. The structure is Boolean, while evaluation uses
/// a private three-valued outcome so missing data stays unknown through NOT,
/// AND, and OR.
///
/// # Examples
///
/// ```
/// use qubit_metadata::FilterExpression;
///
/// # fn main() -> qubit_metadata::MetadataResult<()> {
/// let expression = FilterExpression::builder()
///     .eq("tenant", "acme")
///     .build()?;
/// assert!(matches!(
///     expression.view(),
///     qubit_metadata::FilterExpressionView::Condition(_)
/// ));
/// # Ok(())
/// # }
/// ```
#[derive(Clone, PartialEq)]
#[must_use]
pub struct FilterExpression {
    /// Private expression node.
    node: FilterExpressionNode,
    /// Total number of nodes after logical-group flattening.
    node_count: usize,
    /// Maximum node depth after logical-group flattening.
    max_depth: usize,
}

impl fmt::Debug for FilterExpression {
    /// Formats the expression without exposing its cached implementation
    /// metrics.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FilterExpression")
            .field("node", &self.node)
            .finish()
    }
}

impl FilterExpression {
    /// Creates a builder for a non-empty filter expression.
    #[inline(always)]
    #[must_use]
    pub const fn builder() -> FilterExpressionBuilder {
        FilterExpressionBuilder::new()
    }

    /// Creates an expression that matches every metadata object.
    #[inline(always)]
    #[must_use = "the constructed all-matching expression should be used"]
    pub const fn match_all() -> Self {
        Self::true_expression()
    }

    /// Creates an expression that matches no metadata object.
    #[inline(always)]
    #[must_use = "the constructed no-match expression should be used"]
    pub const fn match_none() -> Self {
        Self::false_expression()
    }

    /// Combines this expression with `other` using logical AND.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::FilterLimitExceeded`] when the resulting
    /// expression exceeds library hard limits.
    #[inline]
    pub fn try_and(self, other: Self) -> MetadataResult<Self> {
        let expression = Self::and_unchecked(self, other);
        expression.validate_limits(FilterLimits::MAX)?;
        Ok(expression)
    }

    /// Combines this expression with `other` using logical OR.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::FilterLimitExceeded`] when the resulting
    /// expression exceeds library hard limits.
    #[inline]
    pub fn try_or(self, other: Self) -> MetadataResult<Self> {
        let expression = Self::or_unchecked(self, other);
        expression.validate_limits(FilterLimits::MAX)?;
        Ok(expression)
    }

    /// Negates this expression.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::FilterLimitExceeded`] when the resulting
    /// expression exceeds library hard limits.
    #[inline]
    pub fn try_not(self) -> MetadataResult<Self> {
        let expression = self.negated_unchecked();
        expression.validate_limits(FilterLimits::MAX)?;
        Ok(expression)
    }

    /// Returns a borrowed view of this expression node.
    ///
    /// # Returns
    ///
    /// A zero-copy view preserving the node's Boolean structure.
    #[inline(always)]
    #[must_use = "the expression view should be inspected"]
    pub fn view(&self) -> FilterExpressionView<'_> {
        match &self.node {
            FilterExpressionNode::Condition(condition) => FilterExpressionView::Condition(condition),
            FilterExpressionNode::And(children) => FilterExpressionView::And(children),
            FilterExpressionNode::Or(children) => FilterExpressionView::Or(children),
            FilterExpressionNode::Not(inner) => FilterExpressionView::Not(inner),
            FilterExpressionNode::True => FilterExpressionView::True,
            FilterExpressionNode::False => FilterExpressionView::False,
        }
    }

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
    pub(crate) fn condition(condition: Condition) -> MetadataResult<Self> {
        condition.validate_operands()?;
        Ok(Self {
            node: FilterExpressionNode::Condition(condition),
            node_count: 1,
            max_depth: 1,
        })
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
            node_count: 1,
            max_depth: 1,
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
            node_count: 1,
            max_depth: 1,
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
    pub(crate) fn and_unchecked(left: Self, right: Self) -> Self {
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
    pub(crate) fn or_unchecked(left: Self, right: Self) -> Self {
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
        let node_count = expression.node_count + 1;
        let max_depth = expression.max_depth + 1;
        Self {
            node: FilterExpressionNode::Not(Box::new(expression)),
            node_count,
            max_depth,
        }
    }

    /// Returns the three-valued logical negation of this expression.
    ///
    /// # Returns
    ///
    /// A simplified negated expression.
    pub(crate) fn negated_unchecked(self) -> Self {
        match self {
            Self {
                node: FilterExpressionNode::True,
                ..
            } => Self::false_expression(),
            Self {
                node: FilterExpressionNode::False,
                ..
            } => Self::true_expression(),
            Self {
                node: FilterExpressionNode::Not(inner),
                ..
            } => *inner,
            expression => Self::not_expression(expression),
        }
    }

    /// Reports whether this is a constant true expression.
    ///
    /// # Returns
    ///
    /// `true` only for the constant true node.
    #[inline]
    pub(crate) const fn is_true(&self) -> bool {
        matches!(&self.node, FilterExpressionNode::True)
    }

    /// Reports whether this is a constant false expression.
    ///
    /// # Returns
    ///
    /// `true` only for the constant false node.
    #[inline]
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
    pub(crate) fn evaluate(&self, metadata: &Metadata, options: FilterMatchOptions) -> MatchOutcome {
        match &self.node {
            FilterExpressionNode::Condition(condition) => {
                condition.evaluate(metadata, options.numeric_comparison_policy())
            }
            FilterExpressionNode::And(children) => {
                MatchOutcome::and(children.iter().map(|child| child.evaluate(metadata, options)))
            }
            FilterExpressionNode::Or(children) => {
                MatchOutcome::or(children.iter().map(|child| child.evaluate(metadata, options)))
            }
            FilterExpressionNode::Not(inner) => inner.evaluate(metadata, options).not(),
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
    #[cfg(feature = "schema")]
    pub(crate) fn visit_conditions<F>(&self, visitor: &mut F) -> MetadataResult<()>
    where
        F: FnMut(&Condition) -> MetadataResult<()>,
    {
        match &self.node {
            FilterExpressionNode::Condition(condition) => visitor(condition),
            FilterExpressionNode::And(children) | FilterExpressionNode::Or(children) => {
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
    pub(crate) fn validate_limits(&self, limits: FilterLimits) -> MetadataResult<()> {
        let mut node_count = 0;
        self.validate_limits_at(limits, 1, &mut node_count)
    }

    /// Validates cached structural metrics against resource limits in O(1).
    ///
    /// # Parameters
    ///
    /// * `limits` - Structural bounds to enforce.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::FilterLimitExceeded`] when the cached maximum
    /// depth or total node count exceeds `limits`. The reported value is the
    /// first value beyond the configured maximum, matching recursive
    /// validation semantics.
    pub(crate) fn validate_structure_limits(&self, limits: FilterLimits) -> MetadataResult<()> {
        if self.max_depth > limits.max_depth() {
            return Err(MetadataError::FilterLimitExceeded {
                kind: FilterLimitKind::Depth,
                value: limits.max_depth() + 1,
                maximum: limits.max_depth(),
            });
        }
        if self.node_count > limits.max_nodes() {
            return Err(MetadataError::FilterLimitExceeded {
                kind: FilterLimitKind::Nodes,
                value: limits.max_nodes() + 1,
                maximum: limits.max_nodes(),
            });
        }
        Ok(())
    }

    /// Recursively validates one expression node in depth-first order.
    ///
    /// # Parameters
    ///
    /// * `limits` - Bounds to enforce.
    /// * `depth` - Root-inclusive depth of this node.
    /// * `node_count` - Number of nodes visited before this node.
    ///
    /// # Errors
    ///
    /// Returns the first depth, node-count, or condition-limit error reached
    /// by the depth-first traversal.
    fn validate_limits_at(&self, limits: FilterLimits, depth: usize, node_count: &mut usize) -> MetadataResult<()> {
        if depth > limits.max_depth() {
            return Err(MetadataError::FilterLimitExceeded {
                kind: FilterLimitKind::Depth,
                value: depth,
                maximum: limits.max_depth(),
            });
        }
        *node_count += 1;
        if *node_count > limits.max_nodes() {
            return Err(MetadataError::FilterLimitExceeded {
                kind: FilterLimitKind::Nodes,
                value: *node_count,
                maximum: limits.max_nodes(),
            });
        }
        match &self.node {
            FilterExpressionNode::Condition(condition) => condition.validate_limits(limits),
            FilterExpressionNode::And(children) | FilterExpressionNode::Or(children) => {
                for child in children {
                    child.validate_limits_at(limits, depth + 1, node_count)?;
                }
                Ok(())
            }
            FilterExpressionNode::Not(inner) => inner.validate_limits_at(limits, depth + 1, node_count),
            FilterExpressionNode::True | FilterExpressionNode::False => Ok(()),
        }
    }

    /// Asserts that cached structural metrics equal recursively computed
    /// metrics.
    #[cfg(test)]
    fn assert_cached_metrics_consistent(&self) {
        match &self.node {
            FilterExpressionNode::And(children) | FilterExpressionNode::Or(children) => {
                for child in children {
                    child.assert_cached_metrics_consistent();
                }
            }
            FilterExpressionNode::Not(inner) => inner.assert_cached_metrics_consistent(),
            FilterExpressionNode::Condition(_) | FilterExpressionNode::True | FilterExpressionNode::False => {}
        }
        let (node_count, max_depth) = self.recursive_metrics();
        assert_eq!(
            self.node_count, node_count,
            "cached node count differs from the expression tree"
        );
        assert_eq!(
            self.max_depth, max_depth,
            "cached maximum depth differs from the expression tree"
        );
    }

    /// Recursively computes the node count and maximum depth for tests.
    #[cfg(test)]
    fn recursive_metrics(&self) -> (usize, usize) {
        match &self.node {
            FilterExpressionNode::Condition(_) | FilterExpressionNode::True | FilterExpressionNode::False => (1, 1),
            FilterExpressionNode::And(children) | FilterExpressionNode::Or(children) => {
                let mut node_count = 1;
                let mut max_child_depth = 0;
                for child in children {
                    let (child_node_count, child_max_depth) = child.recursive_metrics();
                    node_count += child_node_count;
                    max_child_depth = max_child_depth.max(child_max_depth);
                }
                (node_count, max_child_depth + 1)
            }
            FilterExpressionNode::Not(inner) => {
                let (node_count, max_depth) = inner.recursive_metrics();
                (node_count + 1, max_depth + 1)
            }
        }
    }

    /// Combines two non-constant expressions with logical AND while reusing a
    /// left AND group.
    fn combine_and(left: Self, right: Self) -> Self {
        let left_same_kind = matches!(&left.node, FilterExpressionNode::And(_));
        let right_same_kind = matches!(&right.node, FilterExpressionNode::And(_));
        let (node_count, max_depth) = Self::combined_metrics(&left, &right, left_same_kind, right_same_kind);
        let mut children = match left {
            Self {
                node: FilterExpressionNode::And(children),
                ..
            } => children,
            expression => vec![expression],
        };
        match right {
            Self {
                node: FilterExpressionNode::And(mut nested),
                ..
            } => children.append(&mut nested),
            expression => children.push(expression),
        }
        Self {
            node: FilterExpressionNode::And(children),
            node_count,
            max_depth,
        }
    }

    /// Combines two non-constant expressions with logical OR while reusing a
    /// left OR group.
    fn combine_or(left: Self, right: Self) -> Self {
        let left_same_kind = matches!(&left.node, FilterExpressionNode::Or(_));
        let right_same_kind = matches!(&right.node, FilterExpressionNode::Or(_));
        let (node_count, max_depth) = Self::combined_metrics(&left, &right, left_same_kind, right_same_kind);
        let mut children = match left {
            Self {
                node: FilterExpressionNode::Or(children),
                ..
            } => children,
            expression => vec![expression],
        };
        match right {
            Self {
                node: FilterExpressionNode::Or(mut nested),
                ..
            } => children.append(&mut nested),
            expression => children.push(expression),
        }
        Self {
            node: FilterExpressionNode::Or(children),
            node_count,
            max_depth,
        }
    }

    /// Calculates metrics for two expressions combined under one logical
    /// operator, accounting for same-kind root flattening.
    ///
    /// # Parameters
    ///
    /// * `left` - Left expression before flattening.
    /// * `right` - Right expression before flattening.
    /// * `left_same_kind` - Whether the left root is flattened into the result.
    /// * `right_same_kind` - Whether the right root is flattened into the
    ///   result.
    ///
    /// # Returns
    ///
    /// The result's total node count and maximum depth.
    fn combined_metrics(left: &Self, right: &Self, left_same_kind: bool, right_same_kind: bool) -> (usize, usize) {
        let node_count = match (left_same_kind, right_same_kind) {
            (true, true) => left.node_count + right.node_count - 1,
            (true, false) | (false, true) => left.node_count + right.node_count,
            (false, false) => left.node_count + right.node_count + 1,
        };
        let max_depth = match (left_same_kind, right_same_kind) {
            (true, true) => left.max_depth.max(right.max_depth),
            (true, false) => left.max_depth.max(right.max_depth + 1),
            (false, true) => (left.max_depth + 1).max(right.max_depth),
            (false, false) => left.max_depth.max(right.max_depth) + 1,
        };
        (node_count, max_depth)
    }
}

#[cfg(test)]
mod tests {
    use super::FilterExpression;

    /// Verifies that cached metrics agree with a recursive traversal for
    /// representative simplified and nested expression shapes.
    #[test]
    fn test_cached_metrics_match_recursive_metrics() {
        let leaf = FilterExpression::builder()
            .exists("leaf")
            .build()
            .expect("leaf expression should build");
        leaf.assert_cached_metrics_consistent();

        let left = FilterExpression::builder()
            .exists("left_1")
            .exists("left_2")
            .build()
            .expect("left expression should build");
        let right = FilterExpression::builder()
            .exists("right_1")
            .exists("right_2")
            .build()
            .expect("right expression should build");
        let flattened = left.try_and(right).expect("flattened AND should build");
        flattened.assert_cached_metrics_consistent();

        let nested = flattened
            .try_or(
                FilterExpression::builder()
                    .exists("alternative")
                    .build()
                    .expect("alternative expression should build"),
            )
            .expect("nested OR should build")
            .try_not()
            .expect("negated expression should build");
        nested.assert_cached_metrics_consistent();

        FilterExpression::match_all()
            .try_and(nested.clone())
            .expect("true AND expression should simplify")
            .assert_cached_metrics_consistent();
        FilterExpression::match_none()
            .try_or(nested)
            .expect("false OR expression should simplify")
            .assert_cached_metrics_consistent();
        FilterExpression::match_all()
            .try_not()
            .expect("constant negation should simplify")
            .assert_cached_metrics_consistent();
    }
}
