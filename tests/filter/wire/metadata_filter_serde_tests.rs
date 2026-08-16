// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Metadata-filter serde tests.

#[cfg(feature = "json")]
use qubit_budget::ResourceLimit;
#[cfg(feature = "json")]
use qubit_budget::json::JsonDecodeLimits;
#[cfg(feature = "json")]
use qubit_budget::json::JsonResource;
use qubit_metadata::FilterExpression;
use qubit_metadata::FilterLimits;
use qubit_metadata::MetadataFilter;
#[cfg(feature = "json")]
use qubit_metadata::MetadataLimits;
#[cfg(feature = "json")]
use qubit_value::Value;

#[test]
fn test_all_and_none_serde_round_trip() {
    for filter in [MetadataFilter::all(), MetadataFilter::none()] {
        let encoded =
            serde_json::to_string(&filter).expect("filter should serialize");
        let decoded: MetadataFilter =
            serde_json::from_str(&encoded).expect("filter should deserialize");
        assert_eq!(decoded, filter);
    }
}

#[test]
fn test_metadata_filter_serde_round_trip() {
    let expression = FilterExpression::builder()
        .exists("status")
        .build()
        .unwrap();
    let filter = MetadataFilter::builder()
        .expression(expression)
        .build()
        .unwrap();
    let encoded = serde_json::to_string(&filter).unwrap();
    let decoded: MetadataFilter = serde_json::from_str(&encoded).unwrap();

    assert_eq!(decoded, filter);
}

#[test]
fn test_metadata_filter_rejects_unknown_fields_and_versions() {
    let mut unknown = serde_json::to_value(MetadataFilter::all()).unwrap();
    unknown["extra"] = serde_json::json!(true);
    let mut version = serde_json::to_value(MetadataFilter::all()).unwrap();
    version["version"] = serde_json::json!(5);
    let legacy_limits = serde_json::json!({
        "version": 1,
        "expression": {"kind": "all"},
        "options": {"numeric_comparison_policy": "exact"},
        "limits": {"max_depth": 64}
    });

    assert!(serde_json::from_value::<MetadataFilter>(unknown).is_err());
    assert!(serde_json::from_value::<MetadataFilter>(version).is_err());
    assert!(serde_json::from_value::<MetadataFilter>(legacy_limits).is_err());
}

#[test]
fn test_metadata_filter_serialization_excludes_transient_limits() {
    let expression = FilterExpression::builder()
        .exists("status")
        .build()
        .unwrap();
    let filter = MetadataFilter::builder()
        .expression(expression.clone())
        .limits(FilterLimits::builder().max_key_bytes(6).build().unwrap())
        .build()
        .unwrap();
    let default_limits_filter = MetadataFilter::builder()
        .expression(expression)
        .build()
        .unwrap();
    assert_eq!(filter, default_limits_filter);

    let encoded = serde_json::to_value(&filter).unwrap();
    assert!(encoded.get("limits").is_none());
    let decoded: MetadataFilter = serde_json::from_value(encoded).unwrap();
    assert_eq!(decoded.limits(), FilterLimits::MAX);
}

/// Preserves complex borrowed value payloads through the filter Wire shape.
#[cfg(feature = "json")]
#[test]
fn test_complex_metadata_filter_serde_preserves_wire_shape() {
    let expression = FilterExpression::builder()
        .eq("status", "active")
        .in_set("priority", [1_i64, 2_i64])
        .or_group(|group| {
            group
                .eq(
                    "payload",
                    Value::Json(serde_json::json!({"z": {"b": 2, "a": 1}})),
                )
                .exists("owner")
        })
        .build()
        .expect("complex expression should build");
    let filter = MetadataFilter::builder()
        .expression(expression)
        .build()
        .expect("complex filter should build");

    let encoded =
        serde_json::to_value(&filter).expect("filter should serialize");
    assert_eq!(encoded["version"], serde_json::json!(1));
    assert_eq!(encoded["expression"]["kind"], "or");
    assert!(encoded["expression"]["children"].is_array());
    assert!(encoded.to_string().contains("\"payload\""));

    let decoded: MetadataFilter =
        serde_json::from_value(encoded).expect("filter should deserialize");
    assert_eq!(decoded, filter);
}

#[cfg(feature = "json")]
#[test]
fn test_metadata_filter_json_round_trip_and_writer() {
    let expression = FilterExpression::builder()
        .exists("status")
        .build()
        .expect("expression should build");
    let filter = MetadataFilter::builder()
        .expression(expression)
        .build()
        .expect("filter should build");

    let encoded = filter.to_json_vec().expect("filter should encode");
    let decoded = MetadataFilter::decode_json_slice(&encoded)
        .expect("filter should decode");
    assert_eq!(decoded, filter);

    let mut output = Vec::new();
    filter
        .to_json_writer(&mut output)
        .expect("filter should write");
    assert_eq!(
        MetadataFilter::decode_json_slice(&output)
            .expect("written filter should decode"),
        filter
    );
}

#[cfg(feature = "json")]
#[test]
fn test_metadata_filter_json_decoder_rejects_malformed_input() {
    let error = MetadataFilter::decode_json_slice(b"null!")
        .expect_err("malformed filter input must be rejected");
    assert!(!error.to_string().is_empty());

    let error = MetadataFilter::decode_json_slice(b"{}")
        .expect_err("an incomplete filter envelope must be rejected");
    assert!(!error.to_string().is_empty());

    let limits = MetadataLimits::default().with_json_decode(
        JsonDecodeLimits::builder()
            .input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, 1))
            .build(),
    );
    let error = MetadataFilter::decode_json_slice_with_limits(
        b"{}",
        limits,
        FilterLimits::MAX,
    )
    .expect_err("the input budget must be enforced");
    assert!(!error.to_string().is_empty());
}

#[cfg(feature = "json")]
#[test]
fn test_metadata_filter_json_writer_propagates_io_error() {
    struct FailingWriter;

    impl std::io::Write for FailingWriter {
        fn write(&mut self, _buffer: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("write failed"))
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    MetadataFilter::all()
        .to_json_writer(FailingWriter)
        .expect_err("writer errors must be returned");
}
