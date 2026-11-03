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
use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonResource;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataLimits;

fuzz_target!(|data: &[u8]| {
    if let Ok(value) = Metadata::decode_json_slice(data) {
        let encoded = serde_json::to_vec(&value).expect("metadata should serialize");
        // Ordinary Serde may normalize numbers and slightly grow a valid input.
        // Permit only the exact encoded document size while retaining every
        // other default decoder budget and the strict metadata wire decoder.
        let input_limit = encoded.len().max(qubit_metadata::DEFAULT_MAX_JSON_BYTES);
        let decode_limits = MetadataLimits::builder()
            .json_decode(
                MetadataLimits::default()
                    .json_decode()
                    .into_builder()
                    .input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, input_limit))
                    .build(),
            )
            .build()
            .expect("default metadata limits should remain valid");
        let decoded = Metadata::decode_json_slice_with_limits(&encoded, decode_limits)
            .expect("metadata should round trip within its encoded size");
        assert_eq!(decoded, value);
    }
});
