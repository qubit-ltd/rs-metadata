// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Criterion benchmarks for common metadata construction and matching paths.

use std::hint::black_box;

use criterion::{
    Criterion,
    criterion_group,
    criterion_main,
};
use qubit_metadata::{
    FilterExpression,
    Metadata,
    MetadataFilter,
};

/// Benchmarks constructing a small metadata object through the fluent API.
fn benchmark_metadata_construction(criterion: &mut Criterion) {
    criterion.bench_function("metadata/construction_16_fields", |bencher| {
        bencher.iter(|| {
            let metadata = Metadata::new()
                .with("field_00", 0_i64)
                .with("field_01", 1_i64)
                .with("field_02", 2_i64)
                .with("field_03", 3_i64)
                .with("field_04", 4_i64)
                .with("field_05", 5_i64)
                .with("field_06", 6_i64)
                .with("field_07", 7_i64)
                .with("field_08", 8_i64)
                .with("field_09", 9_i64)
                .with("field_10", 10_i64)
                .with("field_11", 11_i64)
                .with("field_12", 12_i64)
                .with("field_13", 13_i64)
                .with("field_14", 14_i64)
                .with("field_15", 15_i64);
            black_box(metadata)
        });
    });
}

/// Benchmarks typed lookup from a prepared metadata object.
fn benchmark_metadata_typed_get(criterion: &mut Criterion) {
    let metadata = Metadata::new()
        .with("tenant", "acme")
        .with("score", 42_i64)
        .with("active", true);

    criterion.bench_function("metadata/typed_get", |bencher| {
        bencher.iter(|| black_box(metadata.get::<i64>(black_box("score"))));
    });
}

/// Benchmarks evaluating a prepared four-condition metadata filter.
fn benchmark_filter_match(criterion: &mut Criterion) {
    let expression = FilterExpression::builder()
        .eq("tenant", "acme")
        .ge("score", 10_i64)
        .exists("active")
        .not_exists("deleted_at")
        .build()
        .expect("benchmark filter expression should build");
    let filter = MetadataFilter::builder()
        .expression(expression)
        .build()
        .expect("benchmark filter should build");
    let metadata = Metadata::new()
        .with("tenant", "acme")
        .with("score", 42_i64)
        .with("active", true);

    criterion.bench_function("metadata/filter_match", |bencher| {
        bencher.iter(|| black_box(filter.matches(black_box(&metadata))));
    });
}

criterion_group!(
    benches,
    benchmark_metadata_construction,
    benchmark_metadata_typed_get,
    benchmark_filter_match,
);
criterion_main!(benches);
