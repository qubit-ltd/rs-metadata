// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Metadata-filter wire tests.

use qubit_metadata::{
    FilterExpression,
    FilterLimits,
    MetadataFilter,
};
use serde_json::json;

/// Builds a filter containing one existence condition.
fn filter_for_existing_key(key: &str) -> MetadataFilter {
    let expression = FilterExpression::builder()
        .exists(key)
        .build()
        .expect("expression should build");
    MetadataFilter::builder()
        .expression(expression)
        .build()
        .expect("filter should build")
}

/// Deserializes a filter under explicit receiver limits.
fn deserialize_with_filter_limits(
    encoded: &str,
    limits: FilterLimits,
) -> Result<MetadataFilter, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_str(encoded);
    MetadataFilter::deserialize_with_filter_limits(&mut deserializer, limits)
}

#[test]
fn test_receiver_accepts_small_expression_with_wider_sender_limits() {
    let encoded =
        serde_json::to_string(&filter_for_existing_key("status")).unwrap();
    let receiver = FilterLimits::builder()
        .max_depth(1)
        .max_nodes(1)
        .max_set_values(1)
        .max_key_bytes(6)
        .build()
        .unwrap();

    let decoded = deserialize_with_filter_limits(&encoded, receiver)
        .expect("actual expression should fit receiver limits");
    assert_eq!(decoded.limits(), receiver);
}

#[test]
fn test_receiver_rejects_expression_exceeding_its_key_limit() {
    let encoded =
        serde_json::to_string(&filter_for_existing_key("status")).unwrap();
    let receiver = FilterLimits::builder().max_key_bytes(5).build().unwrap();

    let error = deserialize_with_filter_limits(&encoded, receiver).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("KeyBytes value 6 exceeds the maximum of 5")
    );
}

#[test]
fn test_receiver_rejects_expression_exceeding_sender_declaration() {
    let mut encoded =
        serde_json::to_value(filter_for_existing_key("status")).unwrap();
    encoded["limits"]["max_key_bytes"] = json!(5);

    let error = serde_json::from_value::<MetadataFilter>(encoded).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("KeyBytes value 6 exceeds the maximum of 5")
    );
}

#[test]
fn test_receiver_accepts_expression_at_exact_boundary() {
    let encoded =
        serde_json::to_string(&filter_for_existing_key("status")).unwrap();
    let receiver = FilterLimits::builder().max_key_bytes(6).build().unwrap();

    assert!(deserialize_with_filter_limits(&encoded, receiver).is_ok());
}

#[test]
fn test_metadata_filter_wire_contains_expression() {
    let expression = FilterExpression::builder()
        .exists("status")
        .build()
        .unwrap();
    let filter = MetadataFilter::builder()
        .expression(expression)
        .build()
        .unwrap();
    let encoded = serde_json::to_value(filter).unwrap();

    assert_eq!(encoded["version"], json!(4));
    assert_eq!(encoded["expression"]["kind"], json!("exists"));
    assert_eq!(
        encoded["limits"]["max_depth"],
        json!(FilterLimits::MAX.max_depth())
    );
}
