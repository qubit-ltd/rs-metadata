// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Criterion benchmarks for common metadata construction and matching paths.

use std::hint::black_box;

use criterion::BatchSize;
use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
#[cfg(feature = "json")]
use qubit_budget::ResourceLimit;
#[cfg(feature = "json")]
use qubit_budget::json::JsonResource;
use qubit_metadata::FilterExpression;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataFilter;
#[cfg(all(feature = "json", feature = "schema"))]
use qubit_metadata::MetadataLimits;
#[cfg(all(feature = "json", feature = "schema"))]
use qubit_metadata::MetadataSchema;
#[cfg(feature = "json")]
use qubit_metadata::MetadataWireEncodeError;
#[cfg(feature = "json")]
use qubit_metadata::default_json_encode_limits;

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

/// Benchmarks bounded JSON decoding of a prepared metadata envelope.
#[cfg(feature = "json")]
fn benchmark_metadata_json_decode(criterion: &mut Criterion) {
    let metadata = Metadata::new()
        .with("tenant", "acme")
        .with("score", 42_i64)
        .with("active", true);
    let encoded = serde_json::to_vec(&metadata).expect("benchmark metadata should serialize");

    criterion.bench_function("metadata/json_decode", |bencher| {
        bencher.iter(|| {
            let decoded = Metadata::decode_json_slice(black_box(&encoded)).expect("benchmark metadata should decode");
            black_box(decoded)
        });
    });
}

#[cfg(not(feature = "json"))]
fn benchmark_metadata_json_decode(_criterion: &mut Criterion) {}

/// Measures bounded encoding and output-budget rejection for request metadata.
#[cfg(feature = "json")]
fn benchmark_metadata_json_encode(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("metadata/json_encode");
    for count in [8, 64, 256] {
        let mut metadata = Metadata::new();
        for index in 0..count {
            metadata.set(&format!("field_{index:03}"), "representative request metadata value");
        }
        let success_limits = default_json_encode_limits();
        let rejection_limits = success_limits
            .into_builder()
            .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 1))
            .build();
        metadata
            .to_json_vec_with_limits(success_limits)
            .expect("benchmark metadata should encode");
        assert!(matches!(
            metadata.to_json_vec_with_limits(rejection_limits),
            Err(MetadataWireEncodeError::Budget(ref source))
                if *source.resource() == JsonResource::OutputBytes
        ));
        group.bench_function(BenchmarkId::new("success", count), |bencher| {
            bencher.iter(|| {
                black_box(
                    black_box(&metadata)
                        .to_json_vec_with_limits(success_limits)
                        .expect("benchmark metadata should encode"),
                )
            });
        });
        group.bench_function(BenchmarkId::new("output_limit_rejection", count), |bencher| {
            bencher.iter(|| black_box(black_box(&metadata).to_json_vec_with_limits(rejection_limits)));
        });
    }
    group.finish();
}

