// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Isolated allocation-request measurements for filter JSON encoding.

use std::alloc::GlobalAlloc;
use std::alloc::Layout;
use std::alloc::System;
use std::hint::black_box;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use qubit_metadata::FilterExpression;
use qubit_metadata::MetadataFilter;
use qubit_metadata::default_json_encode_limits;

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

fn measured<F: Fn() -> Vec<u8>>(encode: F) -> (usize, Distribution) {
    let output = encode();
    let output_bytes = output.len();
    drop(output);

    let mut calls = [0; 100];
    let mut requested_bytes = [0; 100];
    for index in 0..100 {
        CALLS.store(0, Ordering::Relaxed);
        REQUESTED_BYTES.store(0, Ordering::Relaxed);
        COUNTING.store(true, Ordering::SeqCst);
        let output = black_box(encode());
        COUNTING.store(false, Ordering::SeqCst);
        assert_eq!(output.len(), output_bytes, "encoded output size changed");
        drop(output);
        calls[index] = CALLS.load(Ordering::Relaxed);
        requested_bytes[index] = REQUESTED_BYTES.load(Ordering::Relaxed);
    }

    (output_bytes, Distribution { calls, requested_bytes })
}

fn main() {
    // Prove the counting window observes allocator requests before measuring.
    CALLS.store(0, Ordering::Relaxed);
    COUNTING.store(true, Ordering::SeqCst);
    let check = vec![0_u8; 37];
    black_box(check);
    COUNTING.store(false, Ordering::SeqCst);
    assert!(
        CALLS.load(Ordering::Relaxed) > 0,
        "allocation counter self-check failed"
    );

    let samples = filter_encoding_samples();
    for (name, filter) in samples {
        let ordinary = measured(|| serde_json::to_vec(black_box(&filter)).expect("ordinary JSON encoding"));
        let bounded = measured(|| {
            filter
                .to_json_vec_with_limits(default_json_encode_limits())
                .expect("bounded JSON encoding")
        });
        assert_eq!(ordinary.0, bounded.0, "output size differs for {name}");
        let ordinary_calls = Distribution::summary(ordinary.1.calls);
        let ordinary_bytes = Distribution::summary(ordinary.1.requested_bytes);
        let bounded_calls = Distribution::summary(bounded.1.calls);
        let bounded_bytes = Distribution::summary(bounded.1.requested_bytes);
        println!(
            "sample={name} path=serde_json output_bytes={} calls_min={} calls_median={:.1} calls_p95={} calls_max={} requested_bytes_min={} requested_bytes_median={:.1} requested_bytes_p95={} requested_bytes_max={}",
            ordinary.0,
            ordinary_calls.0,
            ordinary_calls.1,
            ordinary_calls.2,
            ordinary_calls.3,
            ordinary_bytes.0,
            ordinary_bytes.1,
            ordinary_bytes.2,
            ordinary_bytes.3
        );
        println!(
            "sample={name} path=bounded output_bytes={} calls_min={} calls_median={:.1} calls_p95={} calls_max={} requested_bytes_min={} requested_bytes_median={:.1} requested_bytes_p95={} requested_bytes_max={}",
            bounded.0,
            bounded_calls.0,
            bounded_calls.1,
            bounded_calls.2,
            bounded_calls.3,
            bounded_bytes.0,
            bounded_bytes.1,
            bounded_bytes.2,
            bounded_bytes.3
        );
    }
}

fn filter_encoding_samples() -> Vec<(String, MetadataFilter)> {
    let leaf = |index: usize| {
        FilterExpression::builder()
            .exists(&format!("field_{index}"))
            .build()
            .expect("benchmark leaf should build")
    };
    let flat = |count: usize, use_or: bool| {
        let mut expressions = (0..count).map(leaf);
        let first = expressions.next().expect("nonempty sample");
        expressions.fold(first, |left, right| {
            if use_or {
                left.try_or(right).expect("bounded sample OR")
            } else {
                left.try_and(right).expect("bounded sample AND")
            }
        })
    };
    let into_filter = |name: String, expression| {
        (
            name,
            MetadataFilter::builder()
                .expression(expression)
                .build()
                .expect("benchmark filter should build"),
        )
    };

    let mut samples = Vec::new();
    samples.push(into_filter("small_1".to_owned(), leaf(0)));
    samples.push(into_filter("small_8".to_owned(), flat(8, false)));
    // A flat group counts its operator node, so 255 leaves is the largest
    // representable sample under the 256-node hard limit.
    for count in [64, 255] {
        samples.push(into_filter(format!("flat_and_{count}"), flat(count, false)));
        samples.push(into_filter(format!("flat_or_{count}"), flat(count, true)));
    }
    for depth in [4, 16] {
        let mut expression = leaf(depth);
        for level in 0..depth {
            expression = match level % 3 {
                0 => expression.try_not().expect("bounded nested NOT"),
                1 => expression.try_and(leaf(depth + level + 1)).expect("bounded nested AND"),
                _ => expression.try_or(leaf(depth + level + 1)).expect("bounded nested OR"),
            };
        }
        samples.push(into_filter(format!("nested_{depth}"), expression));
    }
    // The public builder caps membership operands at 128 values.
    for count in [8, 64, 128] {
        samples.push(into_filter(
            format!("membership_{count}"),
            FilterExpression::builder()
                .in_set("member", 0_i64..count as i64)
                .build()
                .expect("bounded membership sample"),
        ));
    }
    samples
}
