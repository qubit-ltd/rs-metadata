// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! [`MetadataFilterWire`] and wire version.

use serde::{
    Deserialize,
    Serialize,
};

use super::super::metadata_filter::MetadataFilter;
use super::filter_expression_wire::FilterExpressionWire;
use crate::FilterMatchOptions;

/// Current serialized metadata-filter format version.
pub(crate) const METADATA_FILTER_WIRE_VERSION: u8 = 3;

/// Versioned serialized envelope for a metadata filter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MetadataFilterWire {
    /// Wire format version.
    version: u8,
    /// Required root expression.
    expression: FilterExpressionWire,
    /// Match options stored with the filter.
    #[serde(default)]
    options: FilterMatchOptions,
}

impl MetadataFilterWire {
    /// Converts the validated wire envelope into a metadata filter.
    ///
    /// # Returns
    ///
    /// The decoded metadata filter.
    ///
    /// # Errors
    ///
    /// Returns an error for unsupported versions or invalid expression trees.
    pub(crate) fn into_filter(self) -> Result<MetadataFilter, String> {
        if self.version != METADATA_FILTER_WIRE_VERSION {
            return Err(format!(
                "unsupported MetadataFilter wire format version {}; expected {}",
                self.version, METADATA_FILTER_WIRE_VERSION
            ));
        }
        let expression = self.expression.into_expression()?;
        Ok(MetadataFilter::new(expression, self.options))
    }
}

impl From<&MetadataFilter> for MetadataFilterWire {
    #[inline]
    fn from(filter: &MetadataFilter) -> Self {
        Self {
            version: METADATA_FILTER_WIRE_VERSION,
            expression: FilterExpressionWire::from(filter.expression()),
            options: filter.options(),
        }
    }
}
