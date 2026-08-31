// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Fuzzes bounded JSON decoding for [`Metadata`].
//!
//! Every accepted input must serialize and decode back to the same value.

#![no_main]

use libfuzzer_sys::fuzz_target;
use qubit_metadata::Metadata;

fuzz_target!(|data: &[u8]| {
    if let Ok(value) = Metadata::decode_json_slice(data) {
        let encoded = serde_json::to_vec(&value).expect("metadata should serialize");
        let decoded = Metadata::decode_json_slice(&encoded).expect("metadata should round trip");
        assert_eq!(decoded, value);
    }
});
