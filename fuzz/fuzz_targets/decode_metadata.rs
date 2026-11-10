// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Fuzzes bounded JSON decoding for [`Metadata`].

#![no_main]

#[path = "../support/bounded_encode.rs"]
mod bounded_encode;
#[path = "../support/decode_profiles.rs"]
mod decode_profiles;

use libfuzzer_sys::fuzz_target;
use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonResource;
use qubit_metadata::DEFAULT_MAX_JSON_BYTES;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataLimits;

fuzz_target!(|data: &[u8]| {
    if let Ok(value) = Metadata::decode_json_slice(data) {
        check(value, data, false);
    }
    if let Ok(value) = Metadata::decode_json_slice_with_limits(data, decode_profiles::small_metadata_limits()) {
        check(value, data, true);
    }
});

fn check(value: Metadata, data: &[u8], small: bool) {
    let encoded = serde_json::to_vec(&value).expect("metadata should serialize");
    let base = if small {
        decode_profiles::small_metadata_limits()
    } else {
        MetadataLimits::default()
    };
    let decode = MetadataLimits::default()
        .json_decode()
        .into_builder()
        .input_bytes_limit(ResourceLimit::new(
            JsonResource::InputBytes,
            encoded.len().max(DEFAULT_MAX_JSON_BYTES),
        ))
        .build();
    let limits = MetadataLimits::builder()
        .json_decode(decode)
        .max_metadata_entries(base.max_metadata_entries())
        .max_schema_fields(base.max_schema_fields())
        .max_key_bytes(base.max_key_bytes())
        .build()
        .expect("metadata limits should remain valid");
    let decoded = Metadata::decode_json_slice_with_limits(&encoded, limits)
        .expect("accepted metadata should round trip under default JSON budgets");
    assert_eq!(decoded, value);
    bounded_encode::check(
        &encoded,
        bounded_encode::caps(encoded.len(), data.first().copied().unwrap_or(0)),
        |limits| value.to_json_vec_with_limits(limits),
        |writer, limits| value.to_json_writer_with_limits(writer, limits),
    );
}
