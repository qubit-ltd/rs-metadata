/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Provides [`MetadataFilter`].

use serde::{Deserialize, Serialize};

use crate::{
    Condition, FilterMatchOptions, Metadata, MetadataFilterBuilder, MetadataResult,
    MissingKeyPolicy, NumberComparisonPolicy,
};

/// Internal expression tree used by [`MetadataFilter`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    fn matches(&self, meta: &Metadata, options: FilterMatchOptions) -> bool {
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
    fn visit_conditions<F>(&self, visitor: &mut F) -> MetadataResult<()>
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

/// An immutable, composable filter expression over [`Metadata`].
///
/// Construct filters with [`MetadataFilter::builder`]. An empty builder builds a
/// match-all filter, which makes the default behavior explicit while keeping the
/// built filter immutable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MetadataFilter {
    /// Root expression tree. `None` means match all.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    expr: Option<FilterExpr>,
    /// Match policies used by [`MetadataFilter::matches`].
    #[serde(default)]
    options: FilterMatchOptions,
}

impl MetadataFilter {
    /// Creates a filter from expression and options.
    #[inline]
    pub(crate) fn new(expr: Option<FilterExpr>, options: FilterMatchOptions) -> Self {
        Self { expr, options }
    }

    /// Creates a builder for a metadata filter.
    #[inline]
    #[must_use]
    pub fn builder() -> MetadataFilterBuilder {
        MetadataFilterBuilder::default()
    }

    /// Creates a filter that matches every metadata object.
    #[inline]
    #[must_use]
    pub fn all() -> Self {
        Self::default()
    }

    /// Creates a filter that matches no metadata object.
    #[inline]
    #[must_use]
    pub fn none() -> Self {
        Self {
            expr: Some(FilterExpr::False),
            options: FilterMatchOptions::default(),
        }
    }

    /// Returns the current match options.
    #[inline]
    #[must_use]
    pub fn options(&self) -> FilterMatchOptions {
        self.options
    }

    /// Replaces the current match options and returns a new filter.
    #[inline]
    #[must_use]
    pub fn with_options(mut self, options: FilterMatchOptions) -> Self {
        self.options = options;
        self
    }

    /// Returns a new filter with the supplied missing-key policy.
    #[inline]
    #[must_use]
    pub fn with_missing_key_policy(mut self, missing_key_policy: MissingKeyPolicy) -> Self {
        self.options.missing_key_policy = missing_key_policy;
        self
    }

    /// Returns a new filter with the supplied number-comparison policy.
    #[inline]
    #[must_use]
    pub fn with_number_comparison_policy(
        mut self,
        number_comparison_policy: NumberComparisonPolicy,
    ) -> Self {
        self.options.number_comparison_policy = number_comparison_policy;
        self
    }

    /// Returns a new filter that negates this filter.
    #[allow(clippy::should_implement_trait)]
    #[inline]
    #[must_use]
    pub fn not(mut self) -> Self {
        self.expr = MetadataFilterBuilder::negate_expr(self.expr);
        self
    }

    /// Returns `true` if `meta` satisfies this filter.
    #[inline]
    #[must_use]
    pub fn matches(&self, meta: &Metadata) -> bool {
        self.matches_with_options(meta, self.options)
    }

    /// Returns `true` if `meta` satisfies this filter with explicit options.
    #[inline]
    #[must_use]
    pub fn matches_with_options(&self, meta: &Metadata, options: FilterMatchOptions) -> bool {
        self.expr
            .as_ref()
            .is_none_or(|expr| expr.matches(meta, options))
    }

    /// Visits all leaf conditions in this filter.
    pub(crate) fn visit_conditions<F>(&self, mut visitor: F) -> MetadataResult<()>
    where
        F: FnMut(&Condition) -> MetadataResult<()>,
    {
        if let Some(expr) = &self.expr {
            expr.visit_conditions(&mut visitor)?;
        }
        Ok(())
    }
}

impl std::ops::Not for MetadataFilter {
    type Output = MetadataFilter;

    #[inline]
    fn not(self) -> Self::Output {
        MetadataFilter::not(self)
    }
}
