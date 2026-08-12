// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.
// =============================================================================
//! Errors from bounded JSON metadata wire encoding.

use std::fmt;

use qubit_budget::{
    BudgetError,
    JsonResource,
    JsonSerdeError,
    JsonSyntaxError,
    QuantityConversionError,
};
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

impl From<JsonSerdeError<JsonResource>> for MetadataWireEncodeError {
    /// Converts a shared budget adapter error into the metadata encoding
    /// error.
    fn from(error: JsonSerdeError<JsonResource>) -> Self {
        match error {
            JsonSerdeError::Budget(error) => Self::Budget(error),
            JsonSerdeError::Quantity { resource, source } => {
                Self::Quantity { resource, source }
            }
            JsonSerdeError::Syntax(error) => Self::Syntax(error),
            JsonSerdeError::Json(error) => Self::Json(error),
            JsonSerdeError::Io(error) => Self::Io(error),
            _ => Self::Json(JsonError::io(std::io::Error::other(
                "unsupported JSON adapter error",
            ))),
        }
    }
}
