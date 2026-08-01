// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Borrowed V4 wire envelope for [`crate::MetadataFilter`].

use qubit_value::ValueWireEncodeError;
use serde::Serialize;

use super::{
    FilterExpressionWireRef,
    FilterLimitsWire,
};
use crate::MetadataFilter;

use super::metadata_filter_wire::METADATA_FILTER_WIRE_VERSION;

/// Borrowed V4 metadata-filter envelope used only for serialization.
#[derive(Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MetadataFilterWireRef<'a> {
    /// Wire-format version.
    version: u8,
    /// Root Boolean expression.
    expression: FilterExpressionWireRef<'a>,
    /// Evaluation options.
    options: crate::FilterMatchOptions,
    /// Expression limits declared by the sender.
    limits: FilterLimitsWire,
}

impl<'a> TryFrom<&'a MetadataFilter> for MetadataFilterWireRef<'a> {
    type Error = ValueWireEncodeError;

    /// Converts a filter into a borrowed strict v4 envelope.
    fn try_from(filter: &'a MetadataFilter) -> Result<Self, Self::Error> {
        Ok(Self {
            version: METADATA_FILTER_WIRE_VERSION,
            expression: FilterExpressionWireRef::try_from(filter.expression())?,
            options: filter.options(),
            limits: FilterLimitsWire::from(filter.limits()),
        })
    }
}
