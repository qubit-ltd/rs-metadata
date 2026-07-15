// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::FromMetadataValue`].

use qubit_metadata::FromMetadataValue;
use qubit_value::Value;

#[test]
fn from_metadata_value_converts_matching_scalar_type() {
    let value = Value::Int64(42);

    let converted = i64::from_metadata_value(&value)
        .expect("i64 conversion should succeed");

    assert_eq!(converted, 42);
}

#[test]
fn from_metadata_value_reports_structured_conversion_failure() {
    let value = Value::String("active".to_string());

    let error = i64::from_metadata_value(&value)
        .expect_err("string value should not convert to i64")
        .to_string();

    assert!(error.contains("Invalid conversion from string to int64"));
    assert!(error.contains("invalid syntax"));
    assert!(!error.contains("active"));
}

#[test]
fn from_metadata_value_redacts_invalid_source_value() {
    let value = Value::String("credential-that-must-not-leak".to_string());

    let message = i64::from_metadata_value(&value)
        .expect_err("invalid integer syntax should fail")
        .to_string();

    assert!(!message.contains("credential-that-must-not-leak"));
    assert!(message.contains("invalid syntax"));
}
