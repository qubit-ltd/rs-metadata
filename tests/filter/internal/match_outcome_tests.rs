// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for fail-closed unknown outcomes.

use qubit_metadata::FilterExpression;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataFilter;

#[test]
fn test_match_outcome_unknown_remains_non_matching_after_negation() {
    let expression = FilterExpression::builder()
        .eq("missing", "value")
        .not()
        .build()
        .unwrap();
    let filter = MetadataFilter::builder().expression(expression).build().unwrap();

    assert!(!filter.matches(&Metadata::new()));
}
