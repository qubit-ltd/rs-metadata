// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! V1 [`MetadataFilter`] envelope.

use serde::{Deserialize, Serialize};

use super::FilterExpressionWireV1;
use crate::{FilterMatchOptions, MetadataError, MetadataFilter, MetadataResult};

/// Current serialized metadata-filter format version.
pub(crate) const METADATA_FILTER_WIRE_VERSION_V1: u8 = 1;

/// Strict V1 serialized envelope for a metadata filter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MetadataFilterWireV1 {
    /// Wire-format version.
    version: u8,
    /// Root Boolean expression.
    expression: FilterExpressionWireV1,
    /// Evaluation options.
    options: FilterMatchOptions,
}

impl MetadataFilterWireV1 {
    /// Creates a decoded V1 envelope from strict visitor output.
    #[cfg(feature = "json")]
    pub(crate) const fn new(
        version: u8,
        expression: FilterExpressionWireV1,
        options: FilterMatchOptions,
    ) -> Self {
        Self {
            version,
            expression,
            options,
        }
    }

    /// Converts this envelope while enforcing `receiver_limits`.
    ///
    /// # Errors
    ///
    /// Returns an invalid-expression error for unsupported versions or an
    /// expression-limit error when the expression exceeds receiver limits.
    pub(crate) fn into_filter(
        self,
        receiver_limits: crate::FilterLimits,
    ) -> MetadataResult<MetadataFilter> {
        if self.version != METADATA_FILTER_WIRE_VERSION_V1 {
            return Err(MetadataError::InvalidFilterExpression {
                message: format!(
                    "unsupported MetadataFilter wire format version {}; expected {}",
                    self.version, METADATA_FILTER_WIRE_VERSION_V1
                ),
            });
        }
        self.expression.validate_limits(receiver_limits)?;
        let expression = self.expression.into_expression()?;
        Ok(MetadataFilter::new(
            expression,
            self.options,
            receiver_limits,
        ))
    }
}
