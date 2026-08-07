// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.
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
