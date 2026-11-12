// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors from bounded JSON metadata wire encoding.

use std::fmt;

use qubit_budget::BudgetError;
use qubit_budget::MeasuredBudgetError;
use qubit_budget::QuantityConversionError;
use qubit_budget::json::JsonResource;
use qubit_json::decode::JsonSyntaxError;
use qubit_json::encode::JsonEncodeError;
use qubit_json::encode::JsonEncodeErrorSource;
use qubit_json::encode::JsonSerializationError;

use crate::MetadataError;

/// Failure returned by bounded metadata JSON encoding APIs.
///
/// # Examples
///
/// ```
/// use qubit_metadata::MetadataWireEncodeError;
///
/// let error = MetadataWireEncodeError::Io(std::io::Error::other("closed"));
/// assert!(error.to_string().contains("closed"));
/// ```
#[derive(Debug)]
#[non_exhaustive]
#[must_use]
pub enum MetadataWireEncodeError {
    /// The JSON document exceeded one configured resource budget.
    Budget(
        /// Shared JSON resource budget that was exceeded.
        BudgetError<JsonResource, usize>,
    ),
    /// A native JSON measurement could not be represented by the budget
    /// quantity type.
    Quantity {
        /// Resource whose measurement failed to convert.
        resource: JsonResource,
        /// Native measurement conversion failure.
        source: QuantityConversionError,
    },
    /// The metadata value exceeds the strict V1 metadata wire contract.
    ///
    /// The original domain error is preserved as the source.
    Domain(
        /// Structured metadata-domain failure.
        MetadataError,
    ),
    /// The encoded value contains invalid JSON syntax.
    Syntax(
        /// Lexical JSON syntax failure.
        JsonSyntaxError,
    ),
    /// Strict JSON serialization rejected the value during bounded encoding.
    Json(
        /// JSON serialization failure.
        JsonSerializationError,
    ),
    /// The destination writer rejected bounded JSON output.
    Io(
        /// Destination writer error.
        std::io::Error,
    ),
}

impl fmt::Display for MetadataWireEncodeError {
    /// Formats the encoding failure with its underlying error context.
    ///
    /// # Parameters
    ///
    /// * `formatter` - Formatter receiving the error message.
    ///
    /// # Returns
    ///
    /// A formatting result indicating whether the message was written.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Budget(error) => {
                write!(formatter, "JSON resource budget exceeded: {error}")
            }
            Self::Quantity { resource, source } => write!(
                formatter,
                "JSON resource quantity conversion failed for {resource:?}: {source}",
            ),
            Self::Domain(error) => {
                write!(formatter, "metadata wire contract rejected: {error}")
            }
            Self::Syntax(error) => {
                write!(formatter, "invalid JSON wire syntax: {error}")
            }
            Self::Json(error) => {
                write!(formatter, "failed to encode JSON wire value: {error}")
            }
            Self::Io(error) => {
                write!(formatter, "failed to write JSON wire value: {error}")
            }
        }
    }
}

impl std::error::Error for MetadataWireEncodeError {
    /// Returns the underlying budget, domain, syntax, JSON, or I/O error.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Budget(error) => Some(error),
            Self::Quantity { source, .. } => Some(source),
            Self::Domain(error) => Some(error),
            Self::Syntax(error) => Some(error),
            Self::Json(error) => Some(error),
            Self::Io(error) => Some(error),
        }
    }
}

impl From<JsonEncodeError<JsonResource>> for MetadataWireEncodeError {
    /// Converts a shared budget adapter error into the metadata encoding
    /// error.
    ///
    /// # Parameters
    ///
    /// * `error` - Shared JSON encoding failure to convert.
    ///
    /// # Returns
    ///
    /// The corresponding metadata wire encoding error.
    fn from(error: JsonEncodeError<JsonResource>) -> Self {
        match error.into_source() {
            JsonEncodeErrorSource::Budget(source) => match source {
                MeasuredBudgetError::Budget(error) => Self::Budget(error),
                MeasuredBudgetError::Quantity { resource, source } => {
                    Self::Quantity { resource, source }
                }
            },
            JsonEncodeErrorSource::InvalidRawJson(source) => Self::Syntax(source),
            JsonEncodeErrorSource::Serialize(source) => Self::Json(source),
            JsonEncodeErrorSource::Write(source) => Self::Io(source),
        }
    }
}

impl From<MeasuredBudgetError<JsonResource, usize>> for MetadataWireEncodeError {
    /// Converts a measured JSON budget failure into the metadata encoding
    /// error.
    ///
    /// # Parameters
    ///
    /// * `error` - Measured budget failure to convert.
    ///
    /// # Returns
    ///
    /// The corresponding metadata wire encoding error.
    fn from(error: MeasuredBudgetError<JsonResource, usize>) -> Self {
        match error {
            MeasuredBudgetError::Budget(error) => Self::Budget(error),
            MeasuredBudgetError::Quantity { resource, source } => {
                Self::Quantity { resource, source }
            }
        }
    }
}
