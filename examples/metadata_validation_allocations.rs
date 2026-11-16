// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Independent-window allocation counts for request metadata validation.

use std::alloc::GlobalAlloc;
use std::alloc::Layout;
use std::alloc::System;
use std::hint::black_box;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueLimits;
use qubit_json::encode::JsonSerializationErrorKind;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataError;
use qubit_metadata::MetadataWireEncodeError;
use qubit_metadata::MetadataWireLimitKind;
use qubit_metadata::default_json_encode_limits;
use qubit_value::Value;

struct CountingSystem;

static COUNTING: AtomicBool = AtomicBool::new(false);
static CALLS: AtomicUsize = AtomicUsize::new(0);
static REQUESTED_BYTES: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingSystem {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let result = unsafe { System.alloc(layout) };
        record(layout.size());
        result
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let result = unsafe { System.alloc_zeroed(layout) };
        record(layout.size());
        result
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let result = unsafe { System.realloc(pointer, layout, new_size) };
        record(new_size);
        result
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingSystem = CountingSystem;

fn record(requested_bytes: usize) {
    if COUNTING.load(Ordering::Relaxed) {
        CALLS.fetch_add(1, Ordering::Relaxed);
        REQUESTED_BYTES.fetch_add(requested_bytes, Ordering::Relaxed);
    }
}

struct Distribution {
    calls: [usize; 100],
    requested_bytes: [usize; 100],
}

impl Distribution {
    fn summary(mut values: [usize; 100]) -> (usize, f64, usize, usize) {
        values.sort_unstable();
        (
            values[0],
            (values[49] + values[50]) as f64 / 2.0,
            values[94],
            values[99],
        )
    }
}

fn measured<F: Fn() -> Result<(), MetadataWireEncodeError>>(
    output_bytes: usize,
    expected_success: bool,
    encode: F,
) -> (usize, Distribution) {
    // Warm up outside the allocation-counting window.
    assert_eq!(encode().is_ok(), expected_success);
    let mut calls = [0; 100];
    let mut requested_bytes = [0; 100];
    for index in 0..100 {
        CALLS.store(0, Ordering::Relaxed);
        REQUESTED_BYTES.store(0, Ordering::Relaxed);
        COUNTING.store(true, Ordering::SeqCst);
        let observed = black_box(encode());
        COUNTING.store(false, Ordering::SeqCst);
        assert_eq!(observed.is_ok(), expected_success);
        calls[index] = CALLS.load(Ordering::Relaxed);
        requested_bytes[index] = REQUESTED_BYTES.load(Ordering::Relaxed);
    }
    (output_bytes, Distribution { calls, requested_bytes })
}

fn main() {
    verify_candidate_equivalence();

    // Ensure the counter is active before any reported measurement.
    CALLS.store(0, Ordering::Relaxed);
    COUNTING.store(true, Ordering::SeqCst);
    let check = vec![0_u8; 37];
    black_box(check);
    COUNTING.store(false, Ordering::SeqCst);
    assert!(
        CALLS.load(Ordering::Relaxed) > 0,
        "allocation counter self-check failed"
    );

    let limits = default_json_encode_limits()
        .into_builder()
        .output_bytes_limit(qubit_budget::ResourceLimit::new(
            qubit_budget::json::JsonResource::OutputBytes,
            16 * 1024,
        ))
        .build();
    for (name, metadata) in samples() {
        let output_bytes = serde_json::to_vec(&metadata).expect("sample JSON").len();
        let expected_success = output_bytes <= 16 * 1024;
        let current = measured(output_bytes, expected_success, || {
            metadata.to_json_vec_with_limits(limits).map(drop)
        });
        let incremental = measured(output_bytes, expected_success, || {
            validate_without_output(&metadata, limits)
        });
        report(&name, "to_vec", current);
        report(&name, "validate_api", incremental);
    }
}

fn validate_without_output(metadata: &Metadata, limits: JsonEncodeLimits) -> Result<(), MetadataWireEncodeError> {
    metadata.validate_json_with_limits(limits)
}

#[derive(Debug, PartialEq, Eq)]
enum ErrorSignature {
    Success,
    Budget(JsonResource),
    Quantity(JsonResource),
    DomainWire(MetadataWireLimitKind, usize, usize),
    DomainOther,
    Syntax,
    Json(JsonSerializationErrorKind),
    Io,
    Other,
}

fn error_signature(result: Result<(), MetadataWireEncodeError>) -> ErrorSignature {
    match result {
        Ok(()) => ErrorSignature::Success,
        Err(error) => error_signature_ref(&error),
    }
}

fn error_signature_ref(error: &MetadataWireEncodeError) -> ErrorSignature {
    match error {
        MetadataWireEncodeError::Budget(error) => ErrorSignature::Budget(*error.resource()),
        MetadataWireEncodeError::Quantity { resource, .. } => ErrorSignature::Quantity(*resource),
        MetadataWireEncodeError::Domain(MetadataError::WireLimitExceeded { kind, value, maximum }) => {
            ErrorSignature::DomainWire(*kind, *value, *maximum)
        }
        MetadataWireEncodeError::Domain(_) => ErrorSignature::DomainOther,
        MetadataWireEncodeError::Syntax(_) => ErrorSignature::Syntax,
        MetadataWireEncodeError::Json(error) => ErrorSignature::Json(error.kind()),
        MetadataWireEncodeError::Io(_) => ErrorSignature::Io,
        _ => ErrorSignature::Other,
    }
}

fn verify_candidate_equivalence() {
    let mut cases = Vec::new();
    cases.push(("empty", Metadata::new(), default_json_encode_limits()));
    cases.push((
        "scalar",
        Metadata::new().with("integer", 7_i64).with("enabled", true),
        default_json_encode_limits(),
    ));

    let mut special = Metadata::new();
    special.set("escaped", "quote=\" slash=\\ newline=\n tab=\t");
    special.set("unicode", "café 東京 🛰");
    special.set("nested", serde_json::json!({"outer": [1, {"inner": true}]}));
    cases.push(("escaped_unicode_nested", special, default_json_encode_limits()));

    cases.push((
        "non_finite",
        Metadata::new().with("nan", Value::Float64(f64::NAN)),
        default_json_encode_limits(),
    ));

    let mut too_many_entries = Metadata::new();
    for index in 0..=4096 {
        too_many_entries.set(&format!("key_{index:02}"), "x");
    }
    cases.push(("v1_entry_limit", too_many_entries, default_json_encode_limits()));
    let long_key = "k".repeat(257);
    cases.push((
        "v1_key_limit",
        Metadata::new().with(&long_key, "x"),
        default_json_encode_limits(),
    ));

    let value_budget_cases = [
        (
            "value_depth_budget",
            serde_json::json!({"nested": true}),
            JsonValueLimits::builder().max_depth(1).build(),
            JsonResource::Depth,
        ),
        (
            "value_nodes_budget",
            serde_json::json!(true),
            JsonValueLimits::builder().max_nodes(0).build(),
            JsonResource::Nodes,
        ),
        (
            "value_sequence_items_budget",
            serde_json::json!([1]),
            JsonValueLimits::builder().max_sequence_items(0).build(),
            JsonResource::SequenceItems,
        ),
        (
            "value_map_entries_budget",
            serde_json::json!({"a": 1}),
            JsonValueLimits::builder().max_map_entries(0).build(),
            JsonResource::MapEntries,
        ),
        (
            "value_key_bytes_budget",
            serde_json::json!({"a": 1}),
            JsonValueLimits::builder().max_key_bytes(0).build(),
            JsonResource::KeyBytes,
        ),
        (
            "value_string_bytes_budget",
            serde_json::json!("long"),
            JsonValueLimits::builder().max_string_bytes(2).build(),
            JsonResource::StringBytes,
        ),
        (
            "value_number_bytes_budget",
            serde_json::json!(123),
            JsonValueLimits::builder().max_number_bytes(2).build(),
            JsonResource::NumberBytes,
        ),
        (
            "value_payload_bytes_budget",
            serde_json::json!({"a": "b"}),
            JsonValueLimits::builder().max_payload_bytes(1).build(),
            JsonResource::PayloadBytes,
        ),
    ];
    for (name, json, value_limits, expected_resource) in value_budget_cases {
        let metadata = Metadata::new().with("json", Value::Json(json));
        let limits = default_json_encode_limits()
            .into_builder()
            .value_limits(value_limits)
            .build();
        let signature = compare_candidate(name, &metadata, limits);
        assert_eq!(
            signature,
            ErrorSignature::Budget(expected_resource),
            "probe should hit the selected value budget"
        );
    }

    let boundary_metadata = Metadata::new().with("edge", "boundary");
    let encoded_len = serde_json::to_vec(&boundary_metadata)
        .expect("boundary fixture should serialize")
        .len();
    for (name, cap, expected_success) in [
        ("output_cap_minus_one", encoded_len - 1, false),
        ("output_cap_exact", encoded_len, true),
        ("output_cap_plus_one", encoded_len + 1, true),
    ] {
        let limits = default_json_encode_limits()
            .into_builder()
            .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, cap))
            .build();
        let signature = compare_candidate(name, &boundary_metadata, limits);
        assert_eq!(
            matches!(signature, ErrorSignature::Success),
            expected_success,
            "candidate should match the output cap boundary"
        );
        if !expected_success {
            assert_eq!(signature, ErrorSignature::Budget(JsonResource::OutputBytes));
        }
    }

    // V1 wire-contract errors precede nested JSON value-budget failures.
    let mut wire_and_value_violation = Metadata::new().with("json", Value::Json(serde_json::json!("too long")));
    for index in 0..4096 {
        wire_and_value_violation.set(&format!("key_{index:02}"), "x");
    }
    let limits = default_json_encode_limits()
        .into_builder()
        .value_limits(JsonValueLimits::builder().max_string_bytes(1).build())
        .build();
    assert_eq!(
        compare_candidate("multiple_wire_and_value_violations", &wire_and_value_violation, limits),
        ErrorSignature::DomainWire(MetadataWireLimitKind::Entries, 4097, 4096)
    );

    // The value preflight runs before output encoding when both limits fail.
    let metadata = Metadata::new().with("json", Value::Json(serde_json::json!("too long")));
    let limits = default_json_encode_limits()
        .into_builder()
        .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 1))
        .value_limits(JsonValueLimits::builder().max_string_bytes(1).build())
        .build();
    assert_eq!(
        compare_candidate("multiple_value_and_output_violations", &metadata, limits),
        ErrorSignature::Budget(JsonResource::StringBytes)
    );

    // Exercise ordering between an output overflow and a later non-finite
    // float serialization failure. The preflight counts the float but does not
    // reject it; the JSON serializer must determine the first reported error.
    let mut output_then_serialize = Metadata::new();
    output_then_serialize.set("a_large", "x".repeat(512));
    output_then_serialize.set("z_nan", Value::Float64(f64::NAN));
    let limits = default_json_encode_limits()
        .into_builder()
        .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 128))
        .build();
    let first = compare_candidate("output_before_later_non_finite", &output_then_serialize, limits);
    assert_eq!(first, ErrorSignature::Budget(JsonResource::OutputBytes));

    let mut serialize_then_output = Metadata::new();
    serialize_then_output.set("a_nan", Value::Float64(f64::NAN));
    serialize_then_output.set("z_large", "x".repeat(512));
    let second = compare_candidate(
        "non_finite_before_later_output_overflow",
        &serialize_then_output,
        limits,
    );
    assert_eq!(
        second,
        // The metadata V1 DTO preserves the underlying rejection as the
        // stable opaque custom-serialization kind.
        ErrorSignature::Json(JsonSerializationErrorKind::CustomSerialization)
    );

    println!("semantic_probe=pass cases=20 comparison=structured_error_variant_and_resource");
}

