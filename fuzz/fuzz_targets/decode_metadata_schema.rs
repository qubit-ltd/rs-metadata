// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
#![no_main]

use libfuzzer_sys::fuzz_target;
use qubit_metadata::MetadataSchema;

fuzz_target!(|data: &[u8]| {
    if let Ok(value) = MetadataSchema::decode_json_slice(data) {
        let encoded = serde_json::to_vec(&value).expect("schema should serialize");
        let decoded =
            MetadataSchema::decode_json_slice(&encoded).expect("schema should round trip");
        assert_eq!(decoded, value);
    }
});
