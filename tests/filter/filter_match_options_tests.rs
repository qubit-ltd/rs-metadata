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
    assert_eq!(
        options.numeric_comparison_policy(),
        NumericComparisonPolicy::Exact
    );
}

#[test]
fn test_numeric_comparison_policy_supports_mutable_and_owned_chaining() {
    let mut options = FilterMatchOptions::new();
    options.set_numeric_comparison_policy(NumericComparisonPolicy::Approximate);
    assert_eq!(
        options.numeric_comparison_policy(),
        NumericComparisonPolicy::Approximate
    );

    let options = FilterMatchOptions::new()
        .with_numeric_comparison_policy(NumericComparisonPolicy::Approximate);
    assert_eq!(
        options.numeric_comparison_policy(),
        NumericComparisonPolicy::Approximate
    );
}
