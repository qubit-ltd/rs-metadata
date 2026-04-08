/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Tests for [`qubit_metadata::MetadataValueType`].

use serde_json::{
    json,
    Value,
};

use qubit_metadata::MetadataValueType;

#[test]
fn of_classifies_all_json_variants() {
    assert_eq!(MetadataValueType::of(&Value::Null), MetadataValueType::Null);
    assert_eq!(
        MetadataValueType::of(&json!(false)),
        MetadataValueType::Bool
    );
    assert_eq!(MetadataValueType::of(&json!(0)), MetadataValueType::Number);
    assert_eq!(
        MetadataValueType::of(&json!(-1.5)),
        MetadataValueType::Number
    );
    assert_eq!(
        MetadataValueType::of(&json!("x")),
        MetadataValueType::String
    );
    assert_eq!(MetadataValueType::of(&json!([])), MetadataValueType::Array);
    assert_eq!(
        MetadataValueType::of(&json!([1, 2])),
        MetadataValueType::Array
    );
    assert_eq!(MetadataValueType::of(&json!({})), MetadataValueType::Object);
    assert_eq!(
        MetadataValueType::of(&json!({"a": 1})),
        MetadataValueType::Object
    );
}

#[test]
fn from_matches_of() {
    let v = json!({"nested": [true, null]});
    assert_eq!(MetadataValueType::from(&v), MetadataValueType::of(&v));
}

#[test]
fn display_uses_lowercase_json_type_names() {
    assert_eq!(MetadataValueType::Null.to_string(), "null");
    assert_eq!(MetadataValueType::Bool.to_string(), "bool");
    assert_eq!(MetadataValueType::Number.to_string(), "number");
    assert_eq!(MetadataValueType::String.to_string(), "string");
    assert_eq!(MetadataValueType::Array.to_string(), "array");
    assert_eq!(MetadataValueType::Object.to_string(), "object");
}

#[test]
fn equality_and_hash_consistent() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{
        Hash,
        Hasher,
    };

    let a = MetadataValueType::String;
    let b = MetadataValueType::String;
    assert_eq!(a, b);

    let mut h1 = DefaultHasher::new();
    let mut h2 = DefaultHasher::new();
    a.hash(&mut h1);
    b.hash(&mut h2);
    assert_eq!(h1.finish(), h2.finish());
}
