// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Regression coverage for compatibility contracts across metadata releases.

#![cfg(all(feature = "schema", feature = "json"))]

use qubit_datatype::DataType;
use qubit_metadata::FilterMatchOptions;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataError;
use qubit_metadata::MetadataField;
use qubit_metadata::MetadataFilter;
use qubit_metadata::MetadataSchema;
use qubit_metadata::MetadataWireDecodeError;
use qubit_metadata::MetadataWireEncodeError;
use qubit_metadata::MetadataWireLimitKind;
use serde_json::Value;

fn fixture(case: &str) -> Vec<u8> {
    std::fs::read(format!("tests/fixtures/compatibility/{case}.json"))
        .expect("compatibility fixture exists")
}

fn normalized(case: &str) -> Vec<u8> {
    std::fs::read(format!(
        "tests/fixtures/compatibility/normalized/normalized-{case}.json"
    ))
    .expect("normalized output captured from the 0.6.2 consumer exists")
}

fn canonical(value: impl serde::Serialize) -> Vec<u8> {
    serde_json::to_vec(&value).expect("canonical value serializes")
}

#[test]
fn test_rejects_positional_schema_forms_that_0_6_2_accepted() {
    let object = fixture("schema_object");
    let current =
        MetadataSchema::decode_json_slice(&object).expect("both releases accept schema objects");
    assert_eq!(
        serde_json::from_slice::<Value>(&canonical(current)).unwrap(),
        serde_json::from_slice::<Value>(&object).unwrap()
    );

    let field_sequence = fixture("schema_field_sequence");
    assert!(matches!(
        MetadataSchema::decode_json_slice(&field_sequence),
        Err(MetadataWireDecodeError::Decode(_))
    ));
    let old_normalized = normalized("schema_field_sequence");
    let migrated = MetadataSchema::decode_json_slice(&old_normalized)
        .expect("0.6.2-normalized schema reads in current version");
    let fields: Vec<_> = migrated.fields().collect();
    assert_eq!(fields.len(), 1);
    let (name, field) = fields[0];
    assert_eq!(name, "name");
    assert_eq!(field.data_type(), DataType::String);
    assert!(field.is_required());
    let migrated_wire: Value = serde_json::from_slice(&canonical(migrated)).unwrap();
    let old_normalized_wire: Value = serde_json::from_slice(&old_normalized).unwrap();
    assert_eq!(migrated_wire, old_normalized_wire);

    let envelope_sequence = fixture("schema_envelope_sequence");
    assert!(matches!(
        MetadataSchema::decode_json_slice(&envelope_sequence),
        Err(MetadataWireDecodeError::Decode(_))
    ));
}

#[test]
fn test_preserves_numeric_policies_and_rejects_positional_options() {
    let object = fixture("options_object");
    let options: FilterMatchOptions =
        serde_json::from_slice(&object).expect("options object is accepted");
    assert_eq!(
        options.numeric_comparison_policy(),
        qubit_datatype::NumericComparisonPolicy::Approximate
    );
    let exact_object = fixture("options_exact_object");
    let exact: FilterMatchOptions = serde_json::from_slice(&exact_object).unwrap();
    assert_eq!(
        exact.numeric_comparison_policy(),
        qubit_datatype::NumericComparisonPolicy::Exact
    );
    let exact_normalized = normalized("options_exact_object");
    assert_eq!(
        serde_json::from_slice::<Value>(&exact_object).unwrap(),
        serde_json::from_slice::<Value>(&exact_normalized).unwrap()
    );

    let positional = fixture("options_sequence");
    assert!(serde_json::from_slice::<FilterMatchOptions>(&positional).is_err());

    let positional_filter = fixture("filter_options_sequence");
    assert!(matches!(
        MetadataFilter::decode_json_slice(&positional_filter),
        Err(MetadataWireDecodeError::Decode(_))
    ));

    // The 0.6.2 normalized form retained `approximate`; current callers must
    // preserve that policy while converting it to the canonical object form.
    let normalized_options_bytes = normalized("options_sequence");
    let normalized_options: FilterMatchOptions = serde_json::from_slice(&normalized_options_bytes)
        .expect("normalized old options read in current version");
    assert_eq!(
        normalized_options.numeric_comparison_policy(),
        qubit_datatype::NumericComparisonPolicy::Approximate
    );
    let normalized_filter = normalized("filter_options_sequence");
    let current_filter = MetadataFilter::decode_json_slice(&normalized_filter)
        .expect("normalized old filter reads in current version");
    assert_eq!(
        serde_json::from_slice::<Value>(&canonical(current_filter)).unwrap(),
        serde_json::from_slice::<Value>(&normalized_filter).unwrap()
    );
}

#[test]
fn test_classifies_shared_malformed_inputs() {
    for case in ["empty", "invalid_utf8", "wrong_top_level", "invalid_syntax"] {
        let bytes = fixture(case);
        let expected = match case {
            "empty" | "invalid_syntax" => "syntax",
            "invalid_utf8" | "wrong_top_level" => "decode",
            _ => unreachable!(),
        };
        let metadata = Metadata::decode_json_slice(&bytes).unwrap_err();
        let schema = MetadataSchema::decode_json_slice(&bytes).unwrap_err();
        let filter = MetadataFilter::decode_json_slice(&bytes).unwrap_err();
        for (api, error) in [
            ("metadata", metadata),
            ("schema", schema),
            ("filter", filter),
        ] {
            let matches = match expected {
                "syntax" => matches!(&error, MetadataWireDecodeError::Syntax(_)),
                "decode" => matches!(&error, MetadataWireDecodeError::Decode(_)),
                _ => false,
            };
            assert!(
                matches,
                "{case} should classify as {expected} for {api}: {error:?}"
            );
        }
    }
}

#[test]
fn test_reports_structured_bounded_encode_limits() {
    let mut metadata = Metadata::new();
    for index in 0..4097 {
        let key = format!("key-{index}");
        metadata = metadata.with(&key, index as i64);
    }
    assert!(matches!(
        metadata.to_json_vec(),
        Err(MetadataWireEncodeError::Domain(
            MetadataError::WireLimitExceeded {
                kind: MetadataWireLimitKind::Entries,
                value: 4097,
                maximum: 4096
            }
        ))
    ));

    let long_key = "k".repeat(257);
    let metadata = Metadata::new().with(&long_key, "value");
    assert!(matches!(
        metadata.to_json_vec(),
        Err(MetadataWireEncodeError::Domain(
            MetadataError::WireLimitExceeded {
                kind: MetadataWireLimitKind::KeyBytes,
                value: 257,
                maximum: 256
            }
        ))
    ));

    let mut builder = MetadataSchema::builder();
    for index in 0..4097 {
        builder = builder.replace_field(
            &format!("key-{index}"),
            MetadataField::new(DataType::String, false),
        );
    }
    let schema = builder.build().unwrap();
    assert!(matches!(
        schema.to_json_vec(),
        Err(MetadataWireEncodeError::Domain(
            MetadataError::WireLimitExceeded {
                kind: MetadataWireLimitKind::Entries,
                value: 4097,
                maximum: 4096
            }
        ))
    ));

    let long_schema = MetadataSchema::builder()
        .replace_field(&long_key, MetadataField::new(DataType::String, false))
        .build()
        .unwrap();
    assert!(matches!(
        long_schema.to_json_vec(),
        Err(MetadataWireEncodeError::Domain(
            MetadataError::WireLimitExceeded {
                kind: MetadataWireLimitKind::KeyBytes,
                value: 257,
                maximum: 256
            }
        ))
    ));
}
