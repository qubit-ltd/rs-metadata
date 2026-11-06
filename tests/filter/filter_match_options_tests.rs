// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::FilterMatchOptions`].

use qubit_datatype::NumericComparisonPolicy;
use qubit_metadata::FilterMatchOptions;

#[test]
fn test_default_options_use_exact_numeric_comparison() {
    let options = FilterMatchOptions::default();
    assert_eq!(options.numeric_comparison_policy(), NumericComparisonPolicy::Exact);
}

#[test]
fn test_numeric_comparison_policy_builder_supports_owned_chaining() {
    let options = FilterMatchOptions::builder()
        .numeric_comparison_policy(NumericComparisonPolicy::Approximate)
        .build();

    assert_eq!(
        options.numeric_comparison_policy(),
        NumericComparisonPolicy::Approximate
    );
}

#[test]
fn test_filter_match_options_serde_accepts_only_strict_objects() {
    for policy in ["exact", "approximate"] {
        let json = format!(r#"{{"numeric_comparison_policy":"{policy}"}}"#);
        let options: FilterMatchOptions = serde_json::from_str(&json).unwrap();
        let from_value = serde_json::from_value::<FilterMatchOptions>(serde_json::from_str(&json).unwrap()).unwrap();
        assert_eq!(from_value, options);
        let encoded = serde_json::to_value(options).unwrap();
        assert_eq!(encoded, serde_json::from_str::<serde_json::Value>(&json).unwrap());
    }

    for json in [
        r#"["exact"]"#,
        "{}",
        r#"{"extra":true,"numeric_comparison_policy":"exact"}"#,
        r#"{"numeric_comparison_policy":"invalid"}"#,
    ] {
        assert!(serde_json::from_str::<FilterMatchOptions>(json).is_err(), "{json}");
        let value = serde_json::from_str::<serde_json::Value>(json).unwrap();
        assert!(serde_json::from_value::<FilterMatchOptions>(value).is_err(), "{json}");
    }

    assert!(
        serde_json::from_str::<FilterMatchOptions>(
            r#"{"numeric_comparison_policy":"exact","numeric_comparison_policy":"approximate"}"#,
        )
        .is_err()
    );
}
