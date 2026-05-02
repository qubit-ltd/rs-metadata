/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Unit tests for [`qubit_metadata::Metadata`].

use std::collections::BTreeMap;

use qubit_datatype::DataType;
use qubit_metadata::{Metadata, MetadataError, MetadataSchema};
use qubit_value::Value;
use serde_json::json;

#[test]
fn new_is_empty() {
    let meta = Metadata::new();
    assert!(meta.is_empty());
    assert_eq!(meta.len(), 0);
}

#[test]
fn default_is_empty() {
    let meta = Metadata::default();
    assert!(meta.is_empty());
}

#[test]
fn with_builds_metadata_fluently() {
    let meta = Metadata::new()
        .with("author", "alice")
        .with("priority", 42_i64)
        .with("reviewed", true);

    assert_eq!(meta.get::<String>("author").as_deref(), Some("alice"));
    assert_eq!(meta.get::<i64>("priority"), Some(42));
    assert_eq!(meta.get::<bool>("reviewed"), Some(true));
}

#[test]
fn set_and_get_scalar_values() {
    let mut meta = Metadata::new();
    meta.set("author", "alice");
    meta.set("priority", 42_i64);
    meta.set("reviewed", true);
    meta.set("score", std::f64::consts::PI);

    assert_eq!(meta.get::<String>("author").as_deref(), Some("alice"));
    assert_eq!(meta.get::<i64>("priority"), Some(42));
    assert_eq!(meta.get::<bool>("reviewed"), Some(true));
    assert!((meta.get::<f64>("score").unwrap() - std::f64::consts::PI).abs() < 1e-10);
}

#[test]
fn set_overwrites_previous_value() {
    let mut meta = Metadata::new();
    meta.set("key", "first");
    let old = meta.set("key", "second");

    assert_eq!(old, Some(Value::String("first".to_string())));
    assert_eq!(meta.get::<String>("key").as_deref(), Some("second"));
}

#[test]
fn get_missing_key_returns_none() {
    let meta = Metadata::new();
    let value: Option<String> = meta.get("missing");
    assert!(value.is_none());
}

#[test]
fn get_wrong_type_returns_none() {
    let mut meta = Metadata::new();
    meta.set("key", "not-a-number");
    let value: Option<i64> = meta.get("key");
    assert!(value.is_none());
}

#[test]
fn try_get_missing_key_reports_error() {
    let meta = Metadata::new();
    let error = meta.try_get::<String>("missing").unwrap_err();
    assert_eq!(error, MetadataError::MissingKey("missing".to_string()));
}

#[test]
fn try_get_type_mismatch_reports_expected_and_actual_type() {
    let mut meta = Metadata::new();
    meta.set("key", "not-a-number");

    let error = meta.try_get::<i64>("key").unwrap_err();
    match error {
        MetadataError::TypeMismatch {
            key,
            expected,
            actual,
            message,
        } => {
            assert_eq!(key, "key");
            assert_eq!(expected, DataType::Int64);
            assert_eq!(actual, DataType::String);
            assert!(!message.is_empty());
        }
        other => panic!("expected TypeMismatch, got {other:?}"),
    }
}

#[test]
fn get_or_returns_default_for_missing_key_or_type_mismatch() {
    let mut meta = Metadata::new();
    meta.set("key", "text");

    assert_eq!(meta.get_or("missing", 42_i64), 42);
    assert_eq!(meta.get_or("key", 7_i64), 7);
}

#[test]
fn set_checked_returns_previous_value() {
    let schema = MetadataSchema::builder()
        .required("key", DataType::String)
        .build();
    let mut meta = Metadata::new();
    meta.set_checked(&schema, "key", "first").unwrap();
    let old = meta.set_checked(&schema, "key", "second").unwrap();
    assert_eq!(old, Some(Value::String("first".to_string())));
}

