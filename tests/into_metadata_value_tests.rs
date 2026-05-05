/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for [`qubit_metadata::IntoMetadataValue`].

use qubit_metadata::IntoMetadataValue;
use qubit_value::Value;

#[test]
fn into_metadata_value_preserves_owned_string_type() {
    let value = String::from("active").into_metadata_value();

    assert_eq!(value, Value::String("active".to_string()));
}

#[test]
fn into_metadata_value_preserves_borrowed_string_type() {
    let value = "active".into_metadata_value();

    assert_eq!(value, Value::String("active".to_string()));
}

#[test]
fn into_metadata_value_preserves_integer_type() {
    let value = 42_i64.into_metadata_value();

    assert_eq!(value, Value::Int64(42));
}
