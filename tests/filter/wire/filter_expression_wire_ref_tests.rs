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
//! Borrowed filter-expression wire tests.

use qubit_metadata::{
    FilterExpression,
    MetadataFilter,
};
use qubit_value::Value;

#[test]
fn test_expression_serialization_preserves_nested_borrowed_payloads() {
    let expression = FilterExpression::builder()
        .eq(
            "payload",
            Value::Json(serde_json::json!({
                "z": {"b": 2, "a": 1},
            })),
        )
        .and_group(|group| group.exists("owner").ne("status", "inactive"))
        .build()
        .expect("expression should build");

    let filter = MetadataFilter::builder()
        .expression(expression)
        .build()
        .expect("filter should build");
    let encoded =
        serde_json::to_value(&filter).expect("filter should serialize");
    let encoded = &encoded["expression"];

    assert_eq!(encoded["kind"], "and");
    assert!(encoded["children"].is_array());
    assert_eq!(encoded.to_string().matches("payload").count(), 1);
}
