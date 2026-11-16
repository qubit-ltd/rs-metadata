// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use crate::Metadata;
use crate::MetadataLimits;
use crate::MetadataLimitsBuilder;
use crate::MetadataWireDecodeError;

#[test]
fn test_metadata_decoder_rejects_invalid_internal_limits() {
    let limits = MetadataLimits::from_builder(MetadataLimitsBuilder::default().max_metadata_entries(4_097));
    assert!(limits.validate().is_err());
    let error = Metadata::decode_json_slice_with_limits(b"{}", limits)
        .expect_err("invalid internal limits must fail before envelope decoding");
    assert!(matches!(error, MetadataWireDecodeError::InvalidLimits(_)));
}

#[test]
fn test_validate_rejects_each_invalid_internal_domain_limit() {
    for (name, builder) in [
        (
            "metadata entries",
            MetadataLimitsBuilder::default().max_metadata_entries(4_097),
        ),
        (
            "schema fields",
            MetadataLimitsBuilder::default().max_schema_fields(4_097),
        ),
        ("key bytes", MetadataLimitsBuilder::default().max_key_bytes(257)),
    ] {
        assert!(
            MetadataLimits::from_builder(builder).validate().is_err(),
            "{name} must be rejected"
        );
    }
}

#[cfg(feature = "schema")]
#[test]
fn test_schema_decoder_rejects_invalid_internal_limits() {
    use crate::MetadataSchema;

    let limits = MetadataLimits::from_builder(MetadataLimitsBuilder::default().max_schema_fields(4_097));
    let error = MetadataSchema::decode_json_slice_with_limits(b"{}", limits)
        .expect_err("invalid internal limits must fail before envelope decoding");
    assert!(matches!(error, MetadataWireDecodeError::InvalidLimits(_)));
}

#[cfg(feature = "filter")]
#[test]
fn test_filter_decoder_rejects_invalid_internal_limits() {
    use crate::FilterLimits;
    use crate::MetadataFilter;

    let limits = MetadataLimits::from_builder(MetadataLimitsBuilder::default().max_key_bytes(257));
    let error = MetadataFilter::decode_json_slice_with_limits(b"{}", limits, FilterLimits::MAX)
        .expect_err("invalid internal limits must fail before envelope decoding");
    assert!(matches!(error, MetadataWireDecodeError::InvalidLimits(_)));
}
