/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Provides [`MetadataFilter`].

use qubit_value::Value;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use crate::{
    Condition, FilterMatchOptions, Metadata, MetadataFilterBuilder, MetadataResult,
    MissingKeyPolicy, NumberComparisonPolicy,
};

const METADATA_FILTER_WIRE_VERSION: u8 = 1;

/// Internal expression tree used by [`MetadataFilter`].
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MetadataFilterWire {
    #[serde(default = "metadata_filter_wire_version")]
    version: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    expr: Option<FilterExprWire>,
    #[serde(default)]
    options: FilterMatchOptions,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum FilterExprWire {
    Condition { condition: ConditionWire },
    And { children: Vec<FilterExprWire> },
    Or { children: Vec<FilterExprWire> },
    Not { expr: Box<FilterExprWire> },
    False,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
enum ConditionWire {
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

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
enum MetadataFilterInput {
    Wire(MetadataFilterWire),
    Legacy(LegacyMetadataFilterWire),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyMetadataFilterWire {
    #[serde(default)]
    expr: Option<LegacyFilterExpr>,
    #[serde(default)]
    options: FilterMatchOptions,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
enum LegacyFilterExpr {
    Condition(Condition),
    And(Vec<LegacyFilterExpr>),
    Or(Vec<LegacyFilterExpr>),
    Not(Box<LegacyFilterExpr>),
    False,
}

#[inline]
const fn metadata_filter_wire_version() -> u8 {
    METADATA_FILTER_WIRE_VERSION
}

impl From<&MetadataFilter> for MetadataFilterWire {
    fn from(filter: &MetadataFilter) -> Self {
        Self {
            version: METADATA_FILTER_WIRE_VERSION,
            expr: filter.expr.as_ref().map(FilterExprWire::from),
            options: filter.options,
        }
    }
}

impl MetadataFilterInput {
    fn into_filter(self) -> Result<MetadataFilter, String> {
        match self {
            Self::Wire(wire) => wire.into_filter(),
            Self::Legacy(legacy) => Ok(legacy.into_filter()),
        }
    }
}

impl MetadataFilterWire {
    fn into_filter(self) -> Result<MetadataFilter, String> {
        if self.version != METADATA_FILTER_WIRE_VERSION {
            return Err(format!(
                "unsupported MetadataFilter wire format version {}; expected {}",
                self.version, METADATA_FILTER_WIRE_VERSION
            ));
        }
        Ok(MetadataFilter {
            expr: self.expr.map(FilterExprWire::into_expr),
            options: self.options,
        })
    }
}

impl LegacyMetadataFilterWire {
    fn into_filter(self) -> MetadataFilter {
        MetadataFilter {
            expr: self.expr.map(LegacyFilterExpr::into_expr),
            options: self.options,
        }
    }
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
    fn into_expr(self) -> FilterExpr {
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

impl LegacyFilterExpr {
    fn into_expr(self) -> FilterExpr {
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
    fn into_condition(self) -> Condition {
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

/// An immutable, composable filter expression over [`Metadata`].
///
/// Construct filters with [`MetadataFilter::builder`]. An empty builder builds a
/// match-all filter, which makes the default behavior explicit while keeping the
/// built filter immutable.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MetadataFilter {
    /// Root expression tree. `None` means match all.
    expr: Option<FilterExpr>,
    /// Match policies used by [`MetadataFilter::matches`].
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

impl Serialize for MetadataFilter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        MetadataFilterWire::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MetadataFilter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        MetadataFilterInput::deserialize(deserializer)?
            .into_filter()
            .map_err(de::Error::custom)
    }
}

impl std::ops::Not for MetadataFilter {
    type Output = MetadataFilter;

    #[inline]
    fn not(self) -> Self::Output {
        MetadataFilter::not(self)
    }
}
