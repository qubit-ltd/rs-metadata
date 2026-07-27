// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! V4 wire representation of [`crate::FilterExpression`].

use qubit_value::{
    Value,
    ValueWireEncodeError,
    ValueWirePayloadV1,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    Condition,
    FilterExpression,
    FilterExpressionView,
    FilterLimitKind,
    FilterLimits,
    MetadataError,
    MetadataResult,
};

/// One v4 expression node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum FilterExpressionWire {
    /// Constant true.
    All,
    /// Constant false.
    None,
    /// Equality condition.
    Eq {
        /// Metadata key.
        key: String,
        /// Expected value.
        value: ValueWirePayloadV1,
    },
    /// Inequality condition.
    Ne {
        /// Metadata key.
        key: String,
        /// Disallowed value.
        value: ValueWirePayloadV1,
    },
    /// Less-than condition.
    Lt {
        /// Metadata key.
        key: String,
        /// Exclusive upper bound.
        value: ValueWirePayloadV1,
    },
    /// Less-or-equal condition.
    Le {
        /// Metadata key.
        key: String,
        /// Inclusive upper bound.
        value: ValueWirePayloadV1,
    },
    /// Greater-than condition.
    Gt {
        /// Metadata key.
        key: String,
        /// Exclusive lower bound.
        value: ValueWirePayloadV1,
    },
    /// Greater-or-equal condition.
    Ge {
        /// Metadata key.
        key: String,
        /// Inclusive lower bound.
        value: ValueWirePayloadV1,
    },
    /// Inclusion condition.
    In {
        /// Metadata key.
        key: String,
        /// Accepted values.
        values: Vec<ValueWirePayloadV1>,
    },
    /// Exclusion condition.
    NotIn {
        /// Metadata key.
        key: String,
        /// Disallowed values.
        values: Vec<ValueWirePayloadV1>,
    },
    /// Existence condition.
    Exists {
        /// Metadata key.
        key: String,
    },
    /// Non-existence condition.
    NotExists {
        /// Metadata key.
        key: String,
    },
    /// Logical AND with at least two children.
    And {
        /// Child expressions.
        children: Vec<FilterExpressionWire>,
    },
    /// Logical OR with at least two children.
    Or {
        /// Child expressions.
        children: Vec<FilterExpressionWire>,
    },
    /// Logical negation.
    Not {
        /// Expression to negate.
        expression: Box<FilterExpressionWire>,
    },
}

impl FilterExpressionWire {
    /// Validates the unnormalized wire tree against resource limits.
    ///
    /// This check precedes conversion because Boolean normalization can remove
    /// nodes that still consumed resources while decoding the wire payload.
    ///
    /// # Parameters
    ///
    /// * `limits` - Bounds to enforce on the raw wire tree.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::FilterLimitExceeded`] when the raw tree depth,
    /// node count, key length, or membership value count exceeds a bound.
    pub(crate) fn validate_limits(
        &self,
        limits: FilterLimits,
    ) -> MetadataResult<()> {
        let mut node_count = 0;
        self.validate_limits_at(limits, 1, &mut node_count)
    }

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
                let value = Self::into_scalar_value(value)?;
                Ok(FilterExpression::condition(Condition::Equal { key, value }))
            }
            Self::Ne { key, value } => {
                let value = Self::into_scalar_value(value)?;
                Ok(FilterExpression::condition(Condition::NotEqual {
                    key,
                    value,
                }))
            }
            Self::Lt { key, value } => {
                let value = Self::into_scalar_value(value)?;
                Ok(FilterExpression::condition(Condition::Less { key, value }))
            }
            Self::Le { key, value } => {
                let value = Self::into_scalar_value(value)?;
                Ok(FilterExpression::condition(Condition::LessEqual {
                    key,
                    value,
                }))
            }
            Self::Gt { key, value } => {
                let value = Self::into_scalar_value(value)?;
                Ok(FilterExpression::condition(Condition::Greater {
                    key,
                    value,
                }))
            }
            Self::Ge { key, value } => {
                let value = Self::into_scalar_value(value)?;
                Ok(FilterExpression::condition(Condition::GreaterEqual {
                    key,
                    value,
                }))
            }
            Self::In { key, values } => {
                let values = Self::into_scalar_values(values)?;
                Ok(FilterExpression::condition(Condition::In { key, values }))
            }
            Self::NotIn { key, values } => {
                let values = Self::into_scalar_values(values)?;
                Ok(FilterExpression::condition(Condition::NotIn {
                    key,
                    values,
                }))
            }
            Self::Exists { key } => {
                Ok(FilterExpression::condition(Condition::Exists { key }))
            }
            Self::NotExists { key } => {
                Ok(FilterExpression::condition(Condition::NotExists { key }))
            }
            Self::Not { expression } => expression.into_expression()?.try_not(),
            Self::And { children } => {
                Self::combine(children, FilterExpression::try_and, "and")
            }
            Self::Or { children } => {
                Self::combine(children, FilterExpression::try_or, "or")
            }
        }
    }

    /// Recursively validates one raw wire node at `depth`.
    fn validate_limits_at(
        &self,
        limits: FilterLimits,
        depth: usize,
        node_count: &mut usize,
    ) -> MetadataResult<()> {
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
        match self {
            Self::Eq { key, .. }
            | Self::Ne { key, .. }
            | Self::Lt { key, .. }
            | Self::Le { key, .. }
            | Self::Gt { key, .. }
            | Self::Ge { key, .. }
            | Self::Exists { key }
            | Self::NotExists { key } => Self::validate_key(key, limits),
            Self::In { key, values } | Self::NotIn { key, values } => {
                Self::validate_key(key, limits)?;
                if values.len() > limits.max_set_values() {
                    return Err(MetadataError::FilterLimitExceeded {
                        kind: FilterLimitKind::SetValues,
                        value: values.len(),
                        maximum: limits.max_set_values(),
                    });
                }
                Ok(())
            }
            Self::And { children } | Self::Or { children } => {
                for child in children {
                    child.validate_limits_at(limits, depth + 1, node_count)?;
                }
                Ok(())
            }
            Self::Not { expression } => {
                expression.validate_limits_at(limits, depth + 1, node_count)
            }
            Self::All | Self::None => Ok(()),
        }
    }

    /// Validates one condition key against `limits`.
    fn validate_key(key: &str, limits: FilterLimits) -> MetadataResult<()> {
        if key.len() > limits.max_key_bytes() {
            return Err(MetadataError::FilterLimitExceeded {
                kind: FilterLimitKind::KeyBytes,
                value: key.len(),
                maximum: limits.max_key_bytes(),
            });
        }
        Ok(())
    }

    /// Extracts the scalar required by a metadata filter condition.
    fn into_scalar_value(value: ValueWirePayloadV1) -> MetadataResult<Value> {
        value.into_container().into_scalar().map_err(|_| {
            MetadataError::InvalidFilterExpression {
                message: "metadata filter wire values must be scalar"
                    .to_owned(),
            }
        })
    }

    /// Extracts condition values while rejecting collection-shaped payloads.
    fn into_scalar_values(
        values: Vec<ValueWirePayloadV1>,
    ) -> MetadataResult<Vec<Value>> {
        values.into_iter().map(Self::into_scalar_value).collect()
    }

    /// Folds a non-trivial Boolean group into an expression.
    fn combine(
        children: Vec<Self>,
        combine: fn(
            FilterExpression,
            FilterExpression,
        ) -> MetadataResult<FilterExpression>,
        operator: &'static str,
    ) -> MetadataResult<FilterExpression> {
        let mut children = children.into_iter();
        let Some(first) = children.next() else {
            return Err(MetadataError::InvalidFilterExpression {
                message: format!(
                    "'{operator}' group requires at least two children"
                ),
            });
        };
        let Some(second) = children.next() else {
            return Err(MetadataError::InvalidFilterExpression {
                message: format!(
                    "'{operator}' group requires at least two children"
                ),
            });
        };
        let mut expression =
            combine(first.into_expression()?, second.into_expression()?)?;
        for child in children {
            expression = combine(expression, child.into_expression()?)?;
        }
        Ok(expression)
    }
}

