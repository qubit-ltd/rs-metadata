// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Unit tests for [`qubit_metadata::Metadata`].

use std::collections::BTreeMap;

use qubit_datatype::ConversionLimits;
use qubit_datatype::ConversionPolicy;
use qubit_datatype::DataType;
#[cfg(feature = "filter")]
use qubit_metadata::FilterLimitKind;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataError;
#[cfg(feature = "schema")]
use qubit_metadata::MetadataSchema;
use qubit_metadata::MetadataWireLimitKind;
use qubit_value::IntoValueDefault;
use qubit_value::Value;
use qubit_value::ValueError;

mod support;

use support::port::Port;

#[test]
fn test_strict_get_and_explicit_conversion_have_distinct_semantics() {
    let metadata = Metadata::new().with("port", "42");
    let error = metadata.get::<i32>("port").unwrap_err();
    assert!(matches!(error, MetadataError::ValueAccess { source, .. }
        if matches!(*source, ValueError::TypeMismatch { .. })));
    assert_eq!(metadata.convert::<i32>("port").unwrap(), 42);
    assert_eq!(metadata.get::<String>("port").unwrap(), "42");
    assert_eq!(metadata.get_optional::<i32>("absent").unwrap(), None);
    assert_eq!(metadata.get_or::<i32>("absent", 7).unwrap(), 7);
}

#[test]
fn test_optional_and_default_reads_do_not_hide_type_errors() {
    let metadata = Metadata::new()
        .with("unset", Value::Unset(DataType::Int32))
        .with("invalid", "secret-invalid");
    assert_eq!(metadata.get_optional::<i32>("unset").unwrap(), None);
    assert!(metadata.get_optional::<i64>("unset").is_err());
    assert!(metadata.get_or::<i64>("unset", 7).is_err());
    let error = metadata
        .convert_or_with::<i32>(
            "invalid",
            7,
            ConversionPolicy::default_ref(),
            ConversionLimits::default_ref(),
        )
        .unwrap_err();
    assert!(std::error::Error::source(&error).is_some());
    assert!(!error.to_string().contains("secret-invalid"));
    let raw = metadata.get_raw("invalid").unwrap().get_ref::<str>().unwrap();
    assert_eq!(metadata.get_ref::<str>("invalid").unwrap().as_ptr(), raw.as_ptr());
}

#[test]
fn test_conversion_optional_uses_policy_and_defaults_are_lazy() {
    use std::cell::Cell;

    use qubit_datatype::BlankStringPolicy;
    use qubit_datatype::StringConversionPolicy;
    struct CountedDefault<'a>(&'a Cell<usize>);
    impl IntoValueDefault<i32> for CountedDefault<'_> {
        /// Records adaptation so successful reads and errors can prove
        /// laziness.
        fn into_value_default(self) -> i32 {
            self.0.set(self.0.get() + 1);
            7
        }
    }
    let policy = ConversionPolicy::builder()
        .string_policy(
            StringConversionPolicy::builder()
                .trim(true)
                .blank_string_policy(BlankStringPolicy::TreatAsMissing)
                .build(),
        )
        .build();
    let limits = ConversionLimits::default_ref();
    let metadata = Metadata::new()
        .with("blank", " ")
        .with("number", 42_i32)
        .with("invalid", "invalid");
    assert_eq!(metadata.get_optional::<String>("blank").unwrap(), Some(" ".to_owned()));
    assert_eq!(
        metadata.convert_optional_with::<i32>("blank", &policy, limits).unwrap(),
        None
    );
    let called = Cell::new(0);
    assert_eq!(metadata.get_or::<i32>("number", CountedDefault(&called)).unwrap(), 42);
    assert!(metadata.get_or::<i32>("invalid", CountedDefault(&called)).is_err());
    assert!(
        metadata
            .convert_or_with::<i32>("invalid", CountedDefault(&called), &policy, limits)
            .is_err()
    );
    assert_eq!(called.get(), 0);
    assert_eq!(
        metadata
            .convert_or_with::<i32>("blank", CountedDefault(&called), &policy, limits)
            .unwrap(),
        7
    );
    assert_eq!(called.get(), 1);
    assert_eq!(metadata.get_or::<String>("absent", "default").unwrap(), "default");
}

#[derive(Debug, Clone, Copy)]
struct TenantId(u64);

