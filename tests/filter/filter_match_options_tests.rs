// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::FilterMatchOptions`].

use qubit_datatype::NumericComparisonPolicy;
use qubit_metadata::FilterMatchOptions;

#[test]
fn test_default_options_use_exact_numeric_comparison() {
    let options = FilterMatchOptions::default();
    assert_eq!(options.numeric_comparison_policy(), NumericComparisonPolicy::Exact);
}

#[test]
fn test_numeric_comparison_policy_builder_supports_owned_chaining() {
    let options = FilterMatchOptions::builder()
        .numeric_comparison_policy(NumericComparisonPolicy::Approximate)
        .build();

    assert_eq!(
        options.numeric_comparison_policy(),
        NumericComparisonPolicy::Approximate
    );
}
