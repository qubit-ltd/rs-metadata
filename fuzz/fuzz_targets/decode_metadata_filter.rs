// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Fuzzes bounded JSON decoding for [`MetadataFilter`].
//!
//! Every accepted input must serialize and decode back to the same filter.
//! Deliberately small expression limits keep malformed-input exploration
//! bounded while exercising nested nodes, set operands, and key limits.

#![no_main]

#[path = "../support/bounded_encode.rs"]
mod bounded_encode;

use libfuzzer_sys::fuzz_target;
use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonResource;
use qubit_metadata::DEFAULT_MAX_JSON_BYTES;
use qubit_metadata::FilterLimits;
use qubit_metadata::MetadataFilter;
use qubit_metadata::MetadataLimits;

fuzz_target!(|data: &[u8]| {
    let receiver_limits = FilterLimits::builder()
        .max_depth(4)
        .max_nodes(16)
        .max_set_values(8)
        .max_key_bytes(32)
        .build()
        .expect("fixed fuzz limits should be valid");
    if let Ok(value) = MetadataFilter::decode_json_slice_with_limits(data, MetadataLimits::default(), receiver_limits) {
        let encoded = serde_json::to_vec(&value).expect("filter should serialize");
        let input_limit = encoded.len().max(DEFAULT_MAX_JSON_BYTES);
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
        let decoded = MetadataFilter::decode_json_slice_with_limits(&encoded, decode_limits, receiver_limits)
            .expect("filter should round trip");
        assert_eq!(decoded, value);
        bounded_encode::check(
            &encoded,
            bounded_encode::caps(encoded.len(), data.first().copied().unwrap_or(0)),
            |limits| value.to_json_vec_with_limits(limits),
            |writer, limits| value.to_json_writer_with_limits(writer, limits),
        );
    }
});
