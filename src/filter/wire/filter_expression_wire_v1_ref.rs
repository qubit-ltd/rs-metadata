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
use serde::ser::SerializeStruct;

use super::internal::SequenceRef;
use crate::Condition;
use crate::FilterExpression;
use crate::FilterExpressionView;

/// Borrowed V1 expression node used only for serialization.
#[derive(Clone, Copy)]
pub(crate) struct FilterExpressionWireV1Ref<'a>(
    /// Expression borrowed while its canonical V1 representation is serialized.
    &'a FilterExpression,
);

impl<'a> TryFrom<&'a FilterExpression> for FilterExpressionWireV1Ref<'a> {
    /// Error returned when a condition value cannot be represented in V1.
    type Error = ValueWireEncodeError;

    /// Validates an expression and creates its borrowed V1 serialization view.
    ///
    /// # Parameters
    ///
    /// * `expression` - Expression whose values remain borrowed for
    ///   serialization.
    ///
    /// # Errors
    ///
    /// Returns an error if any condition value cannot be represented by V1.
    fn try_from(expression: &'a FilterExpression) -> Result<Self, Self::Error> {
        validate_expression(expression)?;
        Ok(Self(expression))
    }
}

impl<'a> FilterExpressionWireV1Ref<'a> {
    /// Creates a borrowed expression view after its enclosing tree is
    /// preflighted.
    ///
    /// # Parameters
    ///
    /// * `expression` - Child expression covered by the enclosing tree
    ///   preflight.
    pub(super) const fn borrow(expression: &'a FilterExpression) -> Self {
        Self(expression)
    }
}

/// Validates every operand in depth-first wire order without allocating.
///
/// # Parameters
///
/// * `expression` - Expression whose condition values are checked.
///
/// # Errors
///
/// Returns the first V1 representation error in depth-first wire order.
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
///
/// # Parameters
///
/// * `condition` - Condition whose values are checked.
///
/// # Errors
///
/// Returns the first V1 representation error in condition wire order.
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
///
/// # Parameters
///
/// * `value` - Condition value whose V1 representation is checked.
///
/// # Errors
///
/// Returns the value's V1 representation error when it cannot be encoded.
fn validate_value(value: &Value) -> Result<(), ValueWireEncodeError> {
    // Every i64 has a direct V1 representation; no ValueWireEncodeError case
    // applies to this type. Avoid validating it again before serialization.
    if matches!(value.view(), ValueRef::Int64(_)) {
        return Ok(());
    }
    let _ = ValueWirePayloadRefV1::try_from(value)?;
    Ok(())
}

impl Serialize for FilterExpressionWireV1Ref<'_> {
    /// Serializes one expression node with the canonical V1 field order.
    ///
    /// # Parameters
    ///
    /// * `serializer` - Serializer receiving this expression node.
    ///
    /// # Type Parameters
    ///
    /// * `S` - Serializer implementation receiving the expression fields.
    ///
    /// # Errors
    ///
    /// Returns the serializer error if a field or nested node cannot be
    /// serialized.
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
            FilterExpressionView::Condition(condition) => serialize_condition(condition, serializer),
            FilterExpressionView::And(children) => {
                let mut node = serializer.serialize_struct("FilterExpression", 2)?;
                node.serialize_field("kind", "and")?;
                node.serialize_field("children", &SequenceRef::Children(children))?;
                node.end()
            }
            FilterExpressionView::Or(children) => {
                let mut node = serializer.serialize_struct("FilterExpression", 2)?;
                node.serialize_field("kind", "or")?;
                node.serialize_field("children", &SequenceRef::Children(children))?;
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
///
/// # Type Parameters
///
/// * `S` - Serializer implementation receiving the condition fields.
///
/// # Parameters
///
/// * `condition` - Condition to serialize.
/// * `serializer` - Serializer receiving the condition fields.
///
/// # Errors
///
/// Returns the serializer error when a condition field cannot be serialized.
fn serialize_condition<S>(condition: &Condition, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match condition {
        Condition::Equal { key, value } => serialize_value_condition("eq", key, value, serializer),
        Condition::NotEqual { key, value } => serialize_value_condition("ne", key, value, serializer),
        Condition::Less { key, value } => serialize_value_condition("lt", key, value, serializer),
        Condition::LessEqual { key, value } => serialize_value_condition("le", key, value, serializer),
        Condition::Greater { key, value } => serialize_value_condition("gt", key, value, serializer),
        Condition::GreaterEqual { key, value } => serialize_value_condition("ge", key, value, serializer),
        Condition::In { key, values } => serialize_values_condition("in", key, values, serializer),
        Condition::NotIn { key, values } => serialize_values_condition("not_in", key, values, serializer),
        Condition::Exists { key } => serialize_key_condition("exists", key, serializer),
        Condition::NotExists { key } => serialize_key_condition("not_exists", key, serializer),
    }
}

/// Serializes a comparison node containing one borrowed value.
///
/// # Type Parameters
///
/// * `S` - Serializer implementation receiving the condition fields.
///
/// # Parameters
///
/// * `kind` - Canonical V1 operator name.
/// * `key` - Condition key.
/// * `value` - Comparison operand borrowed for serialization.
/// * `serializer` - Serializer receiving the condition fields.
///
/// # Errors
///
/// Returns the serializer error when a condition field cannot be serialized.
fn serialize_value_condition<S>(kind: &'static str, key: &str, value: &Value, serializer: S) -> Result<S::Ok, S::Error>
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
///
/// # Type Parameters
///
/// * `S` - Serializer implementation receiving the condition fields.
///
/// # Parameters
///
/// * `kind` - Canonical V1 operator name.
/// * `key` - Condition key.
/// * `values` - Membership operands borrowed for serialization.
/// * `serializer` - Serializer receiving the condition fields.
///
/// # Errors
///
/// Returns the serializer error when a condition field cannot be serialized.
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
    node.serialize_field("values", &SequenceRef::Values(values))?;
    node.end()
}

/// Serializes an existence node containing only its key.
///
/// # Type Parameters
///
/// * `S` - Serializer implementation receiving the condition fields.
///
/// # Parameters
///
/// * `kind` - Canonical V1 operator name.
/// * `key` - Existence-test key.
/// * `serializer` - Serializer receiving the condition fields.
///
/// # Errors
///
/// Returns the serializer error when a condition field cannot be serialized.
fn serialize_key_condition<S>(kind: &'static str, key: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut node = serializer.serialize_struct("FilterExpression", 2)?;
    node.serialize_field("kind", kind)?;
    node.serialize_field("key", key)?;
    node.end()
}
