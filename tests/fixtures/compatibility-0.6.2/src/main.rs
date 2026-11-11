// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Probe the published 0.6.2 compatibility behavior using shared input bytes.

use std::fs;
use std::path::Path;

use qubit_metadata::Metadata;
use qubit_metadata::MetadataFilter;
use qubit_metadata::MetadataSchema;

fn report<T: std::fmt::Debug + serde::Serialize>(
    root: &Path,
    case: &str,
    api: &str,
    result: Result<T, impl std::fmt::Debug>,
) {
    match result {
        Ok(value) => {
            let normalized = serde_json::to_string(&value).unwrap();
            fs::write(
                root.join("normalized").join(format!("normalized-{case}.json")),
                &normalized,
            )
            .unwrap();
            println!("{case}\t0.6.2\t{api}\taccepted\t{value:?}\tnormalized={normalized}");
        }
        Err(error) => println!("{case}\t0.6.2\t{api}\trejected\t{error:?}"),
    }
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("compatibility");
    let normalized_dir = root.join("normalized");
    if normalized_dir.exists() {
        fs::remove_dir_all(&normalized_dir).unwrap();
    }
    fs::create_dir_all(normalized_dir).unwrap();
    let cases = [
        "schema_object",
        "schema_envelope_sequence",
        "schema_field_sequence",
        "options_object",
        "options_exact_object",
        "options_sequence",
        "filter_options_sequence",
        "empty",
        "invalid_utf8",
        "wrong_top_level",
        "invalid_syntax",
    ];
    for case in cases {
        let bytes = fs::read(root.join(format!("{case}.json"))).unwrap();
        report(
            &root,
            case,
            "MetadataSchema::decode_json_slice",
            MetadataSchema::decode_json_slice(&bytes),
        );
        report(
            &root,
            case,
            "MetadataFilter::decode_json_slice",
            MetadataFilter::decode_json_slice(&bytes),
        );
        report(
            &root,
            case,
            "Metadata::decode_json_slice",
            Metadata::decode_json_slice(&bytes),
        );
        report(
            &root,
            case,
            "serde_json::from_slice<FilterMatchOptions>",
            serde_json::from_slice::<qubit_metadata::FilterMatchOptions>(&bytes),
        );
    }

    let mut metadata = Metadata::new();
    for index in 0..4097 {
        let key = format!("key-{index}");
        metadata = metadata.with(&key, index as i64);
    }
    report(
        &root,
        "encode_metadata_entries_4097",
        "Metadata::to_json_vec",
        metadata.to_json_vec(),
    );
    let key = "k".repeat(257);
    let long_key = Metadata::new().with(&key, "value");
    report(
        &root,
        "encode_metadata_key_257",
        "Metadata::to_json_vec",
        long_key.to_json_vec(),
    );

    let mut builder = MetadataSchema::builder();
    for index in 0..4097 {
        builder = builder.replace_field(
            &format!("key-{index}"),
            qubit_metadata::MetadataField::new(qubit_datatype::DataType::String, false),
        );
    }
    let schema = builder.build().unwrap();
    report(
        &root,
        "encode_schema_fields_4097",
        "MetadataSchema::to_json_vec",
        schema.to_json_vec(),
    );
    let long_schema = MetadataSchema::builder()
        .replace_field(
            "k".repeat(257).as_str(),
            qubit_metadata::MetadataField::new(qubit_datatype::DataType::String, false),
        )
        .build()
        .unwrap();
    report(
        &root,
        "encode_schema_key_257",
        "MetadataSchema::to_json_vec",
        long_schema.to_json_vec(),
    );
}
