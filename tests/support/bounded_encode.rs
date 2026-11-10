// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Shared bounded-encoding oracle for fuzz targets and regression tests.

use std::io;
use std::io::Write;

use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonResource;
use qubit_metadata::MetadataWireEncodeError;
use qubit_metadata::default_json_encode_limits;

pub const PREFIX: &[u8] = b"sentinel:";

#[derive(Debug)]
pub struct ProbeWriter {
    pub bytes: Vec<u8>,
    pub writes: usize,
}

impl ProbeWriter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            bytes: PREFIX.to_vec(),
            writes: 0,
        }
    }
}

impl Write for ProbeWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.writes += 1;
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn same_error(left: &MetadataWireEncodeError, right: &MetadataWireEncodeError) {
    match (left, right) {
        (MetadataWireEncodeError::Budget(a), MetadataWireEncodeError::Budget(b)) => {
            assert_eq!(a, b);
        }
        (MetadataWireEncodeError::Domain(a), MetadataWireEncodeError::Domain(b)) => {
            assert_eq!(a, b);
        }
        (
            MetadataWireEncodeError::Quantity {
                resource: left_resource,
                source: left_source,
            },
            MetadataWireEncodeError::Quantity {
                resource: right_resource,
                source: right_source,
            },
        ) => {
            assert_eq!(left_resource, right_resource);
            assert_eq!(left_source, right_source);
        }
        (MetadataWireEncodeError::Syntax(_), MetadataWireEncodeError::Syntax(_))
        | (MetadataWireEncodeError::Json(_), MetadataWireEncodeError::Json(_))
        | (MetadataWireEncodeError::Io(_), MetadataWireEncodeError::Io(_)) => {}
        _ => panic!("vec and writer returned different error categories: {left:?} / {right:?}"),
    }
}

pub fn check(
    canonical: &[u8],
    caps: impl IntoIterator<Item = usize>,
    vec_encode: impl Fn(JsonEncodeLimits) -> Result<Vec<u8>, MetadataWireEncodeError>,
    mut writer_encode: impl FnMut(&mut ProbeWriter, JsonEncodeLimits) -> Result<(), MetadataWireEncodeError>,
) {
    for cap in caps {
        let limits = default_json_encode_limits()
            .into_builder()
            .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, cap))
            .build();
        check_one(canonical, Some(cap), limits, &vec_encode, &mut writer_encode);
    }
    if canonical.len() > 1 {
        let below = canonical.len() - 1;
        let limits = default_json_encode_limits()
            .into_builder()
            .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, below))
            .build();
        let error = vec_encode(limits).expect_err("L-1 output cap must reject canonical bytes");
        assert!(
            matches!(error, MetadataWireEncodeError::Budget(qubit_budget::BudgetError::Insufficient {
            resource: JsonResource::OutputBytes, limit, requested, ..
        }) if limit == below && requested == canonical.len())
        );
        let mut writer = ProbeWriter::new();
        let writer_error = writer_encode(&mut writer, limits).expect_err("writer must reject canonical bytes at L-1");
        assert!(
            matches!(writer_error, MetadataWireEncodeError::Budget(qubit_budget::BudgetError::Insufficient {
            resource: JsonResource::OutputBytes, limit, requested, ..
        }) if limit == below && requested == canonical.len())
        );
        assert_eq!(writer.writes, 0);
        assert_eq!(writer.bytes, PREFIX);

        let limits = default_json_encode_limits()
            .into_builder()
            .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, canonical.len()))
            .build();
        assert_eq!(vec_encode(limits).expect("L output cap must succeed"), canonical);
        let mut writer = ProbeWriter::new();
        writer_encode(&mut writer, limits).expect("writer must succeed at L output cap");
        let mut expected = PREFIX.to_vec();
        expected.extend_from_slice(canonical);
        assert_eq!(writer.bytes, expected);
    }
    // Exercise independent structure and string payload limits while
    // preserving all other defaults. Either outcome is acceptable for an
    // arbitrary fuzz value as long as both encoding paths agree.
    for limits in [
        default_json_encode_limits().into_builder().max_nodes(1).build(),
        default_json_encode_limits().into_builder().max_string_bytes(1).build(),
    ] {
        check_one(canonical, None, limits, &vec_encode, &mut writer_encode);
    }
}

fn check_one(
    canonical: &[u8],
    output_cap: Option<usize>,
    limits: JsonEncodeLimits,
    vec_encode: &impl Fn(JsonEncodeLimits) -> Result<Vec<u8>, MetadataWireEncodeError>,
    writer_encode: &mut impl FnMut(&mut ProbeWriter, JsonEncodeLimits) -> Result<(), MetadataWireEncodeError>,
) {
    let vector = vec_encode(limits);
    let mut writer = ProbeWriter::new();
    let written = writer_encode(&mut writer, limits);
    match (vector, written) {
        (Ok(bytes), Ok(())) => {
            if let Some(cap) = output_cap {
                assert!(bytes.len() <= cap);
            }
            assert_eq!(bytes, canonical);
            let mut expected = PREFIX.to_vec();
            expected.extend_from_slice(&bytes);
            assert_eq!(writer.bytes, expected);
            assert!(writer.writes > 0);
        }
        (Err(vector_error), Err(writer_error)) => {
            same_error(&vector_error, &writer_error);
            assert_eq!(writer.writes, 0, "failed encoding must not call write");
            assert_eq!(writer.bytes, PREFIX);
        }
        (vector, writer) => panic!("vec and writer disagree: {vector:?} / {writer:?}"),
    }
}

#[must_use]
pub fn caps(canonical_len: usize, first_input_byte: u8) -> [usize; 5] {
    let below = canonical_len.saturating_sub(1).max(1);
    let random = usize::from(first_input_byte) % canonical_len.saturating_add(1).max(1) + 1;
    [1, below, canonical_len, canonical_len.saturating_add(1), random]
}
