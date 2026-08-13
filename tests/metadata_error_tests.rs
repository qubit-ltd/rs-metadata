// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::MetadataError`].

#![cfg(feature = "schema")]

use qubit_datatype::DataType;
use qubit_metadata::{FilterLimitKind, MetadataError};

#[test]
fn test_display_formats_all_variants() {
    assert_eq!(
        MetadataError::MissingKey("k".to_string()).to_string(),
        "Metadata key not found: k"
    );
    assert_eq!(
        MetadataError::MissingValue {
            key: "count".to_string(),
            data_type: DataType::Int64,
        }
        .to_string(),
        "Metadata key 'count' has no concrete value (declared int64)"
    );

    let mismatch = MetadataError::TypeMismatch {
        key: "b".to_string(),
        expected: DataType::Bool,
        actual: DataType::Int64,
        message: "bad".to_string(),
    };
    assert_eq!(
        mismatch.to_string(),
        "Metadata key 'b' expected bool but actual int64: bad"
    );

    let missing = MetadataError::MissingRequiredField {
        key: "score".to_string(),
        expected: DataType::Int64,
    };
    assert_eq!(
        missing.to_string(),
        "Required metadata key 'score' is missing (expected int64)"
    );

    let unknown = MetadataError::UnknownField {
        key: "extra".to_string(),
    };
    assert_eq!(
        unknown.to_string(),
        "Metadata key 'extra' is not defined in schema"
    );

    let unknown_filter = MetadataError::UnknownFilterField {
        key: "extra".to_string(),
    };
    assert_eq!(
        unknown_filter.to_string(),
        "Metadata filter references key 'extra' not defined in schema"
    );

    let invalid_operator = MetadataError::InvalidFilterOperator {
        key: "active".to_string(),
        operator: "gt",
        data_type: DataType::Bool,
        message: "range operators require a numeric or string field".to_string(),
    };
    assert_eq!(
        invalid_operator.to_string(),
        "Metadata filter operator 'gt' is invalid for key 'active' with type bool: range operators require a numeric or string field"
    );

    let invalid_expression = MetadataError::InvalidFilterExpression {
        message: "empty 'and' filter group is not allowed".to_string(),
    };
    assert_eq!(
        invalid_expression.to_string(),
        "Metadata filter expression is invalid: empty 'and' filter group is not allowed"
    );

    let invalid_operand = MetadataError::InvalidFilterOperand {
        operator: "eq",
        data_type: DataType::Float64,
        message: "filter operands must be representable by the V1 wire format".to_string(),
    };
    assert_eq!(
        invalid_operand.to_string(),
        "Metadata filter operator 'eq' cannot use float64: filter operands must be representable by the V1 wire format"
    );

    assert_eq!(
        MetadataError::FilterLimitExceeded {
            kind: FilterLimitKind::Depth,
            value: 65,
            maximum: 64,
        }
        .to_string(),
        "Metadata filter Depth value 65 exceeds the maximum of 64"
    );

    assert_eq!(
        MetadataError::DuplicateSchemaField {
            key: "id".to_string(),
        }
        .to_string(),
        "Metadata schema declares field 'id' more than once"
    );
}

#[test]
fn test_error_source_is_none() {
    let error = MetadataError::MissingKey("x".to_string());
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn test_partial_eq_distinct_missing_keys() {
    assert_eq!(
        MetadataError::MissingKey("a".to_string()),
        MetadataError::MissingKey("a".to_string())
    );
    assert_ne!(
        MetadataError::MissingKey("a".to_string()),
        MetadataError::MissingKey("b".to_string())
    );
}