/// Investigates local costs represented by downstream request validation and
/// persistence.
#[cfg(feature = "json")]
fn benchmark_request_encode_investigation(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("metadata/request_encode_investigation");
    let mut samples = Vec::new();
    for count in [1, 8, 32] {
        let mut metadata = Metadata::new();
        for index in 0..count {
            metadata.set(&format!("k{index:02}"), "v");
        }
        samples.push((format!("short_{count}_entries"), metadata));
    }

    let mut near_limit = Metadata::new();
    for index in 0..32 {
        near_limit.set(&format!("field_{index:02}"), "x".repeat(470));
    }
    samples.push(("near_16k_32_entries".to_owned(), near_limit));

    let limits = default_json_encode_limits()
        .into_builder()
        .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 16 * 1024))
        .build();

    for (sample_name, metadata) in samples {
        let encoded = metadata
            .to_json_vec_with_limits(limits)
            .expect("benchmark metadata sample should encode within the request limit");
        let encoded_len = encoded.len();
        assert!(encoded_len <= 16 * 1024, "sample exceeds the request byte limit");
        drop(encoded);

        group.bench_function(BenchmarkId::new("encode_and_drop", &sample_name), |bencher| {
            bencher.iter_batched(
                || metadata.clone(),
                |metadata| {
                    let encoded = metadata
                        .to_json_vec_with_limits(limits)
                        .expect("benchmark metadata should encode");
                    black_box(encoded.len());
                    drop(encoded);
                },
                BatchSize::SmallInput,
            );
        });
        group.bench_function(BenchmarkId::new("clone", &sample_name), |bencher| {
            bencher.iter_batched(
                || metadata.clone(),
                |metadata| black_box(metadata.clone()),
                BatchSize::SmallInput,
            );
        });
        group.bench_function(BenchmarkId::new("serde_reencode", &sample_name), |bencher| {
            bencher.iter_batched(
                || metadata.clone(),
                |metadata| {
                    let encoded = serde_json::to_vec(&metadata).expect("benchmark metadata should serialize");
                    black_box(encoded.len());
                    drop(encoded);
                },
                BatchSize::SmallInput,
            );
        });
        group.bench_function(BenchmarkId::new("validation_clone_reencode", &sample_name), |bencher| {
            bencher.iter_batched(
                || metadata.clone(),
                |metadata| {
                    let validated = metadata
                        .to_json_vec_with_limits(limits)
                        .expect("benchmark metadata should pass the request limit");
                    black_box(validated.len());
                    drop(validated);
                    let metadata_for_storage = metadata.clone();
                    let persisted =
                        serde_json::to_vec(&metadata_for_storage).expect("benchmark metadata should serialize");
                    black_box(persisted.len());
                    drop(persisted);
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

#[cfg(not(feature = "json"))]
fn benchmark_request_encode_investigation(_criterion: &mut Criterion) {}

#[cfg(not(feature = "json"))]
fn benchmark_metadata_json_encode(_criterion: &mut Criterion) {}

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

/// Benchmarks incremental construction of flattened AND expressions.
fn benchmark_filter_expression_construction(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("metadata/filter_expression_construction");
    for condition_count in [16, 64, 128, 255] {
        let keys: Vec<_> = (0..condition_count).map(|index| format!("key_{index}")).collect();
        group.bench_with_input(
            BenchmarkId::from_parameter(condition_count),
            keys.as_slice(),
            |bencher, keys| {
                bencher.iter(|| {
                    let result = keys
                        .iter()
                        .fold(FilterExpression::builder(), |builder, key| builder.exists(key))
                        .build();
                    black_box(result.expect("benchmark filter should build"))
                });
            },
        );
    }
    group.finish();
}

/// Benchmarks public expression composition separately from builder
/// construction.
fn benchmark_public_expression_composition(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("metadata/public_expression_composition");
    for condition_count in [16, 64, 128, 255] {
        let leaves: Vec<_> = (0..condition_count)
            .map(|index| {
                FilterExpression::builder()
                    .exists(&format!("key_{index}"))
                    .build()
                    .expect("benchmark leaf should build")
            })
            .collect();
        for operator in ["and", "or"] {
            group.bench_function(BenchmarkId::new(operator, condition_count), |bencher| {
                bencher.iter_batched(
                    || leaves.clone(),
                    |leaves| {
                        let mut items = leaves.into_iter();
                        let first = items.next().expect("nonempty benchmark input");
                        let expression = items.fold(first, |left, right| {
                            if operator == "and" {
                                left.try_and(right).expect("bounded public AND composition")
                            } else {
                                left.try_or(right).expect("bounded public OR composition")
                            }
                        });
                        black_box(expression)
                    },
                    BatchSize::SmallInput,
                );
            });
        }

        // A 255-condition AND already uses all 256 nodes, so NOT must stop at 254.
        let not_count = condition_count.min(254);
        let mut not_items = leaves.iter().take(not_count).cloned();
        let not_first = not_items.next().expect("nonempty benchmark input");
        let not_input = not_items.fold(not_first, |left, right| {
            left.try_and(right).expect("bounded public AND composition")
        });
        group.bench_function(BenchmarkId::new("not", not_count), |bencher| {
            bencher.iter_batched(
                || not_input.clone(),
                |expression| black_box(expression.try_not().expect("bounded public NOT composition")),
                BatchSize::SmallInput,
            );
        });
        group.bench_function(BenchmarkId::new("double_not", not_count), |bencher| {
            bencher.iter_batched(
                || not_input.clone(),
                |expression| {
                    black_box(
                        expression
                            .try_not()
                            .expect("bounded first public NOT composition")
                            .try_not()
                            .expect("bounded second public NOT composition"),
                    )
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

/// Benchmarks bounded JSON decoding of a prepared metadata filter envelope.
#[cfg(feature = "json")]
fn benchmark_filter_json_decode(criterion: &mut Criterion) {
    let expression = FilterExpression::builder()
        .eq("tenant", "acme")
        .ge("score", 10_i64)
        .exists("active")
        .build()
        .expect("benchmark filter expression should build");
    let filter = MetadataFilter::builder()
        .expression(expression)
        .build()
        .expect("benchmark filter should build");
    let encoded = serde_json::to_vec(&filter).expect("benchmark filter should serialize");

    criterion.bench_function("metadata/filter_json_decode", |bencher| {
        bencher.iter(|| {
            let decoded =
                MetadataFilter::decode_json_slice(black_box(&encoded)).expect("benchmark filter should decode");
            black_box(decoded)
        });
    });
}

#[cfg(not(feature = "json"))]
fn benchmark_filter_json_decode(_criterion: &mut Criterion) {}

/// Benchmarks bounded JSON decoding of a prepared metadata schema envelope.
#[cfg(all(feature = "json", feature = "schema"))]
fn benchmark_schema_json_decode(criterion: &mut Criterion) {
    let schema = MetadataSchema::builder()
        .required("tenant", qubit_datatype::DataType::String)
        .optional("score", qubit_datatype::DataType::Int64)
        .build()
        .expect("benchmark schema should build");
    let encoded = serde_json::to_vec(&schema).expect("benchmark schema should serialize");

    criterion.bench_function("metadata/schema_json_decode", |bencher| {
        bencher.iter(|| {
            let decoded = MetadataSchema::decode_json_slice_with_limits(black_box(&encoded), MetadataLimits::default())
                .expect("benchmark schema should decode");
            black_box(decoded)
        });
    });
}

#[cfg(not(all(feature = "json", feature = "schema")))]
fn benchmark_schema_json_decode(_criterion: &mut Criterion) {}

/// Measures request cloning, merging, and borrowed versus converting reads.
fn benchmark_downstream_metadata_paths(criterion: &mut Criterion) {
    for count in [8, 64, 256] {
        let mut metadata = Metadata::new();
        for index in 0..count {
            metadata.set(&format!("field_{index:03}"), "representative request metadata value");
        }
        criterion.bench_function(&format!("metadata/clone_{count}_fields"), |bencher| {
            bencher.iter(|| black_box(black_box(&metadata).clone()));
        });
        criterion.bench_function(&format!("metadata/merge_{count}_fields"), |bencher| {
            bencher.iter_batched(
                || (metadata.clone(), metadata.clone()),
                |(mut left, right)| {
                    left.merge(right);
                    black_box(left)
                },
                BatchSize::SmallInput,
            );
        });
    }
    let metadata = Metadata::new().with("model", "embedding-model");
    criterion.bench_function("metadata/borrowed_string", |bencher| {
        bencher.iter(|| black_box(metadata.get_ref::<str>(black_box("model"))));
    });
    criterion.bench_function("metadata/converted_string", |bencher| {
        bencher.iter(|| black_box(metadata.convert::<String>(black_box("model"))));
    });
    let records: Vec<_> = (0..1000_i64)
        .map(|score| Metadata::new().with("score", score))
        .collect();
    let filter = MetadataFilter::builder()
        .expression(
            FilterExpression::builder()
                .ge("score", 500_i64)
                .build()
                .expect("benchmark expression"),
        )
        .build()
        .expect("benchmark filter");
    criterion.bench_function("metadata/filter_1000_records", |bencher| {
        bencher.iter(|| {
            black_box(
                records
                    .iter()
                    .filter(|metadata| filter.matches(black_box(metadata)))
                    .count(),
            )
        });
    });
}

criterion_group!(
    benches,
    benchmark_metadata_construction,
    benchmark_downstream_metadata_paths,
    benchmark_metadata_typed_get,
    benchmark_metadata_json_decode,
    benchmark_metadata_json_encode,
    benchmark_request_encode_investigation,
    benchmark_filter_expression_construction,
    benchmark_public_expression_composition,
    benchmark_filter_match,
    benchmark_filter_json_decode,
    benchmark_schema_json_decode,
);
criterion_main!(benches);
