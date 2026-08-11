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

/// Failure returned by a bounded metadata JSON decoding API.
#[derive(Debug)]
#[non_exhaustive]
pub enum MetadataWireDecodeError {
    /// The JSON document exceeded one shared budget limit.
    Budget(BudgetError<JsonResource, u64>),
    /// A native JSON measurement could not fit the configured budget quantity.
    Quantity {
        /// Resource whose native measurement could not be represented.
        resource: JsonResource,
        /// Exact failed native quantity conversion.
        source: QuantityConversionError,
    },
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
            Self::Quantity { resource, source } => write!(
                formatter,
                "JSON resource quantity conversion failed for {resource:?}: {source}"
            ),
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
            #[cfg(feature = "filter")]
            Self::Filter(error) => Some(error),
            Self::InvalidJson(error) => Some(error),
        }
    }
}
