// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for metadata schema data types.

use qubit_datatype::DataType;
#[cfg(feature = "big-decimal")]
use qubit_datatype::NumericComparisonPolicy;
use qubit_metadata::{
    Metadata,
    MetadataError,
    MetadataFilter,
    MetadataSchema,
    MetadataValidationError,
    UnknownFieldPolicy,
};
use qubit_value::Value;
use serde_json::json;

fn single_issue(error: MetadataValidationError) -> MetadataError {
    let mut issues = error.into_issues();
    assert_eq!(issues.len(), 1);
    issues.remove(0)
}

fn equality_filter_with_value(value: &Value) -> MetadataFilter {
    serde_json::from_value(json!({
        "version": 3,
        "expression": {
            "type": "condition",
            "condition": {
                "op": "eq",
                "key": "field",
                "value": serde_json::to_value(value).unwrap(),
            },
        },
    }))
    .unwrap()
}

#[test]
fn test_schema_validate_filter_uses_state_aware_numeric_classification() {
    macro_rules! assert_numeric_compatible {
        ($value:expr) => {{
            let value = $value;
            let data_type = Value::new(value.clone()).data_type();
            let schema = MetadataSchema::builder()
                .required("field", data_type)
                .build();
            let filter = MetadataFilter::builder()
                .eq("field", value)
                .build()
                .unwrap();
            assert!(
                schema.validate_filter(&filter).is_ok(),
                "numeric value should match its declared type: {data_type}",
            );
        }};
    }

    assert_numeric_compatible!(1_i8);
    assert_numeric_compatible!(1_i16);
    assert_numeric_compatible!(1_i32);
    assert_numeric_compatible!(1_i64);
    assert_numeric_compatible!(1_i128);
    assert_numeric_compatible!(1_u8);
    assert_numeric_compatible!(1_u16);
    assert_numeric_compatible!(1_u32);
    assert_numeric_compatible!(1_u64);
    assert_numeric_compatible!(1_u128);
    assert_numeric_compatible!(1.0_f32);
    assert_numeric_compatible!(1.0_f64);
    #[cfg(feature = "big-integer")]
    assert_numeric_compatible!(num_bigint::BigInt::from(1));
    #[cfg(feature = "big-decimal")]
    assert_numeric_compatible!(bigdecimal::BigDecimal::from(1));

    let schema = MetadataSchema::builder()
        .required("field", DataType::Int64)
        .build();
    assert!(
        schema
            .validate_filter(&equality_filter_with_value(&Value::Unset(
                DataType::Int64,
            )))
            .is_err(),
        "a typed unset value is not a numeric literal",
    );

    let non_numeric =
        MetadataFilter::builder().eq("field", "1").build().unwrap();
    assert!(
        schema.validate_filter(&non_numeric).is_err(),
        "a non-numeric value must not match a numeric field",
    );
}

#[test]
fn test_schema_validate_filter_accepts_numeric_literal_compatibility() {
    let schema = MetadataSchema::builder()
        .required("score", DataType::Int64)
        .build();
    let filter = MetadataFilter::builder()
        .ge("score", 10)
        .build_checked(&schema)
        .unwrap();
    let meta = Metadata::new().with("score", 42_i64);

    assert!(filter.matches(&meta));
}

#[test]
fn test_schema_validate_filter_classifies_every_data_type_for_ranges() {
    for data_type in DataType::ALL {
        let schema = MetadataSchema::builder()
            .required("field", data_type)
            .build();
        let result = if data_type == DataType::String {
            MetadataFilter::builder()
                .gt("field", "lower")
                .build_checked(&schema)
        } else {
            MetadataFilter::builder()
                .gt("field", 1_i64)
                .build_checked(&schema)
        };

        assert_eq!(
            result.is_ok(),
            data_type.is_numeric() || data_type == DataType::String,
            "unexpected range classification for {data_type}"
        );
    }
}

