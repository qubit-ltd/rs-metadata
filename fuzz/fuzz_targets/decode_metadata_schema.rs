// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Fuzzes bounded JSON decoding for [`MetadataSchema`].
//!
//! Every accepted input must serialize and decode back to the same schema.

#![no_main]

#[path = "../support/bounded_encode.rs"]
mod bounded_encode;

use libfuzzer_sys::fuzz_target;
use qubit_metadata::MetadataSchema;

fuzz_target!(|data: &[u8]| {
    if let Ok(value) = MetadataSchema::decode_json_slice(data) {
        let encoded = serde_json::to_vec(&value).expect("schema should serialize");
        let decoded = MetadataSchema::decode_json_slice(&encoded).expect("schema should round trip");
        assert_eq!(decoded, value);
        bounded_encode::check(
            &encoded,
            bounded_encode::caps(encoded.len(), data.first().copied().unwrap_or(0)),
            |limits| value.to_json_vec_with_limits(limits),
            |writer, limits| value.to_json_writer_with_limits(writer, limits),
        );
    }
});
