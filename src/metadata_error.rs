/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! [`MetadataError`] — failures from explicit `Metadata` accessors.

use std::fmt;

use serde_json::Value;

use crate::metadata_value_type::MetadataValueType;

/// Errors produced by explicit `Metadata` accessors such as
/// [`Metadata::try_get`](crate::Metadata::try_get) and
/// [`Metadata::try_set`](crate::Metadata::try_set).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataError {
    /// The requested key does not exist.
    MissingKey(String),
    /// Serialization into [`serde_json::Value`] failed while storing a value.
    SerializationError {
        /// Metadata key being written.
        key: String,
        /// Human-readable serde error message.
        message: String,
    },
    /// Deserialization from [`serde_json::Value`] failed while loading a value.
    DeserializationError {
        /// Metadata key being read.
        key: String,
        /// Fully-qualified Rust type name requested by the caller.
        expected: &'static str,
        /// Actual JSON value type stored under the key.
        actual: MetadataValueType,
        /// Human-readable serde error message.
        message: String,
    },
}

impl MetadataError {
    /// Constructs a deserialization error for key `key`.
    #[inline]
    pub(crate) fn deserialization_error<T>(
        key: &str,
        value: &Value,
        error: serde_json::Error,
    ) -> Self {
        Self::DeserializationError {
            key: key.to_string(),
            expected: std::any::type_name::<T>(),
            actual: MetadataValueType::of(value),
            message: error.to_string(),
        }
    }

    /// Constructs a serialization error for key `key`.
    #[inline]
    pub(crate) fn serialization_error(key: String, error: serde_json::Error) -> Self {
        Self::SerializationError {
            key,
            message: error.to_string(),
        }
    }
}

impl fmt::Display for MetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingKey(key) => write!(f, "Metadata key not found: {key}"),
            Self::SerializationError { key, message } => {
                write!(f, "Failed to serialize metadata value for key '{key}': {message}")
            }
            Self::DeserializationError {
                key,
                expected,
                actual,
                message,
            } => write!(
                f,
                "Failed to deserialize metadata key '{key}' as {expected} from JSON {actual}: {message}"
            ),
        }
    }
}

impl std::error::Error for MetadataError {}
