/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
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
