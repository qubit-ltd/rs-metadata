// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the top-level metadata filter wire envelope.

use crate::support::test_support::sample;
use qubit_datatype::NumericComparisonPolicy;
use qubit_metadata::MetadataFilter;
use serde_json::json;

#[test]
fn test_metadata_filter_wire_round_trips_options_and_expression() {
    let filter = MetadataFilter::builder()
        .eq("status", "active")
        .numeric_comparison_policy(NumericComparisonPolicy::Approximate)
        .build()
        .expect("filter should build");

    let encoded = serde_json::to_value(&filter).expect("filter should serialize");
    assert_eq!(encoded["version"], json!(3));
    assert_eq!(
        encoded["options"],
        json!({
            "numeric_comparison_policy": "approximate"
        })
    );

    let decoded: MetadataFilter =
        serde_json::from_value(encoded).expect("filter should deserialize");
    assert_eq!(decoded, filter);
    assert_eq!(decoded.options(), filter.options());
    assert!(decoded.matches(&sample()));
}

#[test]
fn test_metadata_filter_wire_encodes_explicit_true_for_match_all() {
    let encoded = serde_json::to_value(MetadataFilter::all()).expect("filter should serialize");

    assert_eq!(
        encoded,
        json!({
            "version": 3,
            "expression": {
                "type": "true"
            },
            "options": {
                "numeric_comparison_policy": "exact"
            }
        })
    );
}

#[test]
fn test_metadata_filter_wire_rejects_unsupported_version() {
    let error = serde_json::from_value::<MetadataFilter>(json!({
        "version": 4,
        "expression": { "type": "true" }
    }))
    .expect_err("unsupported wire version should be rejected")
    .to_string();

    assert!(error.contains("unsupported MetadataFilter wire format version 4"));
}

#[test]
fn test_metadata_filter_wire_rejects_previous_version() {
    let error = serde_json::from_value::<MetadataFilter>(json!({
        "version": 2,
        "expression": { "type": "true" }
    }))
    .expect_err("previous wire version should be rejected")
    .to_string();

    assert!(error.contains("unsupported MetadataFilter wire format version 2"));
}
