/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for [`qubit_metadata::FromMetadataValue`].

use qubit_metadata::FromMetadataValue;
use qubit_value::Value;

#[test]
fn from_metadata_value_converts_matching_scalar_type() {
    let value = Value::Int64(42);

    let converted = i64::from_metadata_value(&value).expect("i64 conversion should succeed");

    assert_eq!(converted, 42);
}

#[test]
fn from_metadata_value_reports_type_mismatch() {
    let value = Value::String("active".to_string());

    let error = i64::from_metadata_value(&value)
        .expect_err("string value should not convert to i64")
        .to_string();

    assert!(error.contains("Cannot convert 'active' to i64"));
}
