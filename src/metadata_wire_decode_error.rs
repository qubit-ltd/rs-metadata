// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors from bounded JSON metadata wire decoding.

use std::{
    error::Error,
    fmt,
};

#[cfg(feature = "filter")]
use crate::MetadataError;
use qubit_budget::{
    BudgetError,
    JsonResource,
    QuantityConversionError,
};
use qubit_json::JsonSyntaxError;

/// Failure returned by a bounded metadata JSON decoding API.
#[derive(Debug)]
#[non_exhaustive]
pub enum MetadataWireDecodeError {
    /// The JSON document exceeded one shared budget limit.
    Budget(BudgetError<JsonResource, usize>),
    /// A native JSON measurement could not be represented by the budget
    /// quantity.
    Quantity {
        /// Resource whose measurement failed to convert.
        resource: JsonResource,
        /// Conversion failure retaining the original measurement and target
        /// type.
        source: QuantityConversionError,
    },
    /// The JSON lexical preflight rejected the document with source details.
    Syntax(JsonSyntaxError),
    /// A decoded metadata-filter envelope violated its structured contract.
    #[cfg(feature = "filter")]
    Filter(MetadataError),
    /// JSON syntax or strict wire-envelope decoding failed after the byte limit
    /// check.
    InvalidJson(
        /// The original serde_json error.
        serde_json::Error,
    ),
}

impl fmt::Display for MetadataWireDecodeError {
    /// Formats a bounded JSON decoding failure.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Budget(error) => fmt::Display::fmt(error, formatter),
            Self::Quantity { source, .. } => {
                fmt::Display::fmt(source, formatter)
            }
            Self::Syntax(error) => fmt::Display::fmt(error, formatter),
            #[cfg(feature = "filter")]
            Self::Filter(error) => fmt::Display::fmt(error, formatter),
            Self::InvalidJson(error) => fmt::Display::fmt(error, formatter),
        }
    }
}

impl Error for MetadataWireDecodeError {
    /// Returns the budget, filter, or Serde source associated with the error.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Budget(error) => Some(error),
            Self::Quantity { source, .. } => Some(source),
            Self::Syntax(error) => Some(error),
            #[cfg(feature = "filter")]
            Self::Filter(error) => Some(error),
            Self::InvalidJson(error) => Some(error),
        }
    }
}