impl From<TenantId> for Value {
    /// Converts a tenant identifier into its metadata representation.
    fn from(value: TenantId) -> Self {
        Self::UInt64(value.0)
    }
}

#[test]
fn test_typed_reads_accept_downstream_conversion_targets() {
    let metadata = Metadata::new().with("port", Value::String("8080".to_owned()));
    assert_eq!(metadata.convert::<Port>("port"), Ok(Port(8080)));
}

#[test]
fn test_new_is_empty() {
    let meta = Metadata::new();
    assert!(meta.is_empty());
    assert_eq!(meta.len(), 0);
}

#[test]
fn test_validate_wire_contract_accepts_metadata_within_limits() {
    let metadata = Metadata::new().with("name", "alice");

    assert_eq!(metadata.validate_wire_contract(), Ok(()));
}

#[test]
fn test_validate_wire_contract_rejects_oversized_metadata_key() {
    let metadata = Metadata::new().with("x".repeat(257).as_str(), "value");

    assert_eq!(
        metadata.validate_wire_contract(),
        Err(MetadataError::WireLimitExceeded {
            kind: MetadataWireLimitKind::KeyBytes,
            value: 257,
            maximum: 256,
        })
    );
}

#[test]
fn test_default_is_empty() {
    let meta = Metadata::default();
    assert!(meta.is_empty());
}

#[test]
fn test_with_builds_metadata_fluently() {
    let meta = Metadata::new()
        .with("author", "alice")
        .with("priority", 42_i64)
        .with("reviewed", true);

    assert_eq!(meta.get::<String>("author").as_deref(), Ok("alice"));
    assert_eq!(meta.get::<i64>("priority"), Ok(42));
    assert_eq!(meta.get::<bool>("reviewed"), Ok(true));
}

#[test]
fn test_with_accepts_value_and_domain_newtype() {
    let metadata = Metadata::new()
        .with("raw", Value::String("stored".to_owned()))
        .with("tenant_id", TenantId(42));

    assert_eq!(metadata.get_raw("raw"), Some(&Value::String("stored".to_owned())));
    assert_eq!(metadata.get::<u64>("tenant_id"), Ok(42));
}

#[test]
fn test_set_and_get_scalar_values() {
    let mut meta = Metadata::new();
    meta.set("author", "alice");
    meta.set("priority", 42_i64);
    meta.set("reviewed", true);
    meta.set("score", std::f64::consts::PI);

    assert_eq!(meta.get::<String>("author").as_deref(), Ok("alice"));
    assert_eq!(meta.get::<i64>("priority"), Ok(42));
    assert_eq!(meta.get::<bool>("reviewed"), Ok(true));
    assert!((meta.get::<f64>("score").unwrap() - std::f64::consts::PI).abs() < 1e-10);
}

#[test]
fn test_insert_returns_previous_value() {
    let mut meta = Metadata::new();
    assert_eq!(meta.insert("key", "first"), None);
    let old = meta.insert("key", "second");

    assert_eq!(old, Some(Value::String("first".to_string())));
    assert_eq!(meta.get::<String>("key").as_deref(), Ok("second"));
}

#[test]
fn test_set_supports_mutable_chaining() {
    let mut meta = Metadata::new();
    meta.set("first", 1_i64).set("second", 2_i64);

    assert_eq!(meta.get::<i64>("first"), Ok(1));
    assert_eq!(meta.get::<i64>("second"), Ok(2));
}

#[test]
fn test_get_missing_key_returns_none() {
    let meta = Metadata::new();
    let value = meta.get_optional::<String>("missing").unwrap();
    assert!(value.is_none());
}

#[test]
fn test_get_wrong_type_returns_error() {
    let mut meta = Metadata::new();
    meta.set("key", "not-a-number");
    let value = meta.get::<i64>("key");
    assert!(value.is_err());
}

#[test]
fn test_get_str_borrows_stored_string() {
    let metadata = Metadata::new().with("name", "alice");

    assert_eq!(metadata.get_ref::<str>("name"), Ok("alice"));
    assert_eq!(metadata.get_ref::<str>("name"), Ok("alice"));
}

