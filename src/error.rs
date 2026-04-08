/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Error types and lightweight type introspection for [`Metadata`](crate::Metadata).

use std::fmt;

use serde_json::Value;

/// Coarse-grained JSON value kinds used by [`MetadataError`] and inspection APIs.
///
/// `Metadata` stores arbitrary [`serde_json::Value`] instances, so it cannot
/// recover the caller's original Rust type. `MetadataValueKind` is therefore a
/// JSON-level classification, analogous to the stricter `data_type()` concept
/// in `qubit-value`, but tailored to an open-ended JSON model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetadataValueKind {
    /// JSON `null`.
    Null,
    /// JSON boolean.
    Bool,
    /// JSON number.
    Number,
    /// JSON string.
    String,
    /// JSON array.
    Array,
    /// JSON object.
    Object,
}

impl MetadataValueKind {
    /// Returns the JSON kind of `value`.
    #[inline]
    pub fn of(value: &Value) -> Self {
        match value {
            Value::Null => Self::Null,
            Value::Bool(_) => Self::Bool,
            Value::Number(_) => Self::Number,
            Value::String(_) => Self::String,
            Value::Array(_) => Self::Array,
            Value::Object(_) => Self::Object,
        }
    }
}

impl From<&Value> for MetadataValueKind {
    #[inline]
    fn from(value: &Value) -> Self {
        Self::of(value)
    }
}

impl fmt::Display for MetadataValueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Null => "null",
            Self::Bool => "bool",
            Self::Number => "number",
            Self::String => "string",
            Self::Array => "array",
            Self::Object => "object",
        };
        f.write_str(text)
    }
}

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
        /// Actual JSON kind stored under the key.
        actual: MetadataValueKind,
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
            actual: MetadataValueKind::of(value),
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

/// Result type used by explicit `Metadata` operations that report failure
/// reasons instead of collapsing them into `None`.
pub type MetadataResult<T> = Result<T, MetadataError>;
