// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Borrowed V1 wire representation of [`crate::FilterExpression`].

use qubit_value::Value;
use qubit_value::ValueRef;
use qubit_value::ValueWireEncodeError;
use qubit_value::ValueWirePayloadRefV1;
use serde::Serialize;
use serde::Serializer;
use serde::ser::Error as SerError;
use serde::ser::SerializeSeq;
use serde::ser::SerializeStruct;

use crate::Condition;
use crate::FilterExpression;
use crate::FilterExpressionView;

/// Borrowed V1 expression node used only for serialization.
#[derive(Clone, Copy)]
pub(crate) struct FilterExpressionWireV1Ref<'a>(&'a FilterExpression);

impl<'a> TryFrom<&'a FilterExpression> for FilterExpressionWireV1Ref<'a> {
    /// Error returned when a condition value cannot be represented in V1.
    type Error = ValueWireEncodeError;

    /// Validates an expression and creates its borrowed V1 serialization view.
    ///
    /// # Parameters
    ///
    /// * `expression` - Expression whose values remain borrowed for serialization.
    ///
    /// # Errors
    ///
    /// Returns an error if any condition value cannot be represented by V1.
    fn try_from(expression: &'a FilterExpression) -> Result<Self, Self::Error> {
        validate_expression(expression)?;
        Ok(Self(expression))
    }
}

/// Validates every operand in depth-first wire order without allocating.
fn validate_expression(expression: &FilterExpression) -> Result<(), ValueWireEncodeError> {
    match expression.view() {
        FilterExpressionView::True | FilterExpressionView::False => Ok(()),
        FilterExpressionView::Condition(condition) => validate_condition(condition),
        FilterExpressionView::And(children) | FilterExpressionView::Or(children) => {
            for child in children {
                validate_expression(child)?;
            }
            Ok(())
        }
        FilterExpressionView::Not(inner) => validate_expression(inner),
    }
}

/// Validates every value in one condition in wire order without allocating.
fn validate_condition(condition: &Condition) -> Result<(), ValueWireEncodeError> {
    match condition {
        Condition::Equal { value, .. }
        | Condition::NotEqual { value, .. }
        | Condition::Less { value, .. }
        | Condition::LessEqual { value, .. }
        | Condition::Greater { value, .. }
        | Condition::GreaterEqual { value, .. } => {
            validate_value(value)?;
        }
        Condition::In { values, .. } | Condition::NotIn { values, .. } => {
            for value in values {
                validate_value(value)?;
            }
        }
        Condition::Exists { .. } | Condition::NotExists { .. } => {}
    }
    Ok(())
}

/// Preflights V1 representation constraints without rebuilding safe integers.
fn validate_value(value: &Value) -> Result<(), ValueWireEncodeError> {
    // Every i64 has a direct V1 representation; no ValueWireEncodeError case
    // applies to this type. Avoid validating it again before serialization.
    if matches!(value.view(), ValueRef::Int64(_)) {
        return Ok(());
    }
    let _ = ValueWirePayloadRefV1::try_from(value)?;
    Ok(())
}

struct ValuesRef<'a>(&'a [Value]);

/// Serializes borrowed membership values without collecting them.
impl Serialize for ValuesRef<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for value in self.0 {
            let payload = ValueWirePayloadRefV1::try_from(value).map_err(S::Error::custom)?;
            sequence.serialize_element(&payload)?;
        }
        sequence.end()
    }
}

struct ChildrenRef<'a>(&'a [FilterExpression]);

/// Serializes borrowed child expressions without collecting them.
impl Serialize for ChildrenRef<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for child in self.0 {
            sequence.serialize_element(&FilterExpressionWireV1Ref(child))?;
        }
        sequence.end()
    }
}

impl Serialize for FilterExpressionWireV1Ref<'_> {
    /// Serializes one expression node with the canonical V1 field order.
    ///
    /// # Parameters
    ///
    /// * `serializer` - Serializer receiving this expression node.
    ///
    /// # Errors
    ///
    /// Returns the serializer error if a field or nested node cannot be serialized.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0.view() {
            FilterExpressionView::True => {
                let mut node = serializer.serialize_struct("FilterExpression", 1)?;
                node.serialize_field("kind", "all")?;
                node.end()
            }
            FilterExpressionView::False => {
                let mut node = serializer.serialize_struct("FilterExpression", 1)?;
                node.serialize_field("kind", "none")?;
                node.end()
            }
            FilterExpressionView::Condition(condition) => {
                serialize_condition(condition, serializer)
            }
            FilterExpressionView::And(children) => {
                let mut node = serializer.serialize_struct("FilterExpression", 2)?;
                node.serialize_field("kind", "and")?;
                node.serialize_field("children", &ChildrenRef(children))?;
                node.end()
            }
            FilterExpressionView::Or(children) => {
                let mut node = serializer.serialize_struct("FilterExpression", 2)?;
                node.serialize_field("kind", "or")?;
                node.serialize_field("children", &ChildrenRef(children))?;
                node.end()
            }
            FilterExpressionView::Not(inner) => {
                let mut node = serializer.serialize_struct("FilterExpression", 2)?;
                node.serialize_field("kind", "not")?;
                node.serialize_field("expression", &FilterExpressionWireV1Ref(inner))?;
                node.end()
            }
        }
    }
}

/// Serializes a condition node using its canonical V1 operator name.
fn serialize_condition<S>(condition: &Condition, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match condition {
        Condition::Equal { key, value } => serialize_value_condition("eq", key, value, serializer),
        Condition::NotEqual { key, value } => {
            serialize_value_condition("ne", key, value, serializer)
        }
        Condition::Less { key, value } => serialize_value_condition("lt", key, value, serializer),
        Condition::LessEqual { key, value } => {
            serialize_value_condition("le", key, value, serializer)
        }
        Condition::Greater { key, value } => {
            serialize_value_condition("gt", key, value, serializer)
        }
        Condition::GreaterEqual { key, value } => {
            serialize_value_condition("ge", key, value, serializer)
        }
        Condition::In { key, values } => serialize_values_condition("in", key, values, serializer),
        Condition::NotIn { key, values } => {
            serialize_values_condition("not_in", key, values, serializer)
        }
        Condition::Exists { key } => serialize_key_condition("exists", key, serializer),
        Condition::NotExists { key } => serialize_key_condition("not_exists", key, serializer),
    }
}

/// Serializes a comparison node containing one borrowed value.
fn serialize_value_condition<S>(
    kind: &'static str,
    key: &str,
    value: &Value,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let value = ValueWirePayloadRefV1::try_from(value).map_err(S::Error::custom)?;
    let mut node = serializer.serialize_struct("FilterExpression", 3)?;
    node.serialize_field("kind", kind)?;
    node.serialize_field("key", key)?;
    node.serialize_field("value", &value)?;
    node.end()
}

/// Serializes a membership node containing a borrowed value sequence.
fn serialize_values_condition<S>(
    kind: &'static str,
    key: &str,
    values: &[Value],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut node = serializer.serialize_struct("FilterExpression", 3)?;
    node.serialize_field("kind", kind)?;
    node.serialize_field("key", key)?;
    node.serialize_field("values", &ValuesRef(values))?;
    node.end()
}

/// Serializes an existence node containing only its key.
fn serialize_key_condition<S>(
    kind: &'static str,
    key: &str,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut node = serializer.serialize_struct("FilterExpression", 2)?;
    node.serialize_field("kind", kind)?;
    node.serialize_field("key", key)?;
    node.end()
}