#[test]
fn test_try_get_str_reports_missing_unset_and_non_string_values() {
    let metadata = Metadata::new()
        .with("unset", Value::Unset(DataType::String))
        .with("count", 1_i64);

    assert!(matches!(
        metadata.get_ref::<str>("missing"),
        Err(MetadataError::MissingKey(key)) if key == "missing"
    ));
    assert!(matches!(
        metadata.get_ref::<str>("unset"),
        Err(MetadataError::ValueAccess { key, source })
            if key == "unset" && source.missing().unwrap().source_type() == Some(DataType::String)
    ));
    assert!(matches!(
        metadata.get_ref::<str>("count"),
        Err(MetadataError::ValueAccess { key, source })
            if key == "count" && matches!(*source, ValueError::TypeMismatch {
                expected: DataType::String, actual: DataType::Int64
            })
    ));
}

#[test]
fn test_try_get_missing_key_reports_error() {
    let meta = Metadata::new();
    let error = meta.convert::<String>("missing").unwrap_err();
    assert_eq!(error, MetadataError::MissingKey("missing".to_string()));
}

#[test]
fn test_try_get_unset_value_reports_missing_value() {
    let metadata = Metadata::new().with("count", Value::Unset(DataType::Int64));

    let error = metadata.convert::<i64>("count").unwrap_err();
    let MetadataError::ValueAccess { key, source } = error else {
        panic!("value access")
    };
    assert_eq!(key, "count");
    assert_eq!(source.missing().unwrap().source_type(), Some(DataType::Int64));
    assert_eq!(source.missing().unwrap().target_type(), Some(DataType::Int64));
}

#[test]
fn test_try_get_type_mismatch_reports_expected_and_actual_type() {
    let mut meta = Metadata::new();
    meta.set("key", "known-secret");

    let error = meta.convert::<i64>("key").unwrap_err();
    match error {
        MetadataError::ValueAccess { key, source } => {
            assert_eq!(key, "key");
            let ValueError::Conversion(conversion) = *source else {
                panic!("conversion error")
            };
            assert_eq!(conversion.to_type(), DataType::Int64);
            assert_eq!(conversion.from_type(), Some(DataType::String));
            assert!(!conversion.to_string().contains("known-secret"));
        }
        other => panic!("expected ValueAccess, got {other:?}"),
    }
}

#[test]
fn test_get_or_defaults_missing_keys_and_preserves_type_mismatch() {
    let mut meta = Metadata::new();
    meta.set("key", "text");

    assert_eq!(meta.get_or("missing", 42_i64), Ok(42));
    assert!(meta.get_or("key", 7_i64).is_err());
}

#[test]
#[cfg(feature = "schema")]
fn test_insert_checked_returns_previous_value() {
    let schema = MetadataSchema::builder()
        .required("key", DataType::String)
        .build()
        .expect("schema should build");
    let mut meta = Metadata::new();
    meta.insert_checked(&schema, "key", "first").unwrap();
    let old = meta.insert_checked(&schema, "key", "second").unwrap();
    assert_eq!(old, Some(Value::String("first".to_string())));
}

#[test]
#[cfg(feature = "schema")]
fn test_set_checked_supports_mutable_chaining() {
    let schema = MetadataSchema::builder()
        .required("first", DataType::Int64)
        .required("second", DataType::Int64)
        .build()
        .expect("schema should build");
    let mut meta = Metadata::new();
    meta.set_checked(&schema, "first", 1_i64)
        .unwrap()
        .set_checked(&schema, "second", 2_i64)
        .unwrap();

    assert_eq!(meta.get::<i64>("first"), Ok(1));
    assert_eq!(meta.get::<i64>("second"), Ok(2));
}

#[test]
#[cfg(feature = "schema")]
fn test_set_checked_rejects_type_mismatch() {
    let schema = MetadataSchema::builder()
        .required("key", DataType::String)
        .build()
        .expect("schema should build");
    let mut meta = Metadata::new();
    let error = meta.set_checked(&schema, "key", 1_i64).unwrap_err();

    match error {
        MetadataError::TypeMismatch {
            key, expected, actual, ..
        } => {
            assert_eq!(key, "key");
            assert_eq!(expected, DataType::String);
            assert_eq!(actual, DataType::Int64);
        }
        other => panic!("expected TypeMismatch, got {other:?}"),
    }
}

