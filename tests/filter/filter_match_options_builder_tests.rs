// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::FilterMatchOptionsBuilder`].

use qubit_datatype::NumericComparisonPolicy;
use qubit_metadata::FilterMatchOptions;

#[test]
fn test_numeric_comparison_policy_uses_last_value() {
    let options = FilterMatchOptions::builder()
        .numeric_comparison_policy(NumericComparisonPolicy::Approximate)
        .numeric_comparison_policy(NumericComparisonPolicy::Exact)
        .build();
    assert_eq!(options.numeric_comparison_policy(), NumericComparisonPolicy::Exact);
}
