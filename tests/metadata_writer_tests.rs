// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Integration tests for buffered JSON writers on metadata domain objects.

#![cfg(feature = "json")]

use std::error::Error;
use std::fmt;
use std::io;
use std::io::Write;

use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonResource;
#[cfg(feature = "schema")]
use qubit_datatype::DataType;
#[cfg(feature = "filter")]
use qubit_metadata::FilterExpression;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataError;
#[cfg(feature = "filter")]
use qubit_metadata::MetadataFilter;
use qubit_metadata::MetadataWireEncodeError;
use qubit_metadata::MetadataWireLimitKind;
use qubit_metadata::default_json_encode_limits;

const PREFIX: &[u8] = b"sentinel:";

#[derive(Debug)]
struct DestinationFailure;

impl fmt::Display for DestinationFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("probe destination failed")
    }
}

impl Error for DestinationFailure {}

/// A destination that records accepted bytes and can fail after a chosen count.
struct ProbeWriter {
    bytes: Vec<u8>,
    writes: usize,
    accepted: usize,
    fail_after: Option<usize>,
}

impl ProbeWriter {
    fn new(fail_after: Option<usize>) -> Self {
        Self {
            bytes: PREFIX.to_vec(),
            writes: 0,
            accepted: 0,
            fail_after,
        }
    }
}

impl Write for ProbeWriter {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        self.writes += 1;
        if input.is_empty() {
            return Ok(0);
        }
        let allowed = self
            .fail_after
            .map_or(input.len(), |cap| cap.saturating_sub(self.accepted));
        if allowed == 0 {
            return Err(io::Error::other(DestinationFailure));
        }
        let count = input.len().min(allowed);
        self.bytes.extend_from_slice(&input[..count]);
        self.accepted += count;
        Ok(count)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn assert_writer_contract(
    expected: &[u8],
    explicit_expected: &[u8],
    default_write: impl FnOnce(&mut ProbeWriter) -> Result<(), MetadataWireEncodeError>,
    mut explicit_write: impl FnMut(&mut ProbeWriter, JsonEncodeLimits) -> Result<(), MetadataWireEncodeError>,
    mut partial_write: impl FnMut(&mut ProbeWriter) -> Result<(), MetadataWireEncodeError>,
) {
    let mut probe = ProbeWriter::new(None);
    default_write(&mut probe).expect("default writer should write successfully");
    let mut expected_with_prefix = PREFIX.to_vec();
    expected_with_prefix.extend_from_slice(expected);
    assert_eq!(probe.bytes, expected_with_prefix);

    let limits = default_json_encode_limits();
    let mut probe = ProbeWriter::new(None);
    explicit_write(&mut probe, limits).expect("explicit writer should write successfully");
    let mut expected_with_prefix = PREFIX.to_vec();
    expected_with_prefix.extend_from_slice(explicit_expected);
    assert_eq!(probe.bytes, expected_with_prefix);

    let limits = default_json_encode_limits()
        .into_builder()
        .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 1))
        .build();
    let mut probe = ProbeWriter::new(None);
    let error = explicit_write(&mut probe, limits).expect_err("output budget should reject the document");
    match error {
        MetadataWireEncodeError::Budget(source) => {
            assert_eq!(*source.resource(), JsonResource::OutputBytes);
        }
        other => panic!("expected output budget failure, got {other:?}"),
    }
    assert_eq!(probe.writes, 0);
    assert_eq!(probe.bytes, PREFIX);

    let mut probe = ProbeWriter::new(Some(0));
    let error = partial_write(&mut probe).expect_err("zero-byte destination cap should fail immediately");
    assert!(
        matches!(error, MetadataWireEncodeError::Io(_)),
        "unexpected error: {error:?}"
    );
    assert_eq!(probe.accepted, 0);
    assert_eq!(probe.bytes, PREFIX);

    let mut probe = ProbeWriter::new(Some(3));
    let error = partial_write(&mut probe).expect_err("destination should fail after three bytes");
    let MetadataWireEncodeError::Io(source) = error else {
        panic!("expected I/O failure, got {error:?}");
    };
    assert_eq!(source.kind(), io::ErrorKind::Other);
    assert!(source.get_ref().is_some_and(|cause| cause.is::<DestinationFailure>()));
    let outer = MetadataWireEncodeError::Io(source);
    let chained = Error::source(&outer).expect("wire error should expose its I/O source");
    let chained = chained
        .downcast_ref::<io::Error>()
        .expect("source should be the I/O error");
    assert!(chained.get_ref().is_some_and(|cause| cause.is::<DestinationFailure>()));
    assert!(expected.len() > 3, "encoded document must contain at least three bytes");
    let mut expected_prefix = PREFIX.to_vec();
    expected_prefix.extend_from_slice(&expected[..3]);
    assert_eq!(probe.bytes, expected_prefix);
    assert_eq!(probe.accepted, 3);
}

