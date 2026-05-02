/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

use qubit_metadata::{FilterMatchOptions, MissingKeyPolicy, NumberComparisonPolicy};

#[test]
fn default_options_are_stable() {
    let options = FilterMatchOptions::default();
    assert_eq!(options.missing_key_policy, MissingKeyPolicy::Match);
    assert_eq!(
        options.number_comparison_policy,
        NumberComparisonPolicy::Conservative
    );
}