fn compare_candidate(name: &str, metadata: &Metadata, limits: JsonEncodeLimits) -> ErrorSignature {
    let current = metadata.to_json_vec_with_limits(limits);
    let candidate = validate_without_output(metadata, limits);
    let candidate_signature = error_signature(candidate);
    let current_signature = match &current {
        Ok(_) => {
            assert_eq!(candidate_signature, ErrorSignature::Success, "{name}");
            ErrorSignature::Success
        }
        Err(error) => error_signature_ref(error),
    };
    assert_eq!(
        candidate_signature, current_signature,
        "structured error differs for {name}"
    );
    current_signature
}

fn report(name: &str, path: &str, (output_bytes, distribution): (usize, Distribution)) {
    let calls = Distribution::summary(distribution.calls);
    let bytes = Distribution::summary(distribution.requested_bytes);
    println!(
        "sample={name} path={path} output_bytes={output_bytes} alloc_calls_min={} alloc_calls_median={:.1} alloc_calls_p95={} alloc_calls_max={} requested_bytes_min={} requested_bytes_median={:.1} requested_bytes_p95={} requested_bytes_max={}",
        calls.0, calls.1, calls.2, calls.3, bytes.0, bytes.1, bytes.2, bytes.3
    );
}

fn samples() -> Vec<(String, Metadata)> {
    let mut result = Vec::new();
    result.push(("empty_0_entries".to_owned(), Metadata::new()));
    for count in [1, 8, 32] {
        let mut scalar = Metadata::new();
        let mut text = Metadata::new();
        for index in 0..count {
            scalar.set(&format!("k{index:02}"), index as i64);
            text.set(&format!("k{index:02}"), "representative metadata text");
        }
        result.push((format!("scalar_{count}_entries"), scalar));
        result.push((format!("text_{count}_entries"), text));
    }
    let mut special = Metadata::new();
    special.set("escaped", "quote=\" slash=\\ newline=\n tab=\t");
    special.set("unicode", "café 東京 🛰");
    special.set("nested", serde_json::json!({"outer": [1, {"inner": true}]}));
    result.push(("escaped_unicode_nested".to_owned(), special));
    let mut near_limit = Metadata::new();
    for index in 0..32 {
        near_limit.set(&format!("f{index:02}"), "x".repeat(480));
    }
    let bytes = serde_json::to_vec(&near_limit).expect("near-limit sample JSON").len();
    result.push((format!("near_16k_{bytes}_bytes"), near_limit));
    let mut over_limit = Metadata::new();
    for index in 0..32 {
        over_limit.set(&format!("f{index:02}"), "x".repeat(510));
    }
    let bytes = serde_json::to_vec(&over_limit).expect("over-limit sample JSON").len();
    result.push((format!("over_16k_{bytes}_bytes"), over_limit));
    result
}
