// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Metadata-filter serde tests.

use qubit_metadata::{
    FilterExpression,
    MetadataFilter,
};
use qubit_value::Value;

#[test]
fn test_all_and_none_serde_round_trip() {
    for filter in [MetadataFilter::all(), MetadataFilter::none()] {
        let encoded =
            serde_json::to_string(&filter).expect("filter should serialize");
        let decoded: MetadataFilter =
            serde_json::from_str(&encoded).expect("filter should deserialize");
        assert_eq!(decoded, filter);
    }
}

#[test]
fn test_metadata_filter_serde_round_trip() {
    let expression = FilterExpression::builder()
        .exists("status")
        .build()
        .unwrap();
    let filter = MetadataFilter::builder()
        .expression(expression)
        .build()
        .unwrap();
    let encoded = serde_json::to_string(&filter).unwrap();
    let decoded: MetadataFilter = serde_json::from_str(&encoded).unwrap();

    assert_eq!(decoded, filter);
}

#[test]
fn test_metadata_filter_rejects_unknown_fields_and_versions() {
    let mut unknown = serde_json::to_value(MetadataFilter::all()).unwrap();
    unknown["extra"] = serde_json::json!(true);
    let mut version = serde_json::to_value(MetadataFilter::all()).unwrap();
    version["version"] = serde_json::json!(5);

    assert!(serde_json::from_value::<MetadataFilter>(unknown).is_err());
    assert!(serde_json::from_value::<MetadataFilter>(version).is_err());
}

/// Preserves complex borrowed value payloads through the filter Wire shape.
#[test]
fn test_complex_metadata_filter_serde_preserves_wire_shape() {
    let expression = FilterExpression::builder()
        .eq("status", "active")
        .in_set("priority", [1_i64, 2_i64])
        .or_group(|group| {
            group
                .eq(
                    "payload",
                    Value::Json(serde_json::json!({"z": {"b": 2, "a": 1}})),
                )
                .exists("owner")
        })
        .build()
        .expect("complex expression should build");
    let filter = MetadataFilter::builder()
        .expression(expression)
        .build()
        .expect("complex filter should build");

    let encoded =
        serde_json::to_value(&filter).expect("filter should serialize");
    assert_eq!(encoded["version"], serde_json::json!(4));
    assert_eq!(encoded["expression"]["kind"], "or");
    assert!(encoded["expression"]["children"].is_array());
    assert!(encoded.to_string().contains("\"payload\""));

    let decoded: MetadataFilter =
        serde_json::from_value(encoded).expect("filter should deserialize");
    assert_eq!(decoded, filter);
}