#[test]
fn set_checked_rejects_type_mismatch() {
    let schema = MetadataSchema::builder()
        .required("key", DataType::String)
        .build();
    let mut meta = Metadata::new();
    let error = meta.set_checked(&schema, "key", 1_i64).unwrap_err();

    match error {
        MetadataError::TypeMismatch {
            key,
            expected,
            actual,
            ..
        } => {
            assert_eq!(key, "key");
            assert_eq!(expected, DataType::String);
            assert_eq!(actual, DataType::Int64);
        }
        other => panic!("expected TypeMismatch, got {other:?}"),
    }
}

#[test]
fn with_checked_rejects_unknown_field() {
    let schema = MetadataSchema::builder()
        .required("known", DataType::String)
        .build();
    let error = Metadata::new()
        .with_checked(&schema, "unknown", "value")
        .unwrap_err();

    assert_eq!(
        error,
        MetadataError::UnknownField {
            key: "unknown".to_string(),
        }
    );
}

#[test]
fn get_raw_and_set_raw_use_qubit_value() {
    let mut meta = Metadata::new();
    meta.set_raw("raw", Value::Json(json!({"nested": true})));

    assert_eq!(
        meta.get_raw("raw"),
        Some(&Value::Json(json!({"nested": true})))
    );
    assert_eq!(
        meta.get::<serde_json::Value>("raw"),
        Some(json!({"nested": true}))
    );
}

#[test]
fn with_raw_builds_metadata_fluently() {
    let meta = Metadata::new().with_raw("raw", Value::Json(json!({"nested": true})));

    assert_eq!(
        meta.get_raw("raw"),
        Some(&Value::Json(json!({"nested": true})))
    );
}

#[test]
fn data_type_reports_value_data_type() {
    let mut meta = Metadata::new();
    meta.set("flag", true);
    meta.set("count", 7_i64);
    meta.set("name", "alice");
    meta.set_raw("payload", Value::Json(json!({"nested": true})));

    assert_eq!(meta.data_type("flag"), Some(DataType::Bool));
    assert_eq!(meta.data_type("count"), Some(DataType::Int64));
    assert_eq!(meta.data_type("name"), Some(DataType::String));
    assert_eq!(meta.data_type("payload"), Some(DataType::Json));
    assert_eq!(meta.data_type("missing"), None);
}

#[test]
fn metadata_error_display_messages_are_human_readable() {
    let missing = MetadataError::MissingKey("missing".to_string());
    assert_eq!(missing.to_string(), "Metadata key not found: missing");

    let mismatch = MetadataError::TypeMismatch {
        key: "answer".to_string(),
        expected: DataType::Int64,
        actual: DataType::String,
        message: "invalid type".to_string(),
    };
    assert_eq!(
        mismatch.to_string(),
        "Metadata key 'answer' expected int64 but actual string: invalid type"
    );

    let _error_ref: &dyn std::error::Error = &mismatch;
}

#[test]
fn contains_key_and_len_track_entries() {
    let mut meta = Metadata::new();
    assert!(!meta.contains_key("k"));
    assert_eq!(meta.len(), 0);

    meta.set("k", "v");
    assert!(meta.contains_key("k"));
    assert_eq!(meta.len(), 1);

    meta.set("k", "new");
    assert_eq!(meta.len(), 1);
}

#[test]
fn remove_and_clear_work() {
    let mut meta = Metadata::new();
    meta.set("a", 1_i64);
    meta.set("b", 2_i64);

    assert_eq!(meta.remove("a"), Some(Value::Int64(1)));
    assert!(!meta.contains_key("a"));

    meta.clear();
    assert!(meta.is_empty());
}

#[test]
fn iterators_return_sorted_entries() {
    let mut meta = Metadata::new();
    meta.set("z", "last");
    meta.set("a", 1_i64);
    meta.set("m", true);

    let keys: Vec<&str> = meta.iter().map(|(key, _)| key).collect();
    assert_eq!(keys, vec!["a", "m", "z"]);

    let keys: Vec<&str> = meta.keys().collect();
    assert_eq!(keys, vec!["a", "m", "z"]);

    let values: Vec<&Value> = meta.values().collect();
    assert_eq!(
        values,
        vec![
            &Value::Int64(1),
            &Value::Bool(true),
            &Value::String("last".to_string())
        ]
    );
}

