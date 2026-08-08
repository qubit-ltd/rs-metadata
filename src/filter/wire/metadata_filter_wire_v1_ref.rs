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
use crate::MetadataFilter;

use super::metadata_filter_wire_v1::METADATA_FILTER_WIRE_VERSION_V1;

/// Borrowed V1 metadata-filter envelope used only for serialization.
#[derive(Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MetadataFilterWireV1Ref<'a> {
    /// Wire-format version.
    version: u8,
    /// Root Boolean expression.
    expression: FilterExpressionWireV1Ref<'a>,
    /// Evaluation options.
    options: crate::FilterMatchOptions,
}

impl<'a> TryFrom<&'a MetadataFilter> for MetadataFilterWireV1Ref<'a> {
    type Error = ValueWireEncodeError;

    /// Converts a filter into a borrowed strict V1 envelope.
    fn try_from(filter: &'a MetadataFilter) -> Result<Self, Self::Error> {
        Ok(Self {
            version: METADATA_FILTER_WIRE_VERSION_V1,
            expression: FilterExpressionWireV1Ref::try_from(
                filter.expression(),
            )?,
            options: filter.options(),
        })
    }
}
