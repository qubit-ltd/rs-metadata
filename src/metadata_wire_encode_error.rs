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
use qubit_budget::QuantityConversionError;
use qubit_budget::json::JsonResource;
use qubit_json::text::JsonEncodeError;
use qubit_json::text::JsonSyntaxError;
use serde_json::Error as JsonError;

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
    /// Serde JSON rejected the value during bounded encoding.
    Json(JsonError),
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
        match error {
            JsonEncodeError::Budget(error) => match error {
                qubit_budget::MeasuredBudgetError::Budget(error) => {
                    Self::Budget(error)
                }
                qubit_budget::MeasuredBudgetError::Quantity {
                    resource,
                    source,
                } => Self::Quantity { resource, source },
            },
            JsonEncodeError::InvalidRawJson(error) => Self::Syntax(error),
            JsonEncodeError::Serialize(error) => Self::Json(error),
            JsonEncodeError::Write(error) => Self::Io(error),
        }
    }
}
