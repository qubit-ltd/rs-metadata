/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! [`MetadataError`] — failures from explicit metadata APIs and schema checks.

use std::fmt;

use qubit_datatype::DataType;
use qubit_value::{Value, ValueError};

/// Errors produced by explicit metadata accessors and schema validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataError {
    /// The requested key does not exist.
    MissingKey(String),
    /// A stored value cannot be converted to the requested type.
    TypeMismatch {
        /// Metadata key being read or validated.
        key: String,
        /// Expected data type.
        expected: DataType,
        /// Actual stored data type.
        actual: DataType,
        /// Human-readable conversion or validation message.
        message: String,
    },
    /// A required schema field is missing from a metadata object.
    MissingRequiredField {
        /// Required metadata key.
        key: String,
        /// Expected data type for the missing field.
        expected: DataType,
    },
    /// A metadata object contains a key not accepted by the schema.
    UnknownField {
        /// Unknown metadata key.
        key: String,
    },
    /// A filter references a key that is not defined by the schema.
    UnknownFilterField {
        /// Unknown filter key.
        key: String,
    },
    /// A filter uses an operator that is not compatible with the field type.
    InvalidFilterOperator {
        /// Metadata key being filtered.
        key: String,
        /// Filter operator name.
        operator: &'static str,
        /// Field data type defined by the schema.
        data_type: DataType,
        /// Human-readable validation message.
        message: String,
    },
    /// A filter expression is structurally invalid.
    InvalidFilterExpression {
        /// Human-readable validation message.
        message: String,
    },
}

impl MetadataError {
    /// Builds a conversion error for `key` using the requested type and stored value.
    #[inline]
    pub(crate) fn conversion_error(
        key: &str,
        expected: DataType,
        value: &Value,
        error: ValueError,
    ) -> Self {
        Self::TypeMismatch {
            key: key.to_string(),
            expected,
            actual: value.data_type(),
            message: error.to_string(),
        }
    }

    /// Builds a schema type-mismatch error for `key`.
    #[inline]
    pub(crate) fn type_mismatch(key: &str, expected: DataType, actual: DataType) -> Self {
        Self::TypeMismatch {
            key: key.to_string(),
            expected,
            actual,
            message: format!("expected {expected}, got {actual}"),
        }
    }
}

impl fmt::Display for MetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingKey(key) => write!(f, "Metadata key not found: {key}"),
            Self::TypeMismatch {
                key,
                expected,
                actual,
                message,
            } => write!(
                f,
                "Metadata key '{key}' expected {expected} but actual {actual}: {message}"
            ),
            Self::MissingRequiredField { key, expected } => write!(
                f,
                "Required metadata key '{key}' is missing (expected {expected})"
            ),
            Self::UnknownField { key } => {
                write!(f, "Metadata key '{key}' is not defined in schema")
            }
            Self::UnknownFilterField { key } => {
                write!(
                    f,
                    "Metadata filter references key '{key}' not defined in schema"
                )
            }
            Self::InvalidFilterOperator {
                key,
                operator,
                data_type,
                message,
            } => write!(
                f,
                "Metadata filter operator '{operator}' is invalid for key '{key}' with type {data_type}: {message}"
            ),
            Self::InvalidFilterExpression { message } => {
                write!(f, "Metadata filter expression is invalid: {message}")
            }
        }
    }
}

impl std::error::Error for MetadataError {}
