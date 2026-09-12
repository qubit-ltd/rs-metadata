// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Temporary field storage for one expression map.

use qubit_value::ValueWirePayloadV1;

use super::super::filter_expression_wire_v1::FilterExpressionWireV1;
use super::expression_kind::ExpressionKind;
use super::limit_support::ensure_absent;
use super::limit_support::required;

/// Temporarily owned fields collected from one expression map.
pub(super) struct ExpressionFields {
    /// Parsed expression kind tag.
    pub(super) kind: Option<ExpressionKind>,
    /// Optional condition key.
    pub(super) key: Option<String>,
    /// Optional single condition operand.
    pub(super) value: Option<ValueWirePayloadV1>,
    /// Optional membership operands.
    pub(super) values: Option<Vec<ValueWirePayloadV1>>,
    /// Optional Boolean children.
    pub(super) children: Option<Vec<FilterExpressionWireV1>>,
    /// Optional negated child expression.
    pub(super) expression: Option<Box<FilterExpressionWireV1>>,
}

impl ExpressionFields {
    /// Converts validated fields into the corresponding V1 wire variant.
    pub(super) fn into_wire(self) -> Result<FilterExpressionWireV1, String> {
        let Self {
            kind,
            key,
            value,
            values,
            children,
            expression,
        } = self;
        let kind = kind.ok_or_else(|| "missing field `kind`".to_owned())?;
        match kind {
            ExpressionKind::All => {
                ensure_absent(key, "key")?;
                ensure_absent(value, "value")?;
                ensure_absent(values, "values")?;
                ensure_absent(children, "children")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::All)
            }
            ExpressionKind::None => {
                ensure_absent(key, "key")?;
                ensure_absent(value, "value")?;
                ensure_absent(values, "values")?;
                ensure_absent(children, "children")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::None)
            }
            ExpressionKind::Eq => {
                ensure_absent(values, "values")?;
                ensure_absent(children, "children")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::Eq {
                    key: required(key, "key")?,
                    value: required(value, "value")?,
                })
            }
            ExpressionKind::Ne => {
                ensure_absent(values, "values")?;
                ensure_absent(children, "children")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::Ne {
                    key: required(key, "key")?,
                    value: required(value, "value")?,
                })
            }
            ExpressionKind::Lt => {
                ensure_absent(values, "values")?;
                ensure_absent(children, "children")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::Lt {
                    key: required(key, "key")?,
                    value: required(value, "value")?,
                })
            }
            ExpressionKind::Le => {
                ensure_absent(values, "values")?;
                ensure_absent(children, "children")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::Le {
                    key: required(key, "key")?,
                    value: required(value, "value")?,
                })
            }
            ExpressionKind::Gt => {
                ensure_absent(values, "values")?;
                ensure_absent(children, "children")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::Gt {
                    key: required(key, "key")?,
                    value: required(value, "value")?,
                })
            }
            ExpressionKind::Ge => {
                ensure_absent(values, "values")?;
                ensure_absent(children, "children")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::Ge {
                    key: required(key, "key")?,
                    value: required(value, "value")?,
                })
            }
            ExpressionKind::In => {
                ensure_absent(value, "value")?;
                ensure_absent(children, "children")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::In {
                    key: required(key, "key")?,
                    values: required(values, "values")?,
                })
            }
            ExpressionKind::NotIn => {
                ensure_absent(value, "value")?;
                ensure_absent(children, "children")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::NotIn {
                    key: required(key, "key")?,
                    values: required(values, "values")?,
                })
            }
            ExpressionKind::Exists => {
                ensure_absent(value, "value")?;
                ensure_absent(values, "values")?;
                ensure_absent(children, "children")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::Exists {
                    key: required(key, "key")?,
                })
            }
            ExpressionKind::NotExists => {
                ensure_absent(value, "value")?;
                ensure_absent(values, "values")?;
                ensure_absent(children, "children")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::NotExists {
                    key: required(key, "key")?,
                })
            }
            ExpressionKind::And => {
                ensure_absent(key, "key")?;
                ensure_absent(value, "value")?;
                ensure_absent(values, "values")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::And {
                    children: required(children, "children")?,
                })
            }
            ExpressionKind::Or => {
                ensure_absent(key, "key")?;
                ensure_absent(value, "value")?;
                ensure_absent(values, "values")?;
                ensure_absent(expression, "expression")?;
                Ok(FilterExpressionWireV1::Or {
                    children: required(children, "children")?,
                })
            }
            ExpressionKind::Not => {
                ensure_absent(key, "key")?;
                ensure_absent(value, "value")?;
                ensure_absent(values, "values")?;
                ensure_absent(children, "children")?;
                Ok(FilterExpressionWireV1::Not {
                    expression: required(expression, "expression")?,
                })
            }
        }
    }
}
