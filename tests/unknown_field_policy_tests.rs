/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use qubit_metadata::UnknownFieldPolicy;

#[test]
fn unknown_field_policy_default_is_reject() {
    assert_eq!(UnknownFieldPolicy::default(), UnknownFieldPolicy::Reject);
}