#[test]
fn into_iter_consumes_metadata() {
    let mut meta = Metadata::new();
    meta.set("x", 10_i64);

    let pairs: Vec<(String, Value)> = meta.into_iter().collect();
    assert_eq!(pairs, vec![("x".to_string(), Value::Int64(10))]);
}

#[test]
fn ref_into_iter_counts_entries() {
    let mut meta = Metadata::new();
    meta.set("k", "v");
    assert_eq!((&meta).into_iter().count(), 1);
}

#[test]
fn merge_and_merged_work() {
    let mut a = Metadata::new();
    a.set("x", 1_i64);

    let mut b = Metadata::new();
    b.set("y", 2_i64);

    let c = a.merged(&b);
    assert_eq!(a.len(), 1);
    assert_eq!(c.len(), 2);

    a.merge(b);
    assert_eq!(a.get::<i64>("x"), Some(1));
    assert_eq!(a.get::<i64>("y"), Some(2));
}

#[test]
fn merge_overwrites_on_conflict() {
    let mut a = Metadata::new();
    a.set("k", "original");

    let mut b = Metadata::new();
    b.set("k", "overwritten");

    a.merge(b);
    assert_eq!(a.get::<String>("k").as_deref(), Some("overwritten"));
}

#[test]
fn retain_keeps_matching_entries() {
    let mut meta = Metadata::new();
    meta.set("a", 1_i64);
    meta.set("b", 2_i64);
    meta.set("c", 3_i64);

    meta.retain(|key, _| key != "b");
    assert!(!meta.contains_key("b"));
    assert_eq!(meta.len(), 2);
}

#[test]
fn btreemap_conversions_work() {
    let mut map = BTreeMap::new();
    map.insert("k".to_string(), Value::String("v".to_string()));

    let meta = Metadata::from(map);
    assert_eq!(meta.get::<String>("k").as_deref(), Some("v"));

    let map: BTreeMap<String, Value> = meta.into();
    assert_eq!(map.get("k"), Some(&Value::String("v".to_string())));
}

#[test]
fn into_inner_returns_underlying_map() {
    let mut meta = Metadata::new();
    meta.set("k", 1_i64);

    let inner = meta.into_inner();
    assert_eq!(inner.get("k"), Some(&Value::Int64(1)));
}

#[test]
fn from_iterator_and_extend_work() {
    let pairs = vec![
        ("a".to_string(), Value::Int64(1)),
        ("b".to_string(), Value::Int64(2)),
    ];
    let mut meta: Metadata = pairs.into_iter().collect();

    meta.extend(vec![("c".to_string(), Value::Int64(3))]);
    assert_eq!(meta.len(), 3);
}

#[test]
fn serde_round_trip_uses_value_encoding() {
    let meta = Metadata::new()
        .with("name", "bob")
        .with("age", 30_i64)
        .with("active", true);

    let json_text = serde_json::to_string(&meta).unwrap();
    let restored: Metadata = serde_json::from_str(&json_text).unwrap();
    assert_eq!(meta, restored);
}

#[test]
fn clone_is_independent() {
    let mut original = Metadata::new();
    original.set("k", "v");

    let mut cloned = original.clone();
    cloned.set("k", "changed");

    assert_eq!(original.get::<String>("k").as_deref(), Some("v"));
}

#[test]
fn partial_eq_compares_values() {
    let mut a = Metadata::new();
    a.set("x", 1_i64);

    let mut b = Metadata::new();
    b.set("x", 1_i64);
    assert_eq!(a, b);

    b.set("x", 2_i64);
    assert_ne!(a, b);
}
