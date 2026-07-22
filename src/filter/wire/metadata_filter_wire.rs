// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! V4 [`MetadataFilter`] envelope.

use serde::{Deserialize, Serialize};

use super::{FilterExpressionWire, FilterLimitsWire};
use crate::{FilterLimits, FilterMatchOptions, MetadataError, MetadataFilter, MetadataResult};

/// Current serialized metadata-filter format version.
pub(crate) const METADATA_FILTER_WIRE_VERSION: u8 = 4;

/// Strict v4 serialized envelope for a metadata filter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MetadataFilterWire {
    /// Wire-format version.
    version: u8,
    /// Root Boolean expression.
    expression: FilterExpressionWire,
    /// Evaluation options.
    options: FilterMatchOptions,
    /// Expression limits declared by the sender.
    limits: FilterLimitsWire,
}

impl MetadataFilterWire {
    /// Converts this envelope while enforcing `receiver_limits`.
    ///
    /// # Errors
    ///
    /// Returns an invalid-expression error for unsupported versions or limits
    /// wider than the receiver, and expression-limit errors for oversized ASTs.
    pub(crate) fn into_filter(
        self,
        receiver_limits: FilterLimits,
    ) -> MetadataResult<MetadataFilter> {
        if self.version != METADATA_FILTER_WIRE_VERSION {
            return Err(MetadataError::InvalidFilterExpression {
                message: format!(
                    "unsupported MetadataFilter wire format version {}; expected {}",
                    self.version, METADATA_FILTER_WIRE_VERSION
                ),
            });
        }
        let limits = self.limits.into_limits()?;
        if limits.max_depth() > receiver_limits.max_depth()
            || limits.max_nodes() > receiver_limits.max_nodes()
            || limits.max_set_values() > receiver_limits.max_set_values()
            || limits.max_key_bytes() > receiver_limits.max_key_bytes()
        {
            return Err(MetadataError::InvalidFilterExpression {
                message: "wire filter limits exceed receiver limits".to_string(),
            });
        }
        let expression = self.expression.into_expression()?;
        expression.validate_limits(receiver_limits)?;
        expression.validate_limits(limits)?;
        Ok(MetadataFilter::new(expression, self.options, limits))
    }
}

impl From<&MetadataFilter> for MetadataFilterWire {
    /// Converts a filter into its strict v4 envelope.
    #[inline]
    fn from(filter: &MetadataFilter) -> Self {
        Self {
            version: METADATA_FILTER_WIRE_VERSION,
            expression: FilterExpressionWire::from(filter.expression()),
            options: filter.options(),
            limits: FilterLimitsWire::from(filter.limits()),
        }
    }
}