#[cfg(feature = "schema")]
#[test]
fn test_insert_checked_rejects_unset_required_field() {
    let schema = MetadataSchema::builder()
        .required("key", DataType::String)
        .build()
        .expect("schema should build");
    let mut metadata = Metadata::new();

    let error = metadata
        .insert_checked(&schema, "key", Value::Unset(DataType::String))
        .expect_err("required fields should reject unset values");

    assert_eq!(
        error,
        MetadataError::MissingRequiredField {
            key: "key".to_string(),
            expected: DataType::String,
        }
    );
    assert!(metadata.is_empty());
}

#[test]
#[cfg(feature = "schema")]
fn test_with_checked_rejects_unknown_field() {
    let schema = MetadataSchema::builder()
        .required("known", DataType::String)
        .build()
        .expect("schema should build");
    let error = Metadata::new().with_checked(&schema, "unknown", "value").unwrap_err();

    assert_eq!(
        error,
        MetadataError::UnknownField {
            key: "unknown".to_string(),
        }
    );
}

#[test]
#[cfg(feature = "schema")]
fn test_with_checked_accepts_schema_compatible_value() {
    let schema = MetadataSchema::builder()
        .required("known", DataType::String)
        .build()
        .expect("schema should build");
    let metadata = Metadata::new()
        .with_checked(&schema, "known", "value")
        .expect("compatible value should be accepted");

    assert_eq!(metadata.get_ref::<str>("known"), Ok("value"));
}

#[test]
fn test_get_raw_and_set_value_support_mutable_chaining() {
    let mut meta = Metadata::new();
    meta.set("raw", Value::String("stored".to_string()))
        .set("count", Value::Int64(7));

    assert_eq!(meta.get_raw("raw"), Some(&Value::String("stored".to_string())));
    assert_eq!(meta.get::<String>("raw").as_deref(), Ok("stored"));
    assert_eq!(meta.get::<i64>("count"), Ok(7));
}

#[test]
fn test_insert_value_returns_previous_value() {
    let mut meta = Metadata::new();
    assert_eq!(meta.insert("raw", Value::String("first".to_string())), None);
    assert_eq!(
        meta.insert("raw", Value::String("second".to_string())),
        Some(Value::String("first".to_string()))
    );
}

#[test]
fn test_with_value_builds_metadata_fluently() {
    let meta = Metadata::new().with("raw", Value::String("stored".to_string()));

    assert_eq!(meta.get_raw("raw"), Some(&Value::String("stored".to_string())));
}

#[test]
fn test_data_type_reports_value_data_type() {
    let mut meta = Metadata::new();
    meta.set("flag", true);
    meta.set("count", 7_i64);
    meta.set("name", "alice");

    assert_eq!(meta.data_type("flag"), Some(DataType::Bool));
    assert_eq!(meta.data_type("count"), Some(DataType::Int64));
    assert_eq!(meta.data_type("name"), Some(DataType::String));
    assert_eq!(meta.data_type("missing"), None);
}

#[test]
fn test_metadata_error_display_messages_are_human_readable() {
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

    #[cfg(feature = "filter")]
    {
        assert_eq!(
            MetadataError::InvalidFilterLimit {
                kind: FilterLimitKind::Depth,
                value: 0,
                maximum: 64,
            }
            .to_string(),
            "Metadata filter Depth limit 0 is outside 1..=64"
        );
        assert_eq!(
            MetadataError::FilterLimitExceeded {
                kind: FilterLimitKind::Nodes,
                value: 3,
                maximum: 2,
            }
            .to_string(),
            "Metadata filter Nodes value 3 exceeds the maximum of 2"
        );
    }

    let _error_ref: &dyn std::error::Error = &mismatch;
}

