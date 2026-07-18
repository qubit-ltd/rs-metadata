// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::UnknownFieldPolicy`].

use qubit_metadata::UnknownFieldPolicy;

#[test]
fn test_unknown_field_policy_default_is_reject() {
    assert_eq!(UnknownFieldPolicy::default(), UnknownFieldPolicy::Reject);
}

#[test]
fn test_unknown_field_policy_serde_uses_snake_case() {
    assert_eq!(
        serde_json::to_value(UnknownFieldPolicy::Allow).unwrap(),
        serde_json::json!("allow")
    );
    assert!(
        serde_json::from_value::<UnknownFieldPolicy>(serde_json::json!(
            "Allow"
        ))
        .is_err()
    );
}
