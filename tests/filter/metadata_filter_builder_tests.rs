// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Unit tests for [`qubit_metadata::MetadataFilterBuilder`] default behavior.
use qubit_metadata::{
    MetadataError,
    MetadataFilterBuilder,
};

#[test]
fn test_builder_default_builds_match_all_filter() {
    let filter = MetadataFilterBuilder::default().build().unwrap();
    assert!(filter.matches(&qubit_metadata::Metadata::new()));
}

#[test]
fn test_build_rejects_membership_condition_exceeding_filter_limit() {
    let values: Vec<i64> = (0_i64..300_i64).collect();

    let error = MetadataFilterBuilder::default()
        .in_set("tag", values)
        .build()
        .expect_err(
            "membership condition exceeding the limit must be rejected",
        );

    assert!(matches!(
        error,
        MetadataError::FilterLimitExceeded {
            limit: "set values",
            ..
        }
    ));
}
