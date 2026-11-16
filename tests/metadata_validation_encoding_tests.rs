// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
#![cfg(feature = "json")]

use qubit_budget::BudgetError;
use qubit_budget::Observation;
use qubit_budget::QuantityConversionError;
use qubit_budget::QuantityMeasurement;
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

#[derive(Debug, PartialEq, Eq)]
enum Signature {
    Success,
    Budget(BudgetError<JsonResource, usize>),
    Quantity(JsonResource, QuantityConversionError),
    Wire(MetadataWireLimitKind, usize, usize),
    Domain,
    Syntax,
    Json(JsonSerializationErrorKind),
    Io,
}

#[derive(Debug)]
struct EncodingSignatures {
    encoded: Signature,
    writer: Signature,
    validation: Signature,
}

fn signature(result: Result<(), MetadataWireEncodeError>) -> Signature {
    match result {
        Ok(()) => Signature::Success,
        Err(MetadataWireEncodeError::Budget(error)) => Signature::Budget(error),
        Err(MetadataWireEncodeError::Quantity { resource, source }) => Signature::Quantity(resource, source),
        Err(MetadataWireEncodeError::Domain(MetadataError::WireLimitExceeded { kind, value, maximum })) => {
            Signature::Wire(kind, value, maximum)
        }
        Err(MetadataWireEncodeError::Domain(_)) => Signature::Domain,
        Err(MetadataWireEncodeError::Syntax(_)) => Signature::Syntax,
        Err(MetadataWireEncodeError::Json(error)) => Signature::Json(error.kind()),
        Err(MetadataWireEncodeError::Io(_)) => Signature::Io,
        Err(_) => panic!("unexpected metadata encoding error"),
    }
}

#[test]
fn test_signature_compares_every_budget_and_quantity_field() {
    let exact = Signature::Budget(BudgetError::LimitExceeded {
        resource: JsonResource::StringBytes,
        observed: Observation::Exact(4),
        maximum: 3,
    });
    assert_ne!(
        exact,
        Signature::Budget(BudgetError::LimitExceeded {
            resource: JsonResource::StringBytes,
            observed: Observation::Exact(5),
            maximum: 3,
        })
    );
    assert_ne!(
        exact,
        Signature::Budget(BudgetError::LimitExceeded {
            resource: JsonResource::StringBytes,
            observed: Observation::Exact(4),
            maximum: 2,
        })
    );
    assert_ne!(
        Signature::Budget(BudgetError::LimitExceeded {
            resource: JsonResource::StringBytes,
            observed: Observation::Exact(4),
            maximum: 3,
        }),
        Signature::Budget(BudgetError::LimitExceeded {
            resource: JsonResource::StringBytes,
            observed: Observation::AtLeast(4),
            maximum: 3,
        })
    );

    let insufficient = Signature::Budget(BudgetError::Insufficient {
        resource: JsonResource::OutputBytes,
        limit: 10,
        remaining: 3,
        requested: 4,
    });
    for (limit, remaining, requested) in [(9, 3, 4), (10, 2, 4), (10, 3, 5)] {
        assert_ne!(
            insufficient,
            Signature::Budget(BudgetError::Insufficient {
                resource: JsonResource::OutputBytes,
                limit,
                remaining,
                requested,
            })
        );
    }

    let quantity = Signature::Quantity(
        JsonResource::Nodes,
        QuantityConversionError::new(QuantityMeasurement::Usize(usize::MAX), "u8"),
    );
    assert_ne!(
        quantity,
        Signature::Quantity(
            JsonResource::Nodes,
            QuantityConversionError::new(QuantityMeasurement::Usize(usize::MAX - 1), "u8"),
        )
    );
    assert_ne!(
        quantity,
        Signature::Quantity(
            JsonResource::Nodes,
            QuantityConversionError::new(QuantityMeasurement::Usize(usize::MAX), "u16"),
        )
    );
}

fn assert_equivalent(metadata: &Metadata, limits: JsonEncodeLimits) -> EncodingSignatures {
    let encoded = metadata.to_json_vec_with_limits(limits);
    let mut written = Vec::new();
    let writer_result = metadata.to_json_writer_with_limits(&mut written, limits);
    if let Ok(bytes) = &encoded {
        assert!(writer_result.is_ok());
        assert_eq!(&written, bytes);
    } else {
        assert!(written.is_empty());
    }
    let encoded = signature(encoded.map(drop));
    let writer = signature(writer_result);
    let validation = signature(metadata.validate_json_with_limits(limits));
    assert_eq!(writer, encoded);
    if is_output_budget_error(&encoded) {
        assert!(is_output_budget_error(&validation));
    } else {
        assert_eq!(validation, encoded);
    }
    EncodingSignatures {
        encoded,
        writer,
        validation,
    }
}

