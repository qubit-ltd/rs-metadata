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
use crate::MetadataWireLimitKind;
use qubit_value::ValueWireDecodeError;

/// Failure returned by a bounded metadata JSON decoding API.
#[derive(Debug)]
#[non_exhaustive]
pub enum MetadataWireDecodeError {
    /// The input was rejected before JSON parsing because it exceeded the byte
    /// limit.
    InputTooLarge {
        /// Complete received input length.
        input_bytes: usize,
        /// Largest permitted input length.
        max_input_bytes: usize,
    },
    /// The decoded metadata or schema exceeds a configured resource bound.
    LimitExceeded {
        /// Decoded resource category that exceeded its bound.
        kind: MetadataWireLimitKind,
        /// Observed resource value.
        value: usize,
        /// Largest permitted resource value.
        maximum: usize,
    },
    /// A nested Value or JSON payload exceeded a shared structural limit.
    Value(ValueWireDecodeError),
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
            Self::InputTooLarge {
                input_bytes,
                max_input_bytes,
            } => write!(
                formatter,
                "metadata JSON input has {input_bytes} bytes, exceeding the limit of {max_input_bytes} bytes"
            ),
            Self::LimitExceeded {
                kind,
                value,
                maximum,
            } => write!(
                formatter,
                "metadata JSON {kind:?} value {value} exceeds the limit of {maximum}"
            ),
            Self::Value(error) => fmt::Display::fmt(error, formatter),
            #[cfg(feature = "filter")]
            Self::Filter(error) => fmt::Display::fmt(error, formatter),
            Self::InvalidJson(error) => fmt::Display::fmt(error, formatter),
        }
    }
}

impl Error for MetadataWireDecodeError {
    /// Returns the underlying serde_json error for parsed inputs.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InputTooLarge { .. } => None,
            Self::LimitExceeded { .. } => None,
            Self::Value(error) => Some(error),
            #[cfg(feature = "filter")]
            Self::Filter(error) => Some(error),
            Self::InvalidJson(error) => Some(error),
        }
    }
}

impl From<ValueWireDecodeError> for MetadataWireDecodeError {
    /// Wraps a nested Value wire resource failure.
    fn from(error: ValueWireDecodeError) -> Self {
        match error {
            ValueWireDecodeError::InputTooLarge {
                input_bytes,
                max_input_bytes,
            } => Self::InputTooLarge {
                input_bytes,
                max_input_bytes,
            },
            ValueWireDecodeError::InvalidJson(error) => {
                Self::InvalidJson(error)
            }
            error => Self::Value(error),
        }
    }
}