#[test]
fn test_contains_key_and_len_track_entries() {
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
fn test_remove_and_clear_work() {
    let mut meta = Metadata::new();
    meta.set("a", 1_i64);
    meta.set("b", 2_i64);

    assert_eq!(meta.remove("a"), Some(Value::Int64(1)));
    assert!(!meta.contains_key("a"));

    meta.clear();
    assert!(meta.is_empty());
}

#[test]
fn test_metadata_iterators_and_conversions_expose_owned_and_borrowed_entries() {
    let mut metadata = Metadata::new();
    metadata.extend([("b".to_string(), Value::Int64(2)), ("a".to_string(), Value::Int64(1))]);
    assert_eq!(metadata.keys().collect::<Vec<_>>(), vec!["a", "b"]);
    assert_eq!(metadata.values().count(), 2);
    assert_eq!(metadata.iter().count(), 2);
    assert_eq!((&metadata).into_iter().count(), 2);

    let map: BTreeMap<String, Value> = metadata.clone().into();
    assert_eq!(map.len(), 2);
    let round_trip = Metadata::from_iter(map);
    let owned: BTreeMap<String, Value> = round_trip.into_iter().collect();
    assert_eq!(owned.len(), 2);
}

#[cfg(feature = "json")]
#[test]
fn test_metadata_default_json_helpers_round_trip_and_write() {
    let metadata = Metadata::new().with("name", "alice");
    let encoded = metadata.to_json_vec().expect("default JSON encoding should succeed");
    let decoded = Metadata::decode_json_slice(&encoded).expect("default JSON decoding should succeed");
    assert_eq!(decoded, metadata);

    let mut output = Vec::new();
    metadata
        .to_json_writer(&mut output)
        .expect("default JSON writer should succeed");
    assert_eq!(
        Metadata::decode_json_slice(&output).expect("written JSON should decode"),
        metadata
    );
    assert!(format!("{metadata:?}").contains("name"));
    assert!(metadata.to_string().contains("name"));
}

#[test]
fn test_metadata_debug_clone_and_equality_are_stable() {
    let metadata = Metadata::new().with("id", 7_i64);
    assert_eq!(metadata.clone(), metadata);
    assert!(format!("{metadata:?}").contains("id"));
}

#[test]
fn test_iterators_return_sorted_entries() {
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
        vec![&Value::Int64(1), &Value::Bool(true), &Value::String("last".to_string())]
    );
}

#[test]
fn test_into_iter_consumes_metadata() {
    let mut meta = Metadata::new();
    meta.set("x", 10_i64);

    let pairs: Vec<(String, Value)> = meta.into_iter().collect();
    assert_eq!(pairs, vec![("x".to_string(), Value::Int64(10))]);
}

#[test]
fn test_ref_into_iter_counts_entries() {
    let mut meta = Metadata::new();
    meta.set("k", "v");
    assert_eq!((&meta).into_iter().count(), 1);
}

#[test]
fn test_merge_and_merged_work() {
    let mut a = Metadata::new();
    a.set("x", 1_i64);

    let mut b = Metadata::new();
    b.set("y", 2_i64);

    let c = a.merged(&b);
    assert_eq!(a.len(), 1);
    assert_eq!(c.len(), 2);

    a.merge(b);
    assert_eq!(a.get::<i64>("x"), Ok(1));
    assert_eq!(a.get::<i64>("y"), Ok(2));
}

#[test]
fn test_merge_overwrites_on_conflict() {
    let mut a = Metadata::new();
    a.set("k", "original");

    let mut b = Metadata::new();
    b.set("k", "overwritten");

    a.merge(b);
    assert_eq!(a.get::<String>("k").as_deref(), Ok("overwritten"));
}

#[test]
fn test_retain_keeps_matching_entries() {
    let mut meta = Metadata::new();
    meta.set("a", 1_i64);
    meta.set("b", 2_i64);
    meta.set("c", 3_i64);

    meta.retain(|key, _| key != "b");
    assert!(!meta.contains_key("b"));
    assert_eq!(meta.len(), 2);
}

#[test]
fn test_btreemap_conversions_work() {
    let mut map = BTreeMap::new();
    map.insert("k".to_string(), Value::String("v".to_string()));

    let meta = Metadata::from(map);
    assert_eq!(meta.get::<String>("k").as_deref(), Ok("v"));

    let map: BTreeMap<String, Value> = meta.into();
    assert_eq!(map.get("k"), Some(&Value::String("v".to_string())));
}

#[test]
fn test_into_inner_returns_underlying_map() {
    let mut meta = Metadata::new();
    meta.set("k", 1_i64);

    let inner = meta.into_inner();
    assert_eq!(inner.get("k"), Some(&Value::Int64(1)));
}

#[test]
fn test_from_iterator_and_extend_work() {
    let pairs = vec![("a".to_string(), Value::Int64(1)), ("b".to_string(), Value::Int64(2))];
    let mut meta: Metadata = pairs.into_iter().collect();

    meta.extend(vec![("c".to_string(), Value::Int64(3))]);
    assert_eq!(meta.len(), 3);
}

