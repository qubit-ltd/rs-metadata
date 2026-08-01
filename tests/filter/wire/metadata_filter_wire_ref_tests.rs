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
//! Borrowed metadata-filter wire tests.

use qubit_metadata::{
    FilterExpression,
    MetadataFilter,
};

#[test]
fn test_metadata_filter_serialization_keeps_wire_version_and_expression() {
    let expression = FilterExpression::builder()
        .exists("owner")
        .build()
        .expect("expression should build");
    let filter = MetadataFilter::builder()
        .expression(expression)
        .build()
        .expect("filter should build");

    let encoded =
        serde_json::to_value(&filter).expect("filter should serialize");

    assert_eq!(encoded["version"], 4);
    assert_eq!(encoded["expression"]["kind"], "exists");
    assert_eq!(encoded["expression"]["key"], "owner");
}