fn assert_value_budget_failure(
    write_with_limits: impl FnOnce(&mut ProbeWriter, JsonEncodeLimits) -> Result<(), MetadataWireEncodeError>,
) {
    let limits = JsonEncodeLimits::builder().max_number_bytes(2).build();
    let mut probe = ProbeWriter::new(None);
    let error = write_with_limits(&mut probe, limits).expect_err("number value should exceed its budget");
    match error {
        MetadataWireEncodeError::Budget(source) => {
            assert_eq!(*source.resource(), JsonResource::NumberBytes);
        }
        other => panic!("expected number budget failure, got {other:?}"),
    }
    assert_eq!(probe.writes, 0);
    assert_eq!(probe.bytes, PREFIX);
}

#[cfg(feature = "schema")]
fn assert_schema_domain_rejection(
    schema: &qubit_metadata::MetadataSchema,
    expected_kind: MetadataWireLimitKind,
    expected_value: usize,
    expected_maximum: usize,
) {
    let limits = default_json_encode_limits()
        .into_builder()
        .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 1))
        .build();
    for error in [
        schema.to_json_vec().expect_err("default vec must reject wire limit"),
        schema
            .to_json_vec_with_limits(limits)
            .expect_err("explicit vec must reject wire limit"),
    ] {
        let MetadataWireEncodeError::Domain(MetadataError::WireLimitExceeded { kind, value, maximum }) = &error else {
            panic!("expected domain wire-limit error, got {error:?}");
        };
        assert_eq!(*kind, expected_kind);
        assert_eq!(*value, expected_value);
        assert_eq!(*maximum, expected_maximum);
        assert!(Error::source(&error).is_some());
    }

    for (name, write) in [("default writer", false), ("explicit writer", true)] {
        let mut probe = ProbeWriter::new(None);
        let error = if write {
            schema.to_json_writer_with_limits(&mut probe, limits)
        } else {
            schema.to_json_writer(&mut probe)
        }
        .expect_err(name);
        let MetadataWireEncodeError::Domain(MetadataError::WireLimitExceeded { kind, value, maximum }) = &error else {
            panic!("expected domain wire-limit error, got {error:?}");
        };
        assert_eq!(*kind, expected_kind);
        assert_eq!(*value, expected_value);
        assert_eq!(*maximum, expected_maximum);
        assert!(Error::source(&error).is_some());
        assert_eq!(probe.writes, 0);
        assert_eq!(probe.bytes, PREFIX);
    }
}

#[test]
fn test_metadata_json_writers_preserve_buffered_contract() {
    let metadata = Metadata::new().with("value", 123_i64);
    let expected = metadata.to_json_vec().expect("metadata should encode");
    let explicit_expected = metadata
        .to_json_vec_with_limits(default_json_encode_limits())
        .expect("metadata should encode with explicit limits");
    assert_writer_contract(
        &expected,
        &explicit_expected,
        |probe| metadata.to_json_writer(probe),
        |probe, limits| metadata.to_json_writer_with_limits(probe, limits),
        |probe| metadata.to_json_writer(probe),
    );

    let metadata = Metadata::new().with("value", 123_i64);
    assert_value_budget_failure(|probe, limits| metadata.to_json_writer_with_limits(probe, limits));
}

fn assert_metadata_domain_rejection(
    metadata: &Metadata,
    expected_kind: MetadataWireLimitKind,
    expected_value: usize,
    maximum: usize,
) {
    let limits = JsonEncodeLimits::builder().max_number_bytes(2).build();
    for error in [
        metadata.to_json_vec().expect_err("default vec must reject wire limit"),
        metadata
            .to_json_vec_with_limits(limits)
            .expect_err("explicit vec must reject wire limit"),
    ] {
        let MetadataWireEncodeError::Domain(MetadataError::WireLimitExceeded {
            kind,
            value,
            maximum: actual_maximum,
        }) = error
        else {
            panic!("expected domain wire-limit error, got {error:?}");
        };
        assert_eq!(kind, expected_kind);
        assert_eq!(value, expected_value);
        assert_eq!(actual_maximum, maximum);
    }
    let mut default_probe = ProbeWriter::new(None);
    let error = metadata
        .to_json_writer(&mut default_probe)
        .expect_err("default writer must reject");
    let MetadataWireEncodeError::Domain(MetadataError::WireLimitExceeded {
        kind,
        value,
        maximum: actual,
    }) = error
    else {
        panic!("expected domain wire-limit error, got {error:?}");
    };
    assert_eq!(kind, expected_kind);
    assert_eq!(value, expected_value);
    assert_eq!(actual, maximum);
    assert_eq!(default_probe.writes, 0);
    assert_eq!(default_probe.bytes, PREFIX);
    let mut explicit_probe = ProbeWriter::new(None);
    let error = metadata
        .to_json_writer_with_limits(&mut explicit_probe, limits)
        .expect_err("explicit writer must reject");
    let MetadataWireEncodeError::Domain(MetadataError::WireLimitExceeded {
        kind,
        value,
        maximum: actual,
    }) = error
    else {
        panic!("expected domain wire-limit error, got {error:?}");
    };
    assert_eq!(kind, expected_kind);
    assert_eq!(value, expected_value);
    assert_eq!(actual, maximum);
    assert_eq!(explicit_probe.writes, 0);
    assert_eq!(explicit_probe.bytes, PREFIX);
}

