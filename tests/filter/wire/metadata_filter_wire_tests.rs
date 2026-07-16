// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the top-level metadata filter wire envelope.

use crate::support::test_support::sample;
use qubit_metadata::{
    MetadataFilter,
    MissingKeyPolicy,
    NumericComparisonPolicy,
};
use serde_json::json;

#[test]
fn metadata_filter_wire_round_trips_options_and_expression() {
    let filter = MetadataFilter::builder()
        .eq("status", "active")
        .missing_key_policy(MissingKeyPolicy::NoMatch)
        .numeric_comparison_policy(NumericComparisonPolicy::Approximate)
        .build()
        .expect("filter should build");

    let encoded =
        serde_json::to_value(&filter).expect("filter should serialize");
    assert_eq!(encoded["version"], json!(2));
    assert_eq!(
        encoded["options"],
        json!({
            "missing_key_policy": "no_match",
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
fn metadata_filter_wire_omits_expression_for_match_all() {
    let encoded = serde_json::to_value(MetadataFilter::all())
        .expect("filter should serialize");

    assert_eq!(
        encoded,
        json!({
            "version": 2,
            "options": {
                "missing_key_policy": "match",
                "numeric_comparison_policy": "exact"
            }
        })
    );
}

#[test]
fn metadata_filter_wire_rejects_unsupported_version() {
    let error = serde_json::from_value::<MetadataFilter>(json!({
        "version": 3
    }))
    .expect_err("unsupported wire version should be rejected")
    .to_string();

    assert!(error.contains("unsupported MetadataFilter wire format version 3"));
}

#[test]
fn metadata_filter_wire_rejects_previous_version() {
    let error = serde_json::from_value::<MetadataFilter>(json!({
        "version": 1
    }))
    .expect_err("previous wire version should be rejected")
    .to_string();

    assert!(error.contains("unsupported MetadataFilter wire format version 1"));
}
