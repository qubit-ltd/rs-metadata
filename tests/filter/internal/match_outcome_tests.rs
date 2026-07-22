// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Behavioral tests for private three-valued match outcomes.

use qubit_metadata::{Metadata, MetadataFilter};

#[test]
fn test_match_outcome_unknown_remains_non_matching_after_negation() {
    let filter = MetadataFilter::builder()
        .eq("missing", "value")
        .not()
        .build()
        .expect("filter should build");

    assert!(!filter.matches(&Metadata::new()));
}
