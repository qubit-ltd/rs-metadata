// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Borrowed sequences serialized as metadata-filter wire arrays.

use qubit_value::Value;
use qubit_value::ValueWirePayloadRefV1;
use serde::Serialize;
use serde::Serializer;
use serde::ser::Error as SerError;
use serde::ser::SerializeSeq;

use super::super::FilterExpressionWireV1Ref;
use crate::FilterExpression;

/// Borrowed values or child expressions emitted as a wire sequence.
pub(crate) enum SequenceRef<'a> {
    /// Membership values serialized as borrowed V1 payloads.
    Values(
        /// Values borrowed for V1 payload serialization.
        &'a [Value],
    ),
    /// Boolean expression children serialized recursively.
    Children(
        /// Child expressions borrowed for recursive serialization.
        &'a [FilterExpression],
    ),
}

impl Serialize for SequenceRef<'_> {
    /// Serializes a borrowed value or expression sequence without collecting
    /// it.
    ///
    /// # Parameters
    ///
    /// * `serializer` - Serializer receiving the sequence elements.
    ///
    /// # Type Parameters
    ///
    /// * `S` - Serializer implementation receiving the sequence.
    ///
    /// # Errors
    ///
    /// Returns the serializer error when a sequence element cannot be encoded.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Values(values) => serialize_values(values, serializer),
            Self::Children(children) => serialize_children(children, serializer),
        }
    }
}

/// Serializes membership operands as V1 payloads.
///
/// # Type Parameters
///
/// * `S` - Serializer implementation receiving the value sequence.
///
/// # Parameters
///
/// * `values` - Membership operands borrowed for serialization.
/// * `serializer` - Serializer receiving the sequence elements.
///
/// # Errors
///
/// Returns the serializer error when a value payload cannot be encoded.
fn serialize_values<S>(values: &[Value], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut sequence = serializer.serialize_seq(Some(values.len()))?;
    for value in values {
        let payload = ValueWirePayloadRefV1::try_from(value).map_err(S::Error::custom)?;
        sequence.serialize_element(&payload)?;
    }
    sequence.end()
}

/// Serializes boolean expression children recursively.
///
/// # Type Parameters
///
/// * `S` - Serializer implementation receiving the child sequence.
///
/// # Parameters
///
/// * `children` - Child expressions borrowed for serialization.
/// * `serializer` - Serializer receiving the sequence elements.
///
/// # Errors
///
/// Returns the serializer error when a child expression cannot be encoded.
fn serialize_children<S>(children: &[FilterExpression], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut sequence = serializer.serialize_seq(Some(children.len()))?;
    for child in children {
        sequence.serialize_element(&FilterExpressionWireV1Ref::borrow(child))?;
    }
    sequence.end()
}