#[test]
fn test_serde_round_trip_uses_value_encoding() {
    let meta = Metadata::new()
        .with("name", "bob")
        .with("age", 30_i64)
        .with("active", true);

    let json_text = serde_json::to_string(&meta).unwrap();
    let restored: Metadata = serde_json::from_str(&json_text).unwrap();
    assert_eq!(meta, restored);
}

#[test]
fn test_clone_is_independent() {
    let mut original = Metadata::new();
    original.set("k", "v");

    let mut cloned = original.clone();
    cloned.set("k", "changed");

    assert_eq!(original.get::<String>("k").as_deref(), Ok("v"));
}

#[test]
fn test_partial_eq_compares_values() {
    let mut a = Metadata::new();
    a.set("x", 1_i64);

    let mut b = Metadata::new();
    b.set("x", 1_i64);
    assert_eq!(a, b);

    b.set("x", 2_i64);
    assert_ne!(a, b);
}

#[test]
fn test_strict_read_preserves_type_and_borrows_string() {
    let metadata = Metadata::new().with("port", "8080").with("count", 3_i64);
    assert_eq!(metadata.get::<i64>("count").expect("i64"), 3);
    let borrowed: &str = metadata.get_ref::<str>("port").expect("borrowed string");
    assert!(std::ptr::eq(
        borrowed.as_ptr(),
        metadata.get_ref::<str>("port").expect("string").as_ptr()
    ));
    let error = metadata.get::<i64>("port").expect_err("no implicit parsing");
    assert!(matches!(error, MetadataError::ValueAccess { source, .. }
        if matches!(*source, ValueError::TypeMismatch { .. })));
    assert_eq!(metadata.convert::<i64>("port").expect("legacy conversion"), 8080);
}

#[test]
fn test_explicit_conversion_preserves_source_and_policy() {
    use std::error::Error;

    use qubit_datatype::ConversionLimits;
    use qubit_datatype::ConversionPolicy;
    use qubit_datatype::NumericConversionPolicy;
    let metadata = Metadata::new().with("score", 1.5_f64);
    let error = metadata.convert::<i32>("score").expect_err("exact conversion");
    assert!(error.source().expect("value source").is::<ValueError>());
    assert!(matches!(error, MetadataError::ValueAccess { source, .. }
        if matches!(*source, ValueError::Conversion(_))));
    let policy = ConversionPolicy::builder()
        .numeric_policy(NumericConversionPolicy::lossy())
        .build();
    assert_eq!(
        metadata
            .convert_with::<i32>("score", &policy, ConversionLimits::default_ref())
            .expect("lossy"),
        1
    );
}

#[test]
fn test_explicit_reads_distinguish_absent_and_unset() {
    let metadata = Metadata::new().with("unset", Value::Unset(DataType::Int64));
    assert_eq!(
        metadata.get::<i64>("missing"),
        Err(MetadataError::MissingKey("missing".into()))
    );
    for error in [
        metadata.convert::<i64>("unset").unwrap_err(),
        metadata.get::<i64>("unset").unwrap_err(),
    ] {
        let MetadataError::ValueAccess { key, source } = error else {
            panic!("value access")
        };
        assert_eq!(key, "unset");
        let missing = source.missing().unwrap();
        assert!(missing.is_unset());
        assert_eq!(missing.source_type(), Some(DataType::Int64));
        assert_eq!(missing.target_type(), Some(DataType::Int64));
    }
}

#[test]
fn test_explicit_conversion_retains_budget_failure() {
    use qubit_datatype::ConversionLimits;
    use qubit_datatype::ConversionPolicy;
    use qubit_datatype::NumericConversionLimits;
    let limits = ConversionLimits::builder()
        .numeric_limits(NumericConversionLimits::builder().max_text_bytes(2).build())
        .build();
    let metadata = Metadata::new().with("number", "12345");
    let expected = metadata
        .get_raw("number")
        .expect("value")
        .to_with::<i64>(ConversionPolicy::default_ref(), &limits)
        .expect_err("numeric text limit");
    let actual = metadata
        .convert_with::<i64>("number", ConversionPolicy::default_ref(), &limits)
        .expect_err("same numeric text limit");
    assert_eq!(
        actual,
        MetadataError::ValueAccess {
            key: "number".into(),
            source: Box::new(expected)
        }
    );
}
