// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! V4 [`MetadataFilter`] envelope.

use qubit_value::WireBudget;
use serde::{
    Deserialize,
    Serialize,
};

use super::{
    FilterExpressionWire,
    FilterLimitsWire,
};
use crate::{
    FilterLimits,
    FilterMatchOptions,
    MetadataError,
    MetadataFilter,
    MetadataResult,
};

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
    /// Charges the raw filter expression against a shared wire budget.
    pub(crate) fn check_wire_budget(
        &self,
        budget: &mut WireBudget,
    ) -> Result<(), qubit_value::ValueWireDecodeError> {
        self.expression.check_wire_budget(budget, 1)
    }

    /// Converts this envelope while enforcing `receiver_limits`.
    ///
    /// # Errors
    ///
    /// Returns an invalid-expression error for unsupported versions, a limit
    /// configuration error for invalid sender limits, or an expression-limit
    /// error when the expression exceeds either sender or receiver limits.
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
        let sender_limits = self.limits.into_limits()?;
        self.expression.validate_limits(sender_limits)?;
        self.expression.validate_limits(receiver_limits)?;
        let expression = self.expression.into_expression()?;
        let limits = sender_limits.constrained_by(receiver_limits);
        Ok(MetadataFilter::new(expression, self.options, limits))
    }
}