fn is_output_budget_error(signature: &Signature) -> bool {
    matches!(
        signature,
        Signature::Budget(BudgetError::Insufficient {
            resource: JsonResource::OutputBytes,
            ..
        })
    )
}

#[test]
fn test_validation_only_matches_structured_errors_and_validation_order() {
    let empty = Metadata::new();
    let ordinary = Metadata::new().with("integer", 7_i64).with("enabled", true);
    let mut special = Metadata::new();
    special.set("escaped", "quote=\" slash=\\ newline=\n tab=\t");
    special.set("unicode", "café 東京 🛰");
    special.set("nested", serde_json::json!({"outer": [1, {"inner": true}]}));
    let non_finite = Metadata::new().with("nan", Value::Float64(f64::NAN));
    for metadata in [&empty, &ordinary, &special, &non_finite] {
        assert_equivalent(metadata, default_json_encode_limits());
    }

    let mut too_many = Metadata::new();
    for index in 0..=4096 {
        too_many.set(&format!("key_{index:04}"), index as i64);
    }
    assert_eq!(
        assert_equivalent(&too_many, default_json_encode_limits()).encoded,
        Signature::Wire(MetadataWireLimitKind::Entries, 4097, 4096)
    );
    let long_key = "k".repeat(257);
    let long_key_metadata = Metadata::new().with(&long_key, 1_i64);
    assert_eq!(
        assert_equivalent(&long_key_metadata, default_json_encode_limits()).encoded,
        Signature::Wire(MetadataWireLimitKind::KeyBytes, 257, 256)
    );

    let value_budget_cases = [
        (
            serde_json::json!({"nested": true}),
            JsonValueLimits::builder().max_depth(1).build(),
            BudgetError::LimitExceeded {
                resource: JsonResource::Depth,
                observed: Observation::Exact(2),
                maximum: 1,
            },
        ),
        (
            serde_json::json!(true),
            JsonValueLimits::builder().max_nodes(0).build(),
            BudgetError::LimitExceeded {
                resource: JsonResource::Nodes,
                observed: Observation::AtLeast(1),
                maximum: 0,
            },
        ),
        (
            serde_json::json!([1]),
            JsonValueLimits::builder().max_sequence_items(0).build(),
            BudgetError::LimitExceeded {
                resource: JsonResource::SequenceItems,
                observed: Observation::Exact(1),
                maximum: 0,
            },
        ),
        (
            serde_json::json!({"a": 1}),
            JsonValueLimits::builder().max_map_entries(0).build(),
            BudgetError::LimitExceeded {
                resource: JsonResource::MapEntries,
                observed: Observation::Exact(1),
                maximum: 0,
            },
        ),
        (
            serde_json::json!({"a": 1}),
            JsonValueLimits::builder().max_key_bytes(0).build(),
            BudgetError::LimitExceeded {
                resource: JsonResource::KeyBytes,
                observed: Observation::Exact(1),
                maximum: 0,
            },
        ),
        (
            serde_json::json!("long"),
            JsonValueLimits::builder().max_string_bytes(2).build(),
            BudgetError::LimitExceeded {
                resource: JsonResource::StringBytes,
                observed: Observation::Exact(4),
                maximum: 2,
            },
        ),
        (
            serde_json::json!(123),
            JsonValueLimits::builder().max_number_bytes(2).build(),
            BudgetError::LimitExceeded {
                resource: JsonResource::NumberBytes,
                observed: Observation::Exact(3),
                maximum: 2,
            },
        ),
        (
            serde_json::json!({"a": "b"}),
            JsonValueLimits::builder().max_payload_bytes(1).build(),
            BudgetError::LimitExceeded {
                resource: JsonResource::PayloadBytes,
                observed: Observation::AtLeast(2),
                maximum: 1,
            },
        ),
    ];
    for (json, value_limits, expected_error) in value_budget_cases {
        let metadata = Metadata::new().with("json", Value::Json(json));
        let limits = default_json_encode_limits()
            .into_builder()
            .value_limits(value_limits)
            .build();
        let actual = assert_equivalent(&metadata, limits);
        assert_eq!(actual.encoded, Signature::Budget(expected_error));
    }

    // Each JSON value contributes two payload bytes, below the three-byte
    // limit on its own; the shared preflight must reject their four-byte sum.
    let metadata = Metadata::new()
        .with("first", Value::Json(serde_json::json!("aa")))
        .with("second", Value::Json(serde_json::json!("bb")));
    let limits = default_json_encode_limits()
        .into_builder()
        .value_limits(JsonValueLimits::builder().max_payload_bytes(3).build())
        .build();
    let actual = assert_equivalent(&metadata, limits);
    let expected = Signature::Budget(BudgetError::LimitExceeded {
        resource: JsonResource::PayloadBytes,
        observed: Observation::AtLeast(4),
        maximum: 3,
    });
    assert_eq!(&actual.encoded, &expected);
    assert_eq!(&actual.writer, &expected);
    assert_eq!(&actual.validation, &expected);

    let boundary = Metadata::new().with("edge", "quote=\" café");
    let encoded_len = serde_json::to_vec(&boundary).expect("serialize boundary").len();
    for (cap, expect_success) in [(encoded_len - 1, false), (encoded_len, true), (encoded_len + 1, true)] {
        let limits = default_json_encode_limits()
            .into_builder()
            .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, cap))
            .build();
        let actual = assert_equivalent(&boundary, limits);
        if !expect_success {
            let cap = encoded_len - 1;
            let buffered = Signature::Budget(BudgetError::Insufficient {
                resource: JsonResource::OutputBytes,
                limit: cap,
                remaining: cap,
                requested: encoded_len,
            });
            assert_eq!(actual.encoded, buffered);
            assert_eq!(actual.writer, buffered);
            assert_eq!(
                actual.validation,
                Signature::Budget(BudgetError::Insufficient {
                    resource: JsonResource::OutputBytes,
                    limit: cap,
                    remaining: 0,
                    requested: 1,
                })
            );
        } else {
            assert_eq!(actual.encoded, Signature::Success);
            assert_eq!(actual.writer, Signature::Success);
            assert_eq!(actual.validation, Signature::Success);
        }
    }

    // Wire contract is checked before value budgets.
    let mut wire_and_value = Metadata::new().with("json", Value::Json(serde_json::json!("too long")));
    for index in 0..4096 {
        wire_and_value.set(&format!("key_{index:04}"), "x");
    }
    let limits = default_json_encode_limits()
        .into_builder()
        .value_limits(JsonValueLimits::builder().max_string_bytes(1).build())
        .build();
    assert_eq!(
        assert_equivalent(&wire_and_value, limits).encoded,
        Signature::Wire(MetadataWireLimitKind::Entries, 4097, 4096)
    );

    // Value preflight precedes output accounting.
    let metadata = Metadata::new().with("json", Value::Json(serde_json::json!("too long")));
    let limits = default_json_encode_limits()
        .into_builder()
        .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 1))
        .value_limits(JsonValueLimits::builder().max_string_bytes(1).build())
        .build();
    let actual = assert_equivalent(&metadata, limits);
    assert_eq!(
        actual.encoded,
        Signature::Budget(BudgetError::LimitExceeded {
            resource: JsonResource::StringBytes,
            observed: Observation::Exact(8),
            maximum: 1,
        })
    );

    // Incremental output preserves serializer/output-budget precedence by key
    // order.
    let mut output_then_serialize = Metadata::new();
    output_then_serialize.set("a_large", "x".repeat(512));
    output_then_serialize.set("z_nan", Value::Float64(f64::NAN));
    let mut serialize_then_output = Metadata::new();
    serialize_then_output.set("a_nan", Value::Float64(f64::NAN));
    serialize_then_output.set("z_large", "x".repeat(512));
    let limits = default_json_encode_limits()
        .into_builder()
        .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 128))
        .build();
    assert!(matches!(
        assert_equivalent(&output_then_serialize, limits).encoded,
        Signature::Budget(BudgetError::Insufficient { .. })
    ));
    assert_eq!(
        assert_equivalent(&serialize_then_output, limits).encoded,
        Signature::Json(JsonSerializationErrorKind::CustomSerialization)
    );
}
