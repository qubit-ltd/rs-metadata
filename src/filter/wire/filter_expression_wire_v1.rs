// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! V1 wire representation of [`crate::FilterExpression`].

use qubit_value::Value;
use qubit_value::ValueWirePayloadV1;
use serde::Serialize;

use crate::Condition;
use crate::FilterExpression;
use crate::FilterLimitKind;
use crate::FilterLimits;
use crate::MetadataError;
use crate::MetadataResult;

/// One V1 expression node.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum FilterExpressionWireV1 {
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
        children: Vec<FilterExpressionWireV1>,
    },
    /// Logical OR with at least two children.
    Or {
        /// Child expressions.
        children: Vec<FilterExpressionWireV1>,
    },
    /// Logical negation.
    Not {
        /// Expression to negate.
        expression: Box<FilterExpressionWireV1>,
    },
}

impl FilterExpressionWireV1 {
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
    pub(crate) fn validate_limits(&self, limits: FilterLimits) -> MetadataResult<()> {
        let mut node_count = 0;
        self.validate_limits_at(limits, 1, &mut node_count)
    }

    /// Converts a V1 node into an expression.
    ///
    /// # Errors
    ///
    /// Returns an invalid-expression error for AND/OR groups with fewer than
    /// two children, or a hard-limit error for oversized trees.
    pub(crate) fn into_expression(self) -> MetadataResult<FilterExpression> {
        let expression = self.into_expression_unchecked()?;
        expression.validate_limits(FilterLimits::MAX)?;
        Ok(expression)
    }

    /// Converts a node recursively without repeatedly traversing the partial
    /// expression tree for hard-limit validation.
    fn into_expression_unchecked(self) -> MetadataResult<FilterExpression> {
        match self {
            Self::All => Ok(FilterExpression::match_all()),
            Self::None => Ok(FilterExpression::match_none()),
            Self::Eq { key, value } => {
                let value = Self::into_scalar_value(value)?;
                FilterExpression::condition(Condition::Equal { key, value })
            }
            Self::Ne { key, value } => {
                let value = Self::into_scalar_value(value)?;
                FilterExpression::condition(Condition::NotEqual { key, value })
            }
            Self::Lt { key, value } => {
                let value = Self::into_scalar_value(value)?;
                FilterExpression::condition(Condition::Less { key, value })
            }
            Self::Le { key, value } => {
                let value = Self::into_scalar_value(value)?;
                FilterExpression::condition(Condition::LessEqual { key, value })
            }
            Self::Gt { key, value } => {
                let value = Self::into_scalar_value(value)?;
                FilterExpression::condition(Condition::Greater { key, value })
            }
            Self::Ge { key, value } => {
                let value = Self::into_scalar_value(value)?;
                FilterExpression::condition(Condition::GreaterEqual { key, value })
            }
            Self::In { key, values } => {
                let values = Self::into_scalar_values(values)?;
                FilterExpression::condition(Condition::In { key, values })
            }
            Self::NotIn { key, values } => {
                let values = Self::into_scalar_values(values)?;
                FilterExpression::condition(Condition::NotIn { key, values })
            }
            Self::Exists { key } => FilterExpression::condition(Condition::Exists { key }),
            Self::NotExists { key } => FilterExpression::condition(Condition::NotExists { key }),
            Self::Not { expression } => Ok(expression.into_expression_unchecked()?.negated_unchecked()),
            Self::And { children } => Self::combine(children, FilterExpression::and_unchecked, "and"),
            Self::Or { children } => Self::combine(children, FilterExpression::or_unchecked, "or"),
        }
    }

