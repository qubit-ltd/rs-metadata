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
#[cfg(feature = "filter")]
use qubit_metadata::MetadataFilter;
use qubit_metadata::MetadataWireEncodeError;
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

fn assert_json_rejection(
    write_with_limits: impl FnOnce(&mut ProbeWriter, JsonEncodeLimits) -> Result<(), MetadataWireEncodeError>,
) {
    let mut probe = ProbeWriter::new(None);
    let error = write_with_limits(&mut probe, default_json_encode_limits())
        .expect_err("overlong key should be rejected by JSON serialization");
    assert!(
        matches!(error, MetadataWireEncodeError::Json(_)),
        "unexpected error: {error:?}"
    );
    assert_eq!(probe.writes, 0);
    assert_eq!(probe.bytes, PREFIX);
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

    let metadata = Metadata::new().with(&"k".repeat(257), 1_i64);
    assert_json_rejection(|probe, limits| metadata.to_json_writer_with_limits(probe, limits));
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
    assert_json_rejection(|probe, limits| schema.to_json_writer_with_limits(probe, limits));
}
