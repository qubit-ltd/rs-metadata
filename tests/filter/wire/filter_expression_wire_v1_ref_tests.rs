// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Borrowed filter-expression wire tests.

#[cfg(feature = "json")]
use qubit_metadata::FilterExpression;
#[cfg(feature = "json")]
use qubit_metadata::MetadataFilter;
#[cfg(feature = "json")]
use qubit_value::Value;

#[cfg(feature = "json")]
#[test]
fn test_expression_serialization_preserves_nested_borrowed_payloads() {
    let expression = FilterExpression::builder()
        .eq(
            "payload",
            Value::Json(serde_json::json!({
                "z": {"b": 2, "a": 1},
            })),
        )
        .and_group(|group| group.exists("owner").ne("status", "inactive"))
        .build()
        .expect("expression should build");

    let filter = MetadataFilter::builder()
        .expression(expression)
        .build()
        .expect("filter should build");
    let encoded = serde_json::to_value(&filter).expect("filter should serialize");
    let encoded = &encoded["expression"];

    assert_eq!(encoded["kind"], "and");
    assert!(encoded["children"].is_array());
    assert_eq!(encoded.to_string().matches("payload").count(), 1);
}

#[cfg(feature = "json")]
#[test]
fn test_expression_serialization_matches_canonical_v1_bytes_for_every_node() {
    let expressions = [
        (FilterExpression::match_all(), r#"{"kind":"all"}"#),
        (FilterExpression::match_none(), r#"{"kind":"none"}"#),
        (
            FilterExpression::builder().eq("k", "v").build().unwrap(),
            r#"{"kind":"eq","key":"k","value":{"scalar":{"string":"v"}}}"#,
        ),
        (
            FilterExpression::builder().ne("k", "v").build().unwrap(),
            r#"{"kind":"ne","key":"k","value":{"scalar":{"string":"v"}}}"#,
        ),
        (
            FilterExpression::builder().lt("k", 1_i64).build().unwrap(),
            r#"{"kind":"lt","key":"k","value":{"scalar":{"int64":1}}}"#,
        ),
        (
            FilterExpression::builder().le("k", 1_i64).build().unwrap(),
            r#"{"kind":"le","key":"k","value":{"scalar":{"int64":1}}}"#,
        ),
        (
            FilterExpression::builder().gt("k", 1_i64).build().unwrap(),
            r#"{"kind":"gt","key":"k","value":{"scalar":{"int64":1}}}"#,
        ),
        (
            FilterExpression::builder().ge("k", 1_i64).build().unwrap(),
            r#"{"kind":"ge","key":"k","value":{"scalar":{"int64":1}}}"#,
        ),
        (
            FilterExpression::builder()
                .in_set("k", [1_i64, 2_i64])
                .build()
                .unwrap(),
            r#"{"kind":"in","key":"k","values":[{"scalar":{"int64":1}},{"scalar":{"int64":2}}]}"#,
        ),
        (
            FilterExpression::builder()
                .not_in_set("k", [1_i64, 2_i64])
                .build()
                .unwrap(),
            r#"{"kind":"not_in","key":"k","values":[{"scalar":{"int64":1}},{"scalar":{"int64":2}}]}"#,
        ),
        (
            FilterExpression::builder().exists("k").build().unwrap(),
            r#"{"kind":"exists","key":"k"}"#,
        ),
        (
            FilterExpression::builder().not_exists("k").build().unwrap(),
            r#"{"kind":"not_exists","key":"k"}"#,
        ),
        (
            FilterExpression::builder()
                .and_group(|g| g.exists("a").exists("b"))
                .build()
                .unwrap(),
            r#"{"kind":"and","children":[{"kind":"exists","key":"a"},{"kind":"exists","key":"b"}]}"#,
        ),
        (
            FilterExpression::builder()
                .or_group(|g| g.exists("a").or_group(|inner| inner.exists("b")))
                .build()
                .unwrap(),
            r#"{"kind":"or","children":[{"kind":"exists","key":"a"},{"kind":"exists","key":"b"}]}"#,
        ),
        (
            FilterExpression::builder()
                .exists("a")
                .not()
                .build()
                .unwrap(),
            r#"{"kind":"not","expression":{"kind":"exists","key":"a"}}"#,
        ),
    ];

    for (expression, expected) in expressions {
        let filter = MetadataFilter::builder()
            .expression(expression)
            .build()
            .unwrap();
        let expected = format!(
            r#"{{"version":1,"expression":{expected},"options":{{"numeric_comparison_policy":"exact"}}}}"#
        );
        let encoded = serde_json::to_string(&filter).unwrap();
        assert_eq!(encoded, expected);
        assert_eq!(serde_json::to_vec(&filter).unwrap(), expected.as_bytes());
        assert_eq!(filter.to_json_vec().unwrap(), expected.as_bytes());
        assert_eq!(
            filter
                .to_json_vec_with_limits(qubit_metadata::default_json_encode_limits())
                .unwrap(),
            expected.as_bytes()
        );

        let mut writer = Vec::new();
        filter.to_json_writer(&mut writer).unwrap();
        assert_eq!(writer, expected.as_bytes());
    }
}

#[cfg(feature = "json")]
#[test]
fn test_nested_json_value_and_escaped_unicode_match_canonical_v1_bytes() {
    let expression = FilterExpression::builder()
        .eq(
            "key\"雪",
            Value::Json(serde_json::json!({
                "z": {"b": 2, "a": 1},
                "text": "quote\" line\n 雪",
            })),
        )
        .and_group(|group| group.exists("owner"))
        .build()
        .unwrap();
    let filter = MetadataFilter::builder()
        .expression(expression)
        .build()
        .unwrap();
    let expected = r#"{"version":1,"expression":{"kind":"and","children":[{"kind":"eq","key":"key\"雪","value":{"scalar":{"json":{"text":"quote\" line\n 雪","z":{"a":1,"b":2}}}}},{"kind":"exists","key":"owner"}]},"options":{"numeric_comparison_policy":"exact"}}"#;

    assert_eq!(serde_json::to_string(&filter).unwrap(), expected);
    assert_eq!(serde_json::to_vec(&filter).unwrap(), expected.as_bytes());
    assert_eq!(filter.to_json_vec().unwrap(), expected.as_bytes());
    assert_eq!(
        filter
            .to_json_vec_with_limits(qubit_metadata::default_json_encode_limits())
            .unwrap(),
        expected.as_bytes()
    );

    let mut writer = Vec::new();
    filter.to_json_writer(&mut writer).unwrap();
    assert_eq!(writer, expected.as_bytes());
}

#[cfg(feature = "json")]
#[test]
fn test_filter_output_budget_fails_before_writer_is_touched() {
    use qubit_budget::ResourceLimit;
    use qubit_budget::json::JsonEncodeLimits;
    use qubit_budget::json::JsonResource;

    let filter = MetadataFilter::builder()
        .expression(FilterExpression::builder().exists("key").build().unwrap())
        .build()
        .unwrap();
    let limits = JsonEncodeLimits::builder()
        .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 1))
        .build();
    let mut writer = Vec::new();

    assert!(filter.to_json_vec_with_limits(limits).is_err());
    assert!(
        filter
            .to_json_writer_with_limits(&mut writer, limits)
            .is_err()
    );
    assert!(writer.is_empty());
}
