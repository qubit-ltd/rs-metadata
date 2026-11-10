// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Fuzzes bounded JSON decoding for [`MetadataFilter`].

#![no_main]

#[path = "../support/bounded_encode.rs"]
mod bounded_encode;
#[path = "../support/decode_profiles.rs"]
mod decode_profiles;

use libfuzzer_sys::fuzz_target;
use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonResource;
use qubit_metadata::DEFAULT_MAX_JSON_BYTES;
use qubit_metadata::FilterLimits;
use qubit_metadata::MetadataFilter;
use qubit_metadata::MetadataLimits;

fuzz_target!(|data: &[u8]| {
    let default_filter = FilterLimits::builder()
        .max_depth(4)
        .max_nodes(16)
        .max_set_values(8)
        .max_key_bytes(32)
        .build()
        .expect("fixed fuzz limits valid");
    if let Ok(value) = MetadataFilter::decode_json_slice_with_limits(data, MetadataLimits::default(), default_filter) {
        check(value, data, MetadataLimits::default(), default_filter);
    }
    if let Ok(value) = MetadataFilter::decode_json_slice_with_limits(
        data,
        decode_profiles::small_metadata_limits(),
        decode_profiles::small_filter_limits(),
    ) {
        check(
            value,
            data,
            decode_profiles::small_metadata_limits(),
            decode_profiles::small_filter_limits(),
        );
    }
});

fn check(value: MetadataFilter, data: &[u8], base: MetadataLimits, filter: FilterLimits) {
    let encoded = serde_json::to_vec(&value).expect("filter should serialize");
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
    let decoded = MetadataFilter::decode_json_slice_with_limits(&encoded, limits, filter)
        .expect("accepted filter should round trip under default JSON budgets");
    assert_eq!(decoded, value);
    bounded_encode::check(
        &encoded,
        bounded_encode::caps(encoded.len(), data.first().copied().unwrap_or(0)),
        |limits| value.to_json_vec_with_limits(limits),
        |writer, limits| value.to_json_writer_with_limits(writer, limits),
    );
}
