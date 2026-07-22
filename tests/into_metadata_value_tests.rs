// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::IntoMetadataValue`].

#[cfg(feature = "big-decimal")]
use bigdecimal::BigDecimal;
#[cfg(feature = "chrono")]
use chrono::NaiveDate;
#[cfg(feature = "big-integer")]
use num_bigint::BigInt;
use qubit_metadata::IntoMetadataValue;
use qubit_value::Value;
#[cfg(feature = "url")]
use url::Url;

#[test]
fn test_into_metadata_value_preserves_owned_string_type() {
    let value = String::from("active").into_metadata_value();

    assert_eq!(value, Value::String("active".to_string()));
}

#[test]
fn test_into_metadata_value_preserves_borrowed_string_type() {
    let value = "active".into_metadata_value();

    assert_eq!(value, Value::String("active".to_string()));
}

#[test]
fn test_into_metadata_value_preserves_integer_type() {
    let value = 42_i64.into_metadata_value();

    assert_eq!(value, Value::Int64(42));
}

/// Verifies the `chrono` feature enables date metadata values.
#[cfg(feature = "chrono")]
#[test]
fn test_chrono_feature_enables_date_metadata_values() {
    let date = NaiveDate::from_ymd_opt(2026, 7, 18).expect("test date should be valid");

    assert_eq!(date.into_metadata_value(), Value::Date(date));
}

/// Verifies the `big-integer` feature enables arbitrary-precision integers.
#[cfg(feature = "big-integer")]
#[test]
fn test_big_integer_feature_enables_metadata_values() {
    let integer = BigInt::from(i128::MAX) + 1_i32;

    assert_eq!(
        integer.clone().into_metadata_value(),
        Value::BigInteger(integer)
    );
}

/// Verifies the `big-decimal` feature enables arbitrary-precision decimals.
#[cfg(feature = "big-decimal")]
#[test]
fn test_big_decimal_feature_enables_metadata_values() {
    let decimal = BigDecimal::from(42_i64);

    assert_eq!(
        decimal.clone().into_metadata_value(),
        Value::BigDecimal(decimal)
    );
}

/// Verifies the `url` feature enables URL metadata values.
#[cfg(feature = "url")]
#[test]
fn test_url_feature_enables_metadata_values() {
    let url = Url::parse("https://example.com/path").expect("test URL should be valid");

    assert_eq!(url.clone().into_metadata_value(), Value::Url(url));
}

/// Verifies the `json` feature enables JSON metadata values.
#[cfg(feature = "json")]
#[test]
fn test_json_feature_enables_metadata_values() {
    let json = serde_json::json!({"enabled": true});

    assert_eq!(json.clone().into_metadata_value(), Value::Json(json));
}
