/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

use qubit_metadata::UnknownFieldPolicy;

#[test]
fn unknown_field_policy_default_is_reject() {
    assert_eq!(UnknownFieldPolicy::default(), UnknownFieldPolicy::Reject);
}
