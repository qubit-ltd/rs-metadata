/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

use qubit_metadata::MissingKeyPolicy;

#[test]
fn missing_key_policy_default_is_match() {
    assert_eq!(MissingKeyPolicy::default(), MissingKeyPolicy::Match);
}