#[test]
#[cfg(feature = "big-number")]
fn test_schema_validate_filter_accepts_big_number_fields() {
    let schema = MetadataSchema::builder()
        .required("amount", DataType::BigDecimal)
        .required("count", DataType::BigInteger)
        .build();

    let filter = MetadataFilter::builder()
        .ge("amount", bigdecimal::BigDecimal::from(10_i64))
        .and_eq("count", num_bigint::BigInt::from(3_i64))
        .build_checked(&schema)
        .unwrap();

    let meta = Metadata::new()
        .with("amount", bigdecimal::BigDecimal::from(11_i64))
        .with("count", num_bigint::BigInt::from(3_i64));
    assert!(filter.matches(&meta));
}

#[test]
fn test_schema_validate_filter_visits_all_condition_kinds() {
    let schema = MetadataSchema::builder()
        .required("score", DataType::Int64)
        .required("status", DataType::String)
        .optional("tag", DataType::String)
        .build();

    let filter = MetadataFilter::builder()
        .lt("score", 100_i64)
        .and_le("score", 100_i64)
        .and_gt("score", 1_i64)
        .and_in_set("status", ["active", "pending"])
        .and_not_in_set("tag", ["archived"])
        .and_exists("status")
        .and_not_exists("tag")
        .not()
        .build_checked(&schema)
        .unwrap();

    schema.validate_filter(&filter).unwrap();
    schema.validate_filter(&MetadataFilter::none()).unwrap();
}

#[test]
fn test_schema_validate_filter_accepts_not_equal_condition() {
    let schema = MetadataSchema::builder()
        .required("status", DataType::String)
        .build();

    MetadataFilter::builder()
        .ne("status", "inactive")
        .build_checked(&schema)
        .unwrap();
}

#[test]
fn test_schema_validate_filter_reports_ne_operator_for_incompatible_value() {
    let schema = MetadataSchema::builder()
        .required("status", DataType::String)
        .build();
    let error = single_issue(
        MetadataFilter::builder()
            .ne("status", 1_i64)
            .build_checked(&schema)
            .unwrap_err(),
    );

    match error {
        MetadataError::InvalidFilterOperator {
            key,
            operator,
            data_type,
            message,
        } => {
            assert_eq!(key, "status");
            assert_eq!(operator, "ne");
            assert_eq!(data_type, DataType::String);
            assert!(message.contains("not compatible"));
        }
        other => panic!("expected InvalidFilterOperator, got {other:?}"),
    }
}

#[test]
fn test_schema_validate_filter_rejects_incompatible_value_predicate() {
    let schema = MetadataSchema::builder()
        .required("status", DataType::String)
        .build();
    let error = single_issue(
        MetadataFilter::builder()
            .eq("status", 1_i64)
            .build_checked(&schema)
            .unwrap_err(),
    );

    match error {
        MetadataError::InvalidFilterOperator {
            key,
            operator,
            data_type,
            message,
        } => {
            assert_eq!(key, "status");
            assert_eq!(operator, "eq");
            assert_eq!(data_type, DataType::String);
            assert!(message.contains("not compatible"));
        }
        other => panic!("expected InvalidFilterOperator, got {other:?}"),
    }
}

#[test]
fn test_schema_validate_filter_rejects_unknown_value_predicate_field() {
    let schema = MetadataSchema::builder()
        .required("status", DataType::String)
        .build();
    let error = single_issue(
        MetadataFilter::builder()
            .eq("missing", "active")
            .build_checked(&schema)
            .unwrap_err(),
    );

    assert_eq!(
        error,
        MetadataError::UnknownFilterField {
            key: "missing".to_string(),
        }
    );
}

#[test]
fn test_schema_validate_filter_allows_unknown_fields_when_policy_allows_them() {
    let schema = MetadataSchema::builder()
        .required("status", DataType::String)
        .unknown_field_policy(UnknownFieldPolicy::Allow)
        .build();

    MetadataFilter::builder()
        .eq("dynamic", "value")
        .and_ge("dynamic_score", 10_i64)
        .and_exists("dynamic_flag")
        .build_checked(&schema)
        .unwrap();
}

