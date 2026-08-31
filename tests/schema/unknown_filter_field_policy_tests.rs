// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::UnknownFilterFieldPolicy`].

use qubit_metadata::UnknownFilterFieldPolicy;

#[test]
fn test_unknown_filter_field_policy_default_is_reject() {
    assert_eq!(UnknownFilterFieldPolicy::default(), UnknownFilterFieldPolicy::Reject);
}

#[test]
fn test_unknown_filter_field_policy_serde_uses_snake_case() {
    assert_eq!(
        serde_json::to_value(UnknownFilterFieldPolicy::AllowUnchecked).unwrap(),
        serde_json::json!("allow_unchecked")
    );
    assert!(serde_json::from_value::<UnknownFilterFieldPolicy>(serde_json::json!("AllowUnchecked")).is_err());
}
