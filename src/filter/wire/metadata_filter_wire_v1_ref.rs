// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Borrowed V1 wire envelope for [`crate::MetadataFilter`].

use qubit_value::ValueWireEncodeError;
use serde::Serialize;
use serde::Serializer;
use serde::ser::SerializeStruct;

use super::FilterExpressionWireV1Ref;
use super::metadata_filter_wire_v1::METADATA_FILTER_WIRE_VERSION_V1;
use crate::FilterMatchOptions;
use crate::MetadataFilter;

/// Borrowed V1 metadata-filter envelope used only for serialization.
#[derive(Clone, Copy)]
pub(crate) struct MetadataFilterWireV1Ref<'a> {
    filter: &'a MetadataFilter,
    expression: FilterExpressionWireV1Ref<'a>,
}

impl<'a> TryFrom<&'a MetadataFilter> for MetadataFilterWireV1Ref<'a> {
    /// Error returned when an expression value cannot be represented in V1.
    type Error = ValueWireEncodeError;

    /// Validates a filter and creates its borrowed strict V1 envelope.
    ///
    /// # Parameters
    ///
    /// * `filter` - Filter whose expression and options are borrowed.
    ///
    /// # Errors
    ///
    /// Returns an error if an expression value cannot be represented by V1.
    fn try_from(filter: &'a MetadataFilter) -> Result<Self, Self::Error> {
        let expression = FilterExpressionWireV1Ref::try_from(filter.expression())?;
        Ok(Self { filter, expression })
    }
}

impl Serialize for MetadataFilterWireV1Ref<'_> {
    /// Serializes the strict V1 envelope in canonical field order.
    ///
    /// # Parameters
    ///
    /// * `serializer` - Serializer receiving the envelope.
    ///
    /// # Errors
    ///
    /// Returns the serializer error if a field cannot be serialized.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let filter = self.filter;
        let options: FilterMatchOptions = filter.options();
        let mut envelope = serializer.serialize_struct("MetadataFilter", 3)?;
        envelope.serialize_field("version", &METADATA_FILTER_WIRE_VERSION_V1)?;
        envelope.serialize_field("expression", &self.expression)?;
        envelope.serialize_field("options", &options)?;
        envelope.end()
    }
}