#[test]
fn test_schema_validate_filter_still_validates_declared_fields_when_unknowns_are_allowed()
 {
    let schema = MetadataSchema::builder()
        .required("status", DataType::String)
        .unknown_field_policy(UnknownFieldPolicy::Allow)
        .build();
    let error = single_issue(
        MetadataFilter::builder()
            .eq("status", 1_i64)
            .and_eq("dynamic", "value")
            .build_checked(&schema)
            .unwrap_err(),
    );

    match error {
        MetadataError::InvalidFilterOperator {
            key,
            operator,
            data_type,
            message,
        } => {
            assert_eq!(key, "status");
            assert_eq!(operator, "eq");
            assert_eq!(data_type, DataType::String);
            assert!(message.contains("not compatible"));
        }
        other => panic!("expected InvalidFilterOperator, got {other:?}"),
    }
}

#[test]
fn test_schema_validate_filter_rejects_incompatible_not_in_value() {
    let schema = MetadataSchema::builder()
        .required("status", DataType::String)
        .build();
    let error = single_issue(
        MetadataFilter::builder()
            .not_in_set("status", [1_i64])
            .build_checked(&schema)
            .unwrap_err(),
    );

    match error {
        MetadataError::InvalidFilterOperator {
            key,
            operator,
            data_type,
            message,
        } => {
            assert_eq!(key, "status");
            assert_eq!(operator, "not_in_set");
            assert_eq!(data_type, DataType::String);
            assert!(message.contains("not compatible"));
        }
        other => panic!("expected InvalidFilterOperator, got {other:?}"),
    }
}

#[test]
fn test_schema_validate_filter_rejects_unknown_exists_field() {
    let schema = MetadataSchema::builder()
        .required("status", DataType::String)
        .build();
    let error = single_issue(
        MetadataFilter::builder()
            .exists("missing")
            .build_checked(&schema)
            .unwrap_err(),
    );

    assert_eq!(
        error,
        MetadataError::UnknownFilterField {
            key: "missing".to_string(),
        }
    );
}

#[test]
fn test_schema_validate_filter_rejects_incompatible_range_value() {
    let schema = MetadataSchema::builder()
        .required("status", DataType::String)
        .build();
    let error = single_issue(
        MetadataFilter::builder()
            .gt("status", 1_i64)
            .build_checked(&schema)
            .unwrap_err(),
    );

    match error {
        MetadataError::InvalidFilterOperator {
            key,
            operator,
            data_type,
            message,
        } => {
            assert_eq!(key, "status");
            assert_eq!(operator, "gt");
            assert_eq!(data_type, DataType::String);
            assert!(message.contains("not compatible"));
        }
        other => panic!("expected InvalidFilterOperator, got {other:?}"),
    }
}

#[test]
fn test_schema_validate_filter_reports_unknown_field() {
    let schema = MetadataSchema::builder()
        .required("score", DataType::Int64)
        .build();
    let error = single_issue(
        MetadataFilter::builder()
            .ge("unknown", 10_i64)
            .build_checked(&schema)
            .unwrap_err(),
    );

    assert_eq!(
        error,
        MetadataError::UnknownFilterField {
            key: "unknown".to_string(),
        }
    );
}