    /// Recursively validates one raw wire node at `depth`.
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
            Self::Not { expression } => expression.validate_limits_at(limits, depth + 1, node_count),
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
        value
            .into_container()
            .into_scalar()
            .map_err(|_| MetadataError::InvalidFilterExpression {
                message: "metadata filter wire values must be scalar".to_owned(),
            })
    }

    /// Extracts condition values while rejecting collection-shaped payloads.
    fn into_scalar_values(values: Vec<ValueWirePayloadV1>) -> MetadataResult<Vec<Value>> {
        values.into_iter().map(Self::into_scalar_value).collect()
    }

    /// Folds a non-trivial Boolean group into an expression.
    fn combine(
        children: Vec<Self>,
        combine: fn(FilterExpression, FilterExpression) -> FilterExpression,
        operator: &'static str,
    ) -> MetadataResult<FilterExpression> {
        let mut children = children.into_iter();
        let Some(first) = children.next() else {
            return Err(MetadataError::InvalidFilterExpression {
                message: format!("'{operator}' group requires at least two children"),
            });
        };
        let Some(second) = children.next() else {
            return Err(MetadataError::InvalidFilterExpression {
                message: format!("'{operator}' group requires at least two children"),
            });
        };
        let mut expression = combine(first.into_expression_unchecked()?, second.into_expression_unchecked()?);
        for child in children {
            expression = combine(expression, child.into_expression_unchecked()?);
        }
        Ok(expression)
    }
}

#[cfg(test)]
mod tests {
    use qubit_value::MultiValues;
    use qubit_value::ValueWirePayloadV1;

    use super::FilterExpressionWireV1;
    use crate::FilterLimitKind;
    use crate::FilterLimits;
    use crate::MetadataError;

    fn non_scalar_payload() -> ValueWirePayloadV1 {
        ValueWirePayloadV1::try_from(MultiValues::Int32(vec![1, 2])).expect("collection payload should convert")
    }

    #[test]
    fn test_validate_limits_rejects_depth_nodes_key_and_membership() {
        let depth_limits = FilterLimits::builder()
            .max_depth(2)
            .build()
            .expect("limits should build");
        let too_deep = FilterExpressionWireV1::Not {
            expression: Box::new(FilterExpressionWireV1::Not {
                expression: Box::new(FilterExpressionWireV1::All),
            }),
        };
        assert!(matches!(
            too_deep.validate_limits(depth_limits),
            Err(MetadataError::FilterLimitExceeded {
                kind: FilterLimitKind::Depth,
                value: 3,
                maximum: 2,
            })
        ));

        let node_limits = FilterLimits::builder()
            .max_nodes(2)
            .build()
            .expect("limits should build");
        let too_many_nodes = FilterExpressionWireV1::And {
            children: vec![
                FilterExpressionWireV1::All,
                FilterExpressionWireV1::None,
                FilterExpressionWireV1::All,
            ],
        };
        assert!(matches!(
            too_many_nodes.validate_limits(node_limits),
            Err(MetadataError::FilterLimitExceeded {
                kind: FilterLimitKind::Nodes,
                value: 3,
                maximum: 2,
            })
        ));

        let key_limits = FilterLimits::builder()
            .max_key_bytes(4)
            .build()
            .expect("limits should build");
        let long_key = FilterExpressionWireV1::Exists {
            key: "status".to_owned(),
        };
        assert!(matches!(
            long_key.validate_limits(key_limits),
            Err(MetadataError::FilterLimitExceeded {
                kind: FilterLimitKind::KeyBytes,
                value: 6,
                maximum: 4,
            })
        ));

        let set_limits = FilterLimits::builder()
            .max_set_values(1)
            .build()
            .expect("limits should build");
        let too_many_values = FilterExpressionWireV1::In {
            key: "k".to_owned(),
            values: vec![non_scalar_payload(), non_scalar_payload()],
        };
        assert!(matches!(
            too_many_values.validate_limits(set_limits),
            Err(MetadataError::FilterLimitExceeded {
                kind: FilterLimitKind::SetValues,
                value: 2,
                maximum: 1,
            })
        ));
    }

    #[test]
    fn test_into_expression_rejects_non_scalar_operands_and_short_groups() {
        let non_scalar = FilterExpressionWireV1::Eq {
            key: "k".to_owned(),
            value: non_scalar_payload(),
        };
        assert!(matches!(
            non_scalar.into_expression(),
            Err(MetadataError::InvalidFilterExpression { .. })
        ));

        let single_child = FilterExpressionWireV1::And {
            children: vec![FilterExpressionWireV1::All],
        };
        assert!(matches!(
            single_child.into_expression(),
            Err(MetadataError::InvalidFilterExpression { .. })
        ));

        let three_children = FilterExpressionWireV1::Or {
            children: vec![
                FilterExpressionWireV1::All,
                FilterExpressionWireV1::None,
                FilterExpressionWireV1::All,
            ],
        };
        let _ = three_children
            .into_expression()
            .expect("three-child groups should fold into an expression");
    }
}