impl TryFrom<&FilterExpression> for FilterExpressionWire {
    type Error = ValueWireEncodeError;

    /// Converts an expression into its v4 node representation.
    fn try_from(expression: &FilterExpression) -> Result<Self, Self::Error> {
        Ok(match expression.view() {
            FilterExpressionView::True => Self::All,
            FilterExpressionView::False => Self::None,
            FilterExpressionView::Condition(Condition::Equal {
                key,
                value,
            }) => Self::Eq {
                key: key.clone(),
                value: ValueWirePayloadV1::try_from(value.clone())?,
            },
            FilterExpressionView::Condition(Condition::NotEqual {
                key,
                value,
            }) => Self::Ne {
                key: key.clone(),
                value: ValueWirePayloadV1::try_from(value.clone())?,
            },
            FilterExpressionView::Condition(Condition::Less { key, value }) => {
                Self::Lt {
                    key: key.clone(),
                    value: ValueWirePayloadV1::try_from(value.clone())?,
                }
            }
            FilterExpressionView::Condition(Condition::LessEqual {
                key,
                value,
            }) => Self::Le {
                key: key.clone(),
                value: ValueWirePayloadV1::try_from(value.clone())?,
            },
            FilterExpressionView::Condition(Condition::Greater {
                key,
                value,
            }) => Self::Gt {
                key: key.clone(),
                value: ValueWirePayloadV1::try_from(value.clone())?,
            },
            FilterExpressionView::Condition(Condition::GreaterEqual {
                key,
                value,
            }) => Self::Ge {
                key: key.clone(),
                value: ValueWirePayloadV1::try_from(value.clone())?,
            },
            FilterExpressionView::Condition(Condition::In { key, values }) => {
                Self::In {
                    key: key.clone(),
                    values: values
                        .iter()
                        .cloned()
                        .map(ValueWirePayloadV1::try_from)
                        .collect::<Result<_, _>>()?,
                }
            }
            FilterExpressionView::Condition(Condition::NotIn {
                key,
                values,
            }) => Self::NotIn {
                key: key.clone(),
                values: values
                    .iter()
                    .cloned()
                    .map(ValueWirePayloadV1::try_from)
                    .collect::<Result<_, _>>()?,
            },
            FilterExpressionView::Condition(Condition::Exists { key }) => {
                Self::Exists { key: key.clone() }
            }
            FilterExpressionView::Condition(Condition::NotExists { key }) => {
                Self::NotExists { key: key.clone() }
            }
            FilterExpressionView::And(children) => Self::And {
                children: children
                    .iter()
                    .map(Self::try_from)
                    .collect::<Result<_, _>>()?,
            },
            FilterExpressionView::Or(children) => Self::Or {
                children: children
                    .iter()
                    .map(Self::try_from)
                    .collect::<Result<_, _>>()?,
            },
            FilterExpressionView::Not(expression) => Self::Not {
                expression: Box::new(Self::try_from(expression)?),
            },
        })
    }
}