#[test]
fn test_schema_validate_filter_collects_all_condition_issues() {
    let schema = MetadataSchema::builder()
        .required("status", DataType::String)
        .required("score", DataType::Int64)
        .required("active", DataType::Bool)
        .build();

    let error = MetadataFilter::builder()
        .eq("status", 1_i64)
        .and_ge("missing_score", 10_i64)
        .and_gt("active", true)
        .build_checked(&schema)
        .unwrap_err();
    let issues = error.issues();

    assert_eq!(issues.len(), 3);
    assert!(matches!(
        &issues[0],
        MetadataError::InvalidFilterOperator { key, operator, .. }
            if key == "status" && operator == &"eq"
    ));
    assert!(matches!(
        &issues[1],
        MetadataError::UnknownFilterField { key } if key == "missing_score"
    ));
    assert!(matches!(
        &issues[2],
        MetadataError::InvalidFilterOperator { key, operator, .. }
            if key == "active" && operator == &"gt"
    ));
}

#[test]
fn test_schema_validate_filter_collects_all_set_value_issues() {
    let schema = MetadataSchema::builder()
        .required("status", DataType::String)
        .build();

    let error = MetadataFilter::builder()
        .in_set("status", [1_i64, 2_i64])
        .build_checked(&schema)
        .unwrap_err();
    let issues = error.issues();

    assert_eq!(issues.len(), 2);
    for issue in issues {
        assert!(matches!(
            issue,
            MetadataError::InvalidFilterOperator {
                key,
                operator,
                data_type,
                ..
            } if key == "status" && operator == &"in_set" && *data_type == DataType::String
        ));
    }
}

#[test]
fn test_schema_validate_filter_reports_unknown_set_field_once() {
    let schema = MetadataSchema::builder()
        .required("status", DataType::String)
        .build();

    let error = MetadataFilter::builder()
        .in_set("missing", [1_i64, 2_i64])
        .build_checked(&schema)
        .unwrap_err();
    let issues = error.issues();

    assert_eq!(
        issues,
        [MetadataError::UnknownFilterField {
            key: "missing".to_string(),
        }]
    );
}

#[test]
fn test_schema_validate_filter_rejects_range_on_bool() {
    let schema = MetadataSchema::builder()
        .required("active", DataType::Bool)
        .build();
    let error = single_issue(
        MetadataFilter::builder()
            .gt("active", true)
            .build_checked(&schema)
            .unwrap_err(),
    );

    match error {
        MetadataError::InvalidFilterOperator {
            key,
            operator,
            data_type,
            ..
        } => {
            assert_eq!(key, "active");
            assert_eq!(operator, "gt");
            assert_eq!(data_type, DataType::Bool);
        }
        other => panic!("expected InvalidFilterOperator, got {other:?}"),
    }
}

#[test]
#[cfg(feature = "big-decimal")]
fn test_schema_validate_filter_accepts_exact_big_decimal_float_comparison() {
    let schema = MetadataSchema::builder()
        .required("amount", DataType::BigDecimal)
        .build();

    MetadataFilter::builder()
        .ge("amount", 1.5_f64)
        .build_checked(&schema)
        .unwrap();

    MetadataFilter::builder()
        .ge("amount", 1.5_f64)
        .numeric_comparison_policy(NumericComparisonPolicy::Approximate)
        .build_checked(&schema)
        .unwrap();

    let error = single_issue(
        MetadataFilter::builder()
            .ge("amount", f64::NAN)
            .build_checked(&schema)
            .unwrap_err(),
    );
    assert!(matches!(error, MetadataError::InvalidFilterOperator { .. }));
}

#[test]
fn test_schema_validate_filter_accepts_all_non_nan_numeric_pairs() {
    let schema = MetadataSchema::builder()
        .required("signed", DataType::Int64)
        .required("unsigned", DataType::UInt64)
        .required("float", DataType::Float64)
        .build();

    MetadataFilter::builder()
        .ge("signed", 10.0_f64)
        .and_ge("unsigned", 10.0_f64)
        .and_ge("float", 10_i64)
        .build_checked(&schema)
        .unwrap();

    MetadataFilter::builder()
        .ge("signed", 1.5_f64)
        .and_ge("float", 9_007_199_254_740_993_i64)
        .and_ge("float", u128::MAX)
        .build_checked(&schema)
        .unwrap();
}
