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
use qubit_json::encode::JsonEncodeErrorKind;
use qubit_json::encode::JsonSerializationError;

/// Failure returned by bounded metadata JSON encoding APIs.
#[derive(Debug)]
#[non_exhaustive]
pub enum MetadataWireEncodeError {
    /// The JSON document exceeded one configured resource budget.
    Budget(BudgetError<JsonResource, usize>),
    /// A native JSON measurement could not be represented by the budget
    /// quantity type.
    Quantity {
        /// Resource whose measurement failed to convert.
        resource: JsonResource,
        /// Native measurement conversion failure.
        source: QuantityConversionError,
    },
    /// The encoded value contains invalid JSON syntax.
    Syntax(JsonSyntaxError),
    /// Strict JSON serialization rejected the value during bounded encoding.
    Json(JsonSerializationError),
    /// The destination writer rejected bounded JSON output.
    Io(std::io::Error),
}

impl fmt::Display for MetadataWireEncodeError {
    /// Formats the encoding failure with its underlying error context.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Budget(error) => {
                write!(formatter, "JSON resource budget exceeded: {error}")
            }
            Self::Quantity { resource, source } => write!(
                formatter,
                "JSON resource quantity conversion failed for {resource:?}: {source}",
            ),
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
    /// Returns the underlying budget, syntax, JSON, or I/O error.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Budget(error) => Some(error),
            Self::Quantity { source, .. } => Some(source),
            Self::Syntax(error) => Some(error),
            Self::Json(error) => Some(error),
            Self::Io(error) => Some(error),
        }
    }
}

impl From<JsonEncodeError<JsonResource>> for MetadataWireEncodeError {
    /// Converts a shared budget adapter error into the metadata encoding
    /// error.
    fn from(error: JsonEncodeError<JsonResource>) -> Self {
        match error.kind() {
            JsonEncodeErrorKind::Budget => match error
                .into_budget_error()
                .expect("budget kind must retain a budget source")
            {
                MeasuredBudgetError::Budget(error) => Self::Budget(error),
                MeasuredBudgetError::Quantity { resource, source } => Self::Quantity { resource, source },
            },
            JsonEncodeErrorKind::InvalidRawJson => Self::Syntax(
                error
                    .into_syntax_error()
                    .expect("invalid raw JSON kind must retain a syntax source"),
            ),
            JsonEncodeErrorKind::Serialize => Self::Json(
                error
                    .into_serialization_error()
                    .expect("serialize kind must retain a serialization source"),
            ),
            JsonEncodeErrorKind::Write => {
                Self::Io(error.into_write_error().expect("write kind must retain an I/O source"))
            }
        }
    }
}
