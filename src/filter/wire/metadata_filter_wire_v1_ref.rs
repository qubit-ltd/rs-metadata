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

use super::FilterExpressionWireV1Ref;
use super::metadata_filter_wire_v1::METADATA_FILTER_WIRE_VERSION_V1;
use crate::FilterMatchOptions;
use crate::MetadataFilter;

/// Borrowed V1 metadata-filter envelope used only for serialization.
///
/// # Type Parameters
///
/// * `'a` - Lifetime of the filter data borrowed by this envelope.
#[derive(Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MetadataFilterWireV1Ref<'a> {
    /// Wire-format version.
    version: u8,
    /// Root Boolean expression.
    expression: FilterExpressionWireV1Ref<'a>,
    /// Evaluation options.
    options: FilterMatchOptions,
}

impl<'a> TryFrom<&'a MetadataFilter> for MetadataFilterWireV1Ref<'a> {
    /// Error returned when an expression value cannot be represented in V1.
    type Error = ValueWireEncodeError;

    /// Converts a filter into a borrowed strict V1 envelope.
    ///
    /// # Parameters
    ///
    /// * `filter` - Filter whose expression and options are serialized.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError`] if the filter expression contains a
    /// value that cannot be represented by the V1 wire format.
    fn try_from(filter: &'a MetadataFilter) -> Result<Self, Self::Error> {
        Ok(Self {
            version: METADATA_FILTER_WIRE_VERSION_V1,
            expression: FilterExpressionWireV1Ref::try_from(filter.expression())?,
            options: filter.options(),
        })
    }
}
