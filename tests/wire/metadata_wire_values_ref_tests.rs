// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for borrowed metadata value serialization.

use qubit_metadata::Metadata;
use serde_json::json;

#[test]
fn test_metadata_serializes_values_as_v1_payloads() {
    let metadata = Metadata::new().with("count", 7_i32);
    let encoded = serde_json::to_value(metadata)
        .expect("metadata should serialize through the borrowed adapter");

    assert_eq!(encoded["values"]["count"], json!({"scalar": {"int32": 7}}));
}
