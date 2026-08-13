// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::UnknownMetadataFieldPolicy`].

use qubit_metadata::UnknownMetadataFieldPolicy;

#[test]
fn test_unknown_metadata_field_policy_default_is_reject() {
    assert_eq!(
        UnknownMetadataFieldPolicy::default(),
        UnknownMetadataFieldPolicy::Reject
    );
}

#[test]
fn test_unknown_metadata_field_policy_serde_uses_snake_case() {
    assert_eq!(
        serde_json::to_value(UnknownMetadataFieldPolicy::Allow).unwrap(),
        serde_json::json!("allow")
    );
    assert!(
        serde_json::from_value::<UnknownMetadataFieldPolicy>(
            serde_json::json!("Allow")
        )
        .is_err()
    );
}
