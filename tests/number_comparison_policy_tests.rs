/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use qubit_metadata::NumberComparisonPolicy;

#[test]
fn number_comparison_policy_default_is_conservative() {
    assert_eq!(
        NumberComparisonPolicy::default(),
        NumberComparisonPolicy::Conservative
    );
}
