/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use qubit_metadata::MissingKeyPolicy;

#[test]
fn missing_key_policy_default_is_match() {
    assert_eq!(MissingKeyPolicy::default(), MissingKeyPolicy::Match);
}
