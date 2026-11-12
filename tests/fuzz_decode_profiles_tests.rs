// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Boundary regressions for the deliberately small fuzz decoding profiles.

#![cfg(feature = "json")]

mod support;

#[cfg(feature = "filter")]
use qubit_metadata::FilterLimitKind;
#[cfg(feature = "filter")]
use qubit_metadata::FilterLimits;
use qubit_metadata::Metadata;
#[cfg(feature = "filter")]
use qubit_metadata::MetadataError;
#[cfg(feature = "filter")]
use qubit_metadata::MetadataLimits;
#[cfg(feature = "filter")]
use qubit_metadata::MetadataWireDecodeError;

#[test]
fn test_small_metadata_profile_rejects_domain_key_and_entry_boundaries() {
    let valid_key = br#"{"version":1,"values":{"1234567890123456":{"scalar":{"string":"ok"}}}}"#;
    let invalid_key = br#"{"version":1,"values":{"12345678901234567":{"scalar":{"string":"ok"}}}}"#;
    assert!(Metadata::decode_json_slice(valid_key).is_ok());
    assert!(
        Metadata::decode_json_slice_with_limits(
            valid_key,
            support::decode_profiles::small_metadata_limits()
        )
        .is_ok()
    );
    assert!(Metadata::decode_json_slice(invalid_key).is_ok());
    assert!(
        Metadata::decode_json_slice_with_limits(
            invalid_key,
            support::decode_profiles::small_metadata_limits()
        )
        .is_err()
    );

    let mut entries = String::from(r#"{"version":1,"values":{"#);
    for index in 0..5 {
        if index != 0 {
            entries.push(',');
        }
        entries.push_str(&format!(r#""k{index}":{{"scalar":{{"string":"v"}}}}"#));
    }
    entries.push_str("}}");
    assert!(Metadata::decode_json_slice(entries.as_bytes()).is_ok());
    assert!(
        Metadata::decode_json_slice_with_limits(
            entries.as_bytes(),
            support::decode_profiles::small_metadata_limits()
        )
        .is_err()
    );
}

#[cfg(feature = "schema")]
#[test]
fn test_small_schema_profile_rejects_domain_field_count_and_key_boundaries() {
    use qubit_metadata::MetadataSchema;
    let key_ok = br#"{"version":1,"fields":{"1234567890123456":{"data_type":"string","required":false}},"unknown_metadata_field_policy":"reject","unknown_filter_field_policy":"reject"}"#;
    let key_bad = br#"{"version":1,"fields":{"12345678901234567":{"data_type":"string","required":false}},"unknown_metadata_field_policy":"reject","unknown_filter_field_policy":"reject"}"#;
    assert!(MetadataSchema::decode_json_slice(key_ok).is_ok());
    assert!(
        MetadataSchema::decode_json_slice_with_limits(
            key_ok,
            support::decode_profiles::small_metadata_limits()
        )
        .is_ok()
    );
    assert!(MetadataSchema::decode_json_slice(key_bad).is_ok());
    assert!(
        MetadataSchema::decode_json_slice_with_limits(
            key_bad,
            support::decode_profiles::small_metadata_limits()
        )
        .is_err()
    );
    let mut fields = String::from(r#"{"version":1,"fields":{"#);
    for index in 0..5 {
        if index != 0 {
            fields.push(',');
        }
        fields.push_str(&format!(
            r#""f{index}":{{"data_type":"string","required":false}}"#
        ));
    }
    fields.push_str(
        r#"},"unknown_metadata_field_policy":"reject","unknown_filter_field_policy":"reject"}"#,
    );
    assert!(MetadataSchema::decode_json_slice(fields.as_bytes()).is_ok());
    assert!(
        MetadataSchema::decode_json_slice_with_limits(
            fields.as_bytes(),
            support::decode_profiles::small_metadata_limits()
        )
        .is_err()
    );
}

#[cfg(feature = "filter")]
#[test]
fn test_small_filter_profile_rejects_node_depth_and_set_boundaries() {
    use qubit_metadata::MetadataFilter;
    let metadata_limits = MetadataLimits::default();
    let small_metadata = support::decode_profiles::small_metadata_limits();
    let small_filter = support::decode_profiles::small_filter_limits();
    let valid = br#"{"version":1,"expression":{"kind":"in","key":"region","values":[{"scalar":{"string":"a"}},{"scalar":{"string":"b"}},{"scalar":{"string":"c"}},{"scalar":{"string":"d"}}]},"options":{"numeric_comparison_policy":"exact"}}"#;
    let oversized_set = br#"{"version":1,"expression":{"kind":"in","key":"region","values":[{"scalar":{"string":"a"}},{"scalar":{"string":"b"}},{"scalar":{"string":"c"}},{"scalar":{"string":"d"}},{"scalar":{"string":"e"}}]},"options":{"numeric_comparison_policy":"exact"}}"#;
    assert!(
        MetadataFilter::decode_json_slice_with_limits(valid, metadata_limits, small_filter).is_ok()
    );
    assert!(
        MetadataFilter::decode_json_slice_with_limits(oversized_set, metadata_limits, small_filter)
            .is_err()
    );
    assert!(
        MetadataFilter::decode_json_slice_with_limits(
            oversized_set,
            metadata_limits,
            support::decode_profiles::small_filter_limits()
        )
        .is_err()
    );
    let nested = br#"{"version":1,"expression":{"kind":"not","expression":{"kind":"not","expression":{"kind":"not","expression":{"kind":"not","expression":{"kind":"exists","key":"x"}}}}},"options":{"numeric_comparison_policy":"exact"}}"#;
    assert!(
        MetadataFilter::decode_json_slice_with_limits(nested, small_metadata, small_filter)
            .is_err()
    );
    let key_ok = br#"{"version":1,"expression":{"kind":"exists","key":"1234567890123456"},"options":{"numeric_comparison_policy":"exact"}}"#;
    let key_bad = br#"{"version":1,"expression":{"kind":"exists","key":"12345678901234567"},"options":{"numeric_comparison_policy":"exact"}}"#;
    assert!(
        MetadataFilter::decode_json_slice_with_limits(
            key_ok,
            MetadataLimits::default(),
            support::decode_profiles::small_filter_limits()
        )
        .is_ok()
    );
    assert!(
        MetadataFilter::decode_json_slice_with_limits(
            key_bad,
            MetadataLimits::default(),
            support::decode_profiles::small_filter_limits()
        )
        .is_err()
    );
    let children = |count: usize| {
        let items = (0..count)
            .map(|i| format!(r#"{{"kind":"exists","key":"k{i}"}}"#))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            r#"{{"version":1,"expression":{{"kind":"and","children":[{items}]}},"options":{{"numeric_comparison_policy":"exact"}}}}"#
        )
    };
    assert!(
        MetadataFilter::decode_json_slice_with_limits(
            children(15).as_bytes(),
            MetadataLimits::default(),
            support::decode_profiles::small_filter_limits()
        )
        .is_ok()
    );
    assert!(
        MetadataFilter::decode_json_slice_with_limits(
            children(16).as_bytes(),
            MetadataLimits::default(),
            support::decode_profiles::small_filter_limits()
        )
        .is_err()
    );
}

#[cfg(feature = "filter")]
#[test]
fn test_production_filter_profile_accepts_named_boundary_fixtures_and_corpus() {
    use qubit_metadata::MetadataFilter;

    let target = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fuzz/fuzz_targets/decode_metadata_filter.rs");
    if target.exists() {
        let target = std::fs::read_to_string(target).expect("fuzz target should read");
        let compact: String = target
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect();
        assert!(compact.contains("letproduction_filter_limits=FilterLimits::MAX;"));
        assert!(compact.contains(
            "MetadataFilter::decode_json_slice_with_limits(data,MetadataLimits::default(),production_filter_limits)"
        ));
        assert!(
            compact
                .contains("check(value,data,MetadataLimits::default(),production_filter_limits)")
        );
    }

    let fixtures = [
        (
            "depth",
            include_bytes!("fixtures/filter-production-depth.json").as_slice(),
        ),
        (
            "nodes",
            include_bytes!("fixtures/filter-production-nodes.json").as_slice(),
        ),
        (
            "set",
            include_bytes!("fixtures/filter-production-set.json").as_slice(),
        ),
        (
            "key",
            include_bytes!("fixtures/filter-production-key.json").as_slice(),
        ),
    ];
    for (name, fixture) in fixtures {
        let decoded = MetadataFilter::decode_json_slice_with_limits(
            fixture,
            MetadataLimits::default(),
            FilterLimits::MAX,
        )
        .unwrap_or_else(|error| panic!("production {name} fixture must be accepted: {error:?}"));
        let encoded = serde_json::to_vec(&decoded).expect("fixture should serialize");
        let round_tripped = MetadataFilter::decode_json_slice_with_limits(
            &encoded,
            MetadataLimits::default(),
            FilterLimits::MAX,
        )
        .expect("accepted fixture should round trip under production filter limits");
        assert_eq!(round_tripped, decoded);
        let corpus = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fuzz/corpus/decode_metadata_filter")
            .join(format!("production-{name}.json"));
        if corpus.exists() {
            assert_eq!(std::fs::read(corpus).expect("corpus should read"), fixture);
        }
    }
}

#[cfg(feature = "filter")]
#[test]
fn test_filter_hard_boundaries_report_structured_limit_errors() {
    use qubit_metadata::MetadataFilter;

    fn envelope(expression: serde_json::Value) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "version": 1,
            "expression": expression,
            "options": {"numeric_comparison_policy": "exact"}
        }))
        .expect("envelope should serialize")
    }
    fn assert_limit(
        input: &[u8],
        limits: FilterLimits,
        kind: FilterLimitKind,
        value: usize,
        maximum: usize,
    ) {
        let result =
            MetadataFilter::decode_json_slice_with_limits(input, MetadataLimits::default(), limits);
        assert!(
            matches!(
                result,
                Err(MetadataWireDecodeError::Filter(MetadataError::FilterLimitExceeded {
                    kind: actual,
                    value: actual_value,
                    maximum: actual_maximum,
                }))
                    if actual == kind && actual_value == value && actual_maximum == maximum
            ),
            "expected {kind:?}={value} max {maximum}, got {result:?}"
        );
    }

    let max = FilterLimits::MAX;
    let nodes = |count: usize| {
        envelope(serde_json::json!({
            "kind": "and",
            "children": (0..count).map(|_| serde_json::json!({"kind":"exists", "key":"x"})).collect::<Vec<_>>()
        }))
    };
    assert!(
        MetadataFilter::decode_json_slice_with_limits(&nodes(255), MetadataLimits::default(), max)
            .is_ok()
    );
    assert_limit(&nodes(256), max, FilterLimitKind::Nodes, 257, 256);

    let set = |count: usize| {
        envelope(serde_json::json!({
            "kind": "in", "key": "x",
            "values": (0..count).map(|_| serde_json::json!({"scalar":{"int64":1}})).collect::<Vec<_>>()
        }))
    };
    assert!(
        MetadataFilter::decode_json_slice_with_limits(&set(128), MetadataLimits::default(), max)
            .is_ok()
    );
    assert_limit(&set(129), max, FilterLimitKind::SetValues, 129, 128);

    let key = |size: usize| envelope(serde_json::json!({"kind":"exists", "key":"x".repeat(size)}));
    assert!(
        MetadataFilter::decode_json_slice_with_limits(&key(256), MetadataLimits::default(), max)
            .is_ok()
    );
    assert_limit(&key(257), max, FilterLimitKind::KeyBytes, 257, 256);

    let not_chain = |count: usize| {
        let expression = (0..count).fold(
            serde_json::json!({"kind":"exists", "key":"x"}),
            |child, _| serde_json::json!({"kind":"not", "expression":child}),
        );
        envelope(expression)
    };
    let json_decode = MetadataLimits::default()
        .json_decode()
        .into_builder()
        .max_depth(256)
        .build();
    let metadata = MetadataLimits::builder()
        .json_decode(json_decode)
        .build()
        .expect("finite JSON profile should build");
    let finite = FilterLimits::builder()
        .max_depth(64)
        .build()
        .expect("finite filter profile should build");
    assert!(
        MetadataFilter::decode_json_slice_with_limits(&not_chain(63), metadata, finite).is_ok()
    );
    let too_deep = MetadataFilter::decode_json_slice_with_limits(&not_chain(64), metadata, finite)
        .expect_err("65 AST levels should exceed the filter depth profile");
    assert!(matches!(
        too_deep,
        MetadataWireDecodeError::Filter(MetadataError::FilterLimitExceeded {
            kind: FilterLimitKind::Depth,
            value: 65,
            maximum: 64,
        })
    ));
}

#[test]
fn test_small_generic_json_profile_covers_each_resource_boundary() {
    use qubit_budget::json::JsonDecodeLimits;
    use qubit_json::decode::JsonDecoder;

    fn accepts(input: &str, limits: JsonDecodeLimits) -> bool {
        JsonDecoder::with_limits(limits)
            .decode_utf8::<serde_json::Value>(input.as_bytes())
            .is_ok()
    }
    fn one_limit(
        build: impl FnOnce(qubit_budget::json::JsonDecodeLimitsBuilder) -> JsonDecodeLimits,
    ) -> JsonDecodeLimits {
        build(JsonDecodeLimits::builder())
    }
    let raw = one_limit(|b| b.max_input_bytes(1024).build());
    assert!(accepts(&format!("\"{}\"", "x".repeat(1022)), raw));
    assert!(!accepts(&format!("\"{}\"", "x".repeat(1023)), raw));

    let depth = one_limit(|b| b.max_depth(16).build());
    let nested_ok = format!("{}0{}", "[".repeat(15), "]".repeat(15));
    let nested_bad = format!("{}0{}", "[".repeat(16), "]".repeat(16));
    assert!(accepts(&nested_ok, depth));
    assert!(!accepts(&nested_bad, depth));

    let nodes = one_limit(|b| b.max_nodes(64).build());
    assert!(accepts(
        &format!("[{}]", (0..63).map(|_| "0").collect::<Vec<_>>().join(",")),
        nodes
    ));
    assert!(!accepts(
        &format!("[{}]", (0..64).map(|_| "0").collect::<Vec<_>>().join(",")),
        nodes
    ));
    let sequences = one_limit(|b| b.max_sequence_items(8).build());
    assert!(accepts("[0,1,2,3,4,5,6,7]", sequences));
    assert!(!accepts("[0,1,2,3,4,5,6,7,8]", sequences));
    let maps = one_limit(|b| b.max_map_entries(8).build());
    let map8 = format!(
        "{{{}}}",
        (0..8)
            .map(|i| format!("\"k{i}\":0"))
            .collect::<Vec<_>>()
            .join(",")
    );
    let map9 = format!(
        "{{{}}}",
        (0..9)
            .map(|i| format!("\"k{i}\":0"))
            .collect::<Vec<_>>()
            .join(",")
    );
    assert!(accepts(&map8, maps));
    assert!(!accepts(&map9, maps));
    let keys = one_limit(|b| b.max_key_bytes(32).build());
    assert!(accepts(&format!("{{\"{}\":0}}", "k".repeat(32)), keys));
    assert!(!accepts(&format!("{{\"{}\":0}}", "k".repeat(33)), keys));
    let strings = one_limit(|b| b.max_string_bytes(64).build());
    assert!(accepts(&format!("\"{}\"", "s".repeat(64)), strings));
    assert!(!accepts(&format!("\"{}\"", "s".repeat(65)), strings));
    let numbers = one_limit(|b| b.max_number_bytes(16).build());
    assert!(accepts("1234567890123456", numbers));
    assert!(!accepts("12345678901234567", numbers));
    let payload = one_limit(|b| b.max_payload_bytes(512).build());
    let payload_ok = format!(
        "[{}]",
        (0..8)
            .map(|_| format!("\"{}\"", "p".repeat(64)))
            .collect::<Vec<_>>()
            .join(",")
    );
    let payload_bad = format!(
        "[{}]",
        (0..9)
            .map(|_| format!("\"{}\"", "p".repeat(64)))
            .collect::<Vec<_>>()
            .join(",")
    );
    assert!(accepts(&payload_ok, payload));
    assert!(!accepts(&payload_bad, payload));
}
