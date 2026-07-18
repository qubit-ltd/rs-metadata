// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::NumericComparisonPolicy;
use qubit_metadata::{
    FilterMatchOptions,
    MissingKeyPolicy,
};

#[test]
fn default_options_are_stable() {
    let options = FilterMatchOptions::default();
    assert_eq!(options.missing_key_policy, MissingKeyPolicy::Match);
    assert_eq!(
        options.numeric_comparison_policy,
        NumericComparisonPolicy::Exact
    );
}
