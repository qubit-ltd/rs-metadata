// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! [`MetadataError`] — failures from explicit metadata APIs and schema checks.

use std::fmt;

use qubit_datatype::DataType;
use qubit_value::Value;
use qubit_value::ValueError;

#[cfg(feature = "filter")]
use crate::FilterLimitKind;
use crate::MetadataWireLimitKind;

/// Errors produced by explicit metadata accessors and schema validation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MetadataError {
    /// The requested key does not exist.
    MissingKey(
        /// Missing metadata key.
        String,
    ),
    /// A requested key exists but stores no concrete value.
    MissingValue {
        /// Metadata key being read.
        key: String,
        /// Data type declared by the unset value.
        data_type: DataType,
    },
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
    /// A metadata or schema value exceeds a strict V1 wire limit.
    WireLimitExceeded {
        /// Wire resource category that exceeded its limit.
        kind: MetadataWireLimitKind,
        /// Observed resource value.
        value: usize,
        /// Largest value accepted by the V1 wire contract.
        maximum: usize,
    },
    /// A required schema field is missing from a metadata object.
    #[cfg(feature = "schema")]
    MissingRequiredField {
        /// Required metadata key.
        key: String,
        /// Expected data type for the missing field.
        expected: DataType,
    },
    /// A metadata object contains a key not accepted by the schema.
    #[cfg(feature = "schema")]
    UnknownField {
        /// Unknown metadata key.
        key: String,
    },
    /// A filter references a key that is not defined by the schema.
    #[cfg(feature = "schema")]
    UnknownFilterField {
        /// Unknown filter key.
        key: String,
    },
    /// A filter uses an operator that is not compatible with the field type.
    #[cfg(feature = "schema")]
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
    #[cfg(feature = "filter")]
    InvalidFilterExpression {
        /// Human-readable validation message.
        message: String,
    },
    /// A filter condition uses an operand with no stable matching semantics.
    #[cfg(feature = "filter")]
    InvalidFilterOperand {
        /// Stable filter operator name.
        operator: &'static str,
        /// Data type declared by the rejected operand.
        data_type: DataType,
        /// Human-readable rejection reason.
        message: String,
    },
    /// A metadata-filter builder was finalized without an expression.
    #[cfg(feature = "filter")]
    MissingFilterExpression,
    /// A configured filter resource bound is outside the allowed range.
    #[cfg(feature = "filter")]
    InvalidFilterLimit {
        /// Resource category being configured.
        kind: FilterLimitKind,
        /// Requested resource bound.
        value: usize,
        /// Library hard maximum for the resource.
        maximum: usize,
    },
    /// A filter exceeds an enforced resource bound.
    #[cfg(feature = "filter")]
    FilterLimitExceeded {
        /// Resource category that exceeded its limit.
        kind: FilterLimitKind,
        /// Observed resource value.
        value: usize,
        /// Largest allowed value for the resource.
        maximum: usize,
    },
    /// A schema builder declares the same field more than once.
    #[cfg(feature = "schema")]
    DuplicateSchemaField {
        /// Duplicated schema key.
        key: String,
    },
}

impl MetadataError {
    /// Builds a conversion error for `key` using the requested type and stored
    /// value.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key being converted.
    /// * `expected` - Requested target data type.
    /// * `value` - Stored value that failed conversion.
    /// * `error` - Lower-level conversion error.
    ///
    /// # Returns
    ///
    /// A structured [`MetadataError::TypeMismatch`] error.
    #[inline]
    pub(crate) fn conversion_error(key: &str, expected: DataType, value: &Value, error: ValueError) -> Self {
        Self::TypeMismatch {
            key: key.to_string(),
            expected,
            actual: value.data_type(),
            message: error.to_string(),
        }
    }

    /// Builds a schema type-mismatch error for `key`.
    ///
    /// # Parameters
    ///
    /// * `key` - Metadata key being validated.
    /// * `expected` - Schema-declared data type.
    /// * `actual` - Actual value data type.
    ///
    /// # Returns
    ///
    /// A structured [`MetadataError::TypeMismatch`] error.
    #[cfg(feature = "schema")]
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
    /// Formats this metadata operation error for display.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingKey(key) => {
                write!(formatter, "Metadata key not found: {key}")
            }
            Self::MissingValue { key, data_type } => write!(
                formatter,
                "Metadata key '{key}' has no concrete value (declared {data_type})"
            ),
            Self::TypeMismatch {
                key,
                expected,
                actual,
                message,
            } => write!(
                formatter,
                "Metadata key '{key}' expected {expected} but actual {actual}: {message}"
            ),
            Self::WireLimitExceeded { kind, value, maximum } => write!(
                formatter,
                "Metadata wire {kind:?} value {value} exceeds the maximum of {maximum}"
            ),
            #[cfg(feature = "schema")]
            Self::MissingRequiredField { key, expected } => {
                write!(
                    formatter,
                    "Required metadata key '{key}' is missing (expected {expected})"
                )
            }
            #[cfg(feature = "schema")]
            Self::UnknownField { key } => {
                write!(formatter, "Metadata key '{key}' is not defined in schema")
            }
            #[cfg(feature = "schema")]
            Self::UnknownFilterField { key } => {
                write!(
                    formatter,
                    "Metadata filter references key '{key}' not defined in schema"
                )
            }
            #[cfg(feature = "schema")]
            Self::InvalidFilterOperator {
                key,
                operator,
                data_type,
                message,
            } => write!(
                formatter,
                "Metadata filter operator '{operator}' is invalid for key '{key}' with type {data_type}: {message}"
            ),
            #[cfg(feature = "filter")]
            Self::InvalidFilterExpression { message } => {
                write!(formatter, "Metadata filter expression is invalid: {message}")
            }
            #[cfg(feature = "filter")]
            Self::InvalidFilterOperand {
                operator,
                data_type,
                message,
            } => write!(
                formatter,
                "Metadata filter operator '{operator}' cannot use {data_type}: {message}"
            ),
            #[cfg(feature = "filter")]
            Self::MissingFilterExpression => {
                write!(formatter, "Metadata filter requires an expression")
            }
            #[cfg(feature = "filter")]
            Self::InvalidFilterLimit { kind, value, maximum } => write!(
                formatter,
                "Metadata filter {kind:?} limit {value} is outside 1..={maximum}"
            ),
            #[cfg(feature = "filter")]
            Self::FilterLimitExceeded { kind, value, maximum } => write!(
                formatter,
                "Metadata filter {kind:?} value {value} exceeds the maximum of {maximum}"
            ),
            #[cfg(feature = "schema")]
            Self::DuplicateSchemaField { key } => {
                write!(formatter, "Metadata schema declares field '{key}' more than once")
            }
        }
    }
}

impl std::error::Error for MetadataError {}
