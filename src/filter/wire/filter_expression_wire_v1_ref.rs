// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Borrowed V1 wire representation of [`crate::FilterExpression`].

use qubit_value::ValueWireEncodeError;
use qubit_value::ValueWirePayloadRefV1;
use serde::Serialize;

use crate::Condition;
use crate::FilterExpression;
use crate::FilterExpressionView;

/// Borrowed V1 expression node used only for serialization.
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum FilterExpressionWireV1Ref<'a> {
    /// Constant true.
    All,
    /// Constant false.
    None,
    /// Equality condition.
    Eq {
        /// Metadata key.
        key: &'a str,
        /// Expected value.
        value: ValueWirePayloadRefV1<'a>,
    },
    /// Inequality condition.
    Ne {
        /// Metadata key.
        key: &'a str,
        /// Disallowed value.
        value: ValueWirePayloadRefV1<'a>,
    },
    /// Less-than condition.
    Lt {
        /// Metadata key.
        key: &'a str,
        /// Exclusive upper bound.
        value: ValueWirePayloadRefV1<'a>,
    },
    /// Less-or-equal condition.
    Le {
        /// Metadata key.
        key: &'a str,
        /// Inclusive upper bound.
        value: ValueWirePayloadRefV1<'a>,
    },
    /// Greater-than condition.
    Gt {
        /// Metadata key.
        key: &'a str,
        /// Exclusive lower bound.
        value: ValueWirePayloadRefV1<'a>,
    },
    /// Greater-or-equal condition.
    Ge {
        /// Metadata key.
        key: &'a str,
        /// Inclusive lower bound.
        value: ValueWirePayloadRefV1<'a>,
    },
    /// Inclusion condition.
    In {
        /// Metadata key.
        key: &'a str,
        /// Accepted values.
        values: Vec<ValueWirePayloadRefV1<'a>>,
    },
    /// Exclusion condition.
    NotIn {
        /// Metadata key.
        key: &'a str,
        /// Disallowed values.
        values: Vec<ValueWirePayloadRefV1<'a>>,
    },
    /// Existence condition.
    Exists {
        /// Metadata key.
        key: &'a str,
    },
    /// Non-existence condition.
    NotExists {
        /// Metadata key.
        key: &'a str,
    },
    /// Logical AND with at least two children.
    And {
        /// Child expressions.
        children: Vec<FilterExpressionWireV1Ref<'a>>,
    },
    /// Logical OR with at least two children.
    Or {
        /// Child expressions.
        children: Vec<FilterExpressionWireV1Ref<'a>>,
    },
    /// Logical negation.
    Not {
        /// Expression to negate.
        expression: Box<FilterExpressionWireV1Ref<'a>>,
    },
}

impl<'a> TryFrom<&'a FilterExpression> for FilterExpressionWireV1Ref<'a> {
    type Error = ValueWireEncodeError;

    /// Converts an expression into a borrowed V1 serialization node.
    fn try_from(expression: &'a FilterExpression) -> Result<Self, Self::Error> {
        Ok(match expression.view() {
            FilterExpressionView::True => Self::All,
            FilterExpressionView::False => Self::None,
            FilterExpressionView::Condition(condition) => match condition {
                Condition::Equal { key, value } => Self::Eq {
                    key,
                    value: ValueWirePayloadRefV1::try_from(value)?,
                },
                Condition::NotEqual { key, value } => Self::Ne {
                    key,
                    value: ValueWirePayloadRefV1::try_from(value)?,
                },
                Condition::Less { key, value } => Self::Lt {
                    key,
                    value: ValueWirePayloadRefV1::try_from(value)?,
                },
                Condition::LessEqual { key, value } => Self::Le {
                    key,
                    value: ValueWirePayloadRefV1::try_from(value)?,
                },
                Condition::Greater { key, value } => Self::Gt {
                    key,
                    value: ValueWirePayloadRefV1::try_from(value)?,
                },
                Condition::GreaterEqual { key, value } => Self::Ge {
                    key,
                    value: ValueWirePayloadRefV1::try_from(value)?,
                },
                Condition::In { key, values } => Self::In {
                    key,
                    values: values
                        .iter()
                        .map(ValueWirePayloadRefV1::try_from)
                        .collect::<Result<_, _>>()?,
                },
                Condition::NotIn { key, values } => Self::NotIn {
                    key,
                    values: values
                        .iter()
                        .map(ValueWirePayloadRefV1::try_from)
                        .collect::<Result<_, _>>()?,
                },
                Condition::Exists { key } => Self::Exists { key },
                Condition::NotExists { key } => Self::NotExists { key },
            },
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
