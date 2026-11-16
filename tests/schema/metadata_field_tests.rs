// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::MetadataField`].

use qubit_datatype::DataType;
use qubit_metadata::MetadataField;
use serde_json::from_str;
use serde_json::from_value;
use serde_json::json;
use serde_json::to_string;
use serde_json::to_value;

#[test]
fn test_metadata_field_new_assigns_values() {
    let field = MetadataField::new(DataType::String, true);
    assert_eq!(field.data_type(), DataType::String);
    assert!(field.is_required());
}

#[test]
fn test_metadata_field_rejects_positional_array() {
    let field = MetadataField::new(DataType::String, true);
    let object = to_value(&field).expect("field should serialize");
    let positional = json!([object["data_type"], object["required"]]);
    let encoded = to_string(&positional).expect("positional field should serialize");

    assert!(
        from_value::<MetadataField>(positional).is_err(),
        "MetadataField should reject positional input from_value"
    );
    assert!(
        from_str::<MetadataField>(&encoded).is_err(),
        "MetadataField should reject positional input from_str"
    );
}

#[test]
fn test_metadata_field_requires_complete_known_object_fields() {
    let field = MetadataField::new(DataType::String, true);
    let object = to_value(&field).expect("field should serialize");
    let invalid_values = [
        ("missing data_type", json!({"required": true})),
        (
            "missing required",
            json!({"data_type": object["data_type"]}),
        ),
        (
            "unknown field",
            json!({"data_type": object["data_type"], "required": true, "extra": false}),
        ),
        ("null", json!(null)),
        ("scalar", json!(true)),
        ("empty array", json!([])),
        ("short array", json!([object["data_type"]])),
        (
            "long array",
            json!([object["data_type"], object["required"], "extra"]),
        ),
    ];

    for (shape, value) in invalid_values {
        let encoded = to_string(&value).expect("invalid field shape should serialize");
        assert!(
            from_value::<MetadataField>(value).is_err(),
            "MetadataField should reject {shape} from_value"
        );
        assert!(
            from_str::<MetadataField>(&encoded).is_err(),
            "MetadataField should reject {shape} from_str"
        );
    }

    let duplicate = format!(
        r#"{{"data_type":{},"data_type":{},"required":true}}"#,
        object["data_type"], object["data_type"]
    );
    assert!(
        from_str::<MetadataField>(&duplicate).is_err(),
        "MetadataField should reject duplicate data_type fields"
    );

    let duplicate_required = format!(
        r#"{{"data_type":{},"required":true,"required":false}}"#,
        object["data_type"]
    );
    assert!(
        from_str::<MetadataField>(&duplicate_required).is_err(),
        "MetadataField should reject duplicate required fields"
    );

    let reordered = format!(r#"{{"required":true,"data_type":{}}}"#, object["data_type"]);
    assert_eq!(
        from_str::<MetadataField>(&reordered).expect("field object order should be flexible"),
        field
    );
}

#[test]
fn test_metadata_field_rejects_unknown_key_before_reading_value() {
    let inputs = [
        r#"{"extra": ["#.to_owned(),
        format!(r#"{{"extra": {}0{}}}"#, "[".repeat(256), "]".repeat(256)),
    ];

    for input in inputs {
        let error = from_str::<MetadataField>(&input).expect_err("unknown key must fail");
        assert!(error.to_string().contains("unknown field"), "{error}");
    }
}