#[test]
fn test_metadata_wire_contract_precedes_value_preflight() {
    let key_metadata = Metadata::new().with(&"k".repeat(257), 123_i64);
    assert_metadata_domain_rejection(&key_metadata, MetadataWireLimitKind::KeyBytes, 257, 256);

    let mut entries_metadata = Metadata::new();
    for index in 0..4_097 {
        entries_metadata.insert(&format!("k{index}"), 123_i64);
    }
    assert_metadata_domain_rejection(&entries_metadata, MetadataWireLimitKind::Entries, 4_097, 4_096);
}

#[cfg(feature = "filter")]
#[test]
fn test_filter_json_writers_preserve_buffered_contract() {
    let filter = MetadataFilter::builder()
        .expression(FilterExpression::builder().eq("value", 123_i64).build().unwrap())
        .build()
        .unwrap();
    let expected = filter.to_json_vec().expect("filter should encode");
    let explicit_expected = filter
        .to_json_vec_with_limits(default_json_encode_limits())
        .expect("filter should encode with explicit limits");
    assert_writer_contract(
        &expected,
        &explicit_expected,
        |probe| filter.to_json_writer(probe),
        |probe, limits| filter.to_json_writer_with_limits(probe, limits),
        |probe| filter.to_json_writer(probe),
    );

    let filter = MetadataFilter::builder()
        .expression(FilterExpression::builder().eq("value", 123_i64).build().unwrap())
        .build()
        .unwrap();
    assert_value_budget_failure(|probe, limits| filter.to_json_writer_with_limits(probe, limits));
}

#[cfg(feature = "schema")]
#[test]
fn test_schema_json_writers_preserve_buffered_contract() {
    let schema = qubit_metadata::MetadataSchema::builder()
        .optional("value", DataType::Int64)
        .build()
        .unwrap();
    let expected = schema.to_json_vec().expect("schema should encode");
    let explicit_expected = schema
        .to_json_vec_with_limits(default_json_encode_limits())
        .expect("schema should encode with explicit limits");
    assert_writer_contract(
        &expected,
        &explicit_expected,
        |probe| schema.to_json_writer(probe),
        |probe, limits| schema.to_json_writer_with_limits(probe, limits),
        |probe| schema.to_json_writer(probe),
    );

    let schema = qubit_metadata::MetadataSchema::builder()
        .optional(&"k".repeat(257), DataType::String)
        .build()
        .unwrap();
    assert_schema_domain_rejection(&schema, MetadataWireLimitKind::KeyBytes, 257, 256);
}

#[cfg(feature = "schema")]
#[test]
fn test_schema_json_wire_contract_boundaries_and_budget_precedence() {
    let mut builder = qubit_metadata::MetadataSchema::builder();
    for index in 0..4_096 {
        builder = builder.optional(&format!("field-{index}"), DataType::Int64);
    }
    let schema = builder.build().unwrap();
    assert!(schema.to_json_vec().is_ok(), "4,096 fields are within the wire limit");

    let mut builder = qubit_metadata::MetadataSchema::builder();
    for index in 0..4_097 {
        builder = builder.optional(&format!("field-{index}"), DataType::Int64);
    }
    let schema = builder.build().unwrap();
    assert_schema_domain_rejection(&schema, MetadataWireLimitKind::Entries, 4_097, 4_096);

    let key_ascii_256 = "a".repeat(256);
    let schema = qubit_metadata::MetadataSchema::builder()
        .optional(&key_ascii_256, DataType::String)
        .build()
        .unwrap();
    assert!(
        schema.to_json_vec().is_ok(),
        "256 ASCII key bytes are within the wire limit"
    );

    let key_256 = format!("{}a", "界".repeat(85));
    assert_eq!(key_256.len(), 256);
    let schema = qubit_metadata::MetadataSchema::builder()
        .optional(&key_256, DataType::String)
        .build()
        .unwrap();
    assert!(
        schema.to_json_vec().is_ok(),
        "256 UTF-8 key bytes are within the wire limit"
    );

    let key_257 = format!("{}ab", "界".repeat(85));
    assert_eq!(key_257.len(), 257);
    let schema = qubit_metadata::MetadataSchema::builder()
        .optional(&key_257, DataType::String)
        .build()
        .unwrap();
    assert_schema_domain_rejection(&schema, MetadataWireLimitKind::KeyBytes, 257, 256);

    let schema = qubit_metadata::MetadataSchema::builder()
        .optional("valid", DataType::String)
        .build()
        .unwrap();
    let tiny_budget = default_json_encode_limits()
        .into_builder()
        .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 1))
        .build();
    assert!(matches!(
        schema.to_json_vec_with_limits(tiny_budget),
        Err(MetadataWireEncodeError::Budget(_))
    ));
}
