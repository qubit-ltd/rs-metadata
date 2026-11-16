// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::MetadataValidationError`].

#![cfg(feature = "schema")]

use std::fmt::Write;

use qubit_datatype::DataType;
use qubit_metadata::MetadataError;
use qubit_metadata::MetadataValidationError;
use support::counting_writer::CountingWriter;

mod support;

#[test]
fn test_validation_error_exposes_collected_issues() {
    let issues = vec![
        MetadataError::MissingRequiredField {
            key: "id".to_string(),
            expected: DataType::String,
        },
        MetadataError::UnknownField {
            key: "extra".to_string(),
        },
    ];

    let error = MetadataValidationError::from_issues(issues.clone()).unwrap();

    assert_eq!(error.issues(), issues.as_slice());
    assert_eq!(error.len(), 2);
    assert_eq!(
        error.to_string(),
        "2 metadata validation issue(s): [MissingRequiredField { key: \"id\", expected: String }, UnknownField { key: \"extra\" }]"
    );
    assert!(std::error::Error::source(&error).is_none());
    assert_eq!(error.into_issues(), issues);
    assert!(MetadataValidationError::from_issues(Vec::new()).is_none());
}

#[test]
fn test_validation_error_can_wrap_single_issue() {
    let issue = MetadataError::UnknownFilterField {
        key: "missing".to_string(),
    };

    let error = MetadataValidationError::from_issue(issue.clone());

    assert_eq!(error.issues(), std::slice::from_ref(&issue));
    assert_eq!(error.len(), 1);
    assert_eq!(error.into_issues(), vec![issue]);
    assert_eq!(
        MetadataValidationError::from_issue(MetadataError::UnknownFilterField {
            key: "missing".to_string(),
        })
        .to_string(),
        "1 metadata validation issue(s): [UnknownFilterField { key: \"missing\" }]"
    );
}

#[test]
fn test_validation_error_display_streams_formatter_writes() {
    let error = MetadataValidationError::from_issue(MetadataError::UnknownField {
        key: "extra".to_string(),
    });
    let mut writer = CountingWriter::new(None);

    write!(&mut writer, "{error}").unwrap();

    assert!(writer.write_count() > 1);
}

#[test]
fn test_validation_error_display_propagates_formatter_errors() {
    let error = MetadataValidationError::from_issue(MetadataError::UnknownField {
        key: "extra".to_string(),
    });
    let mut writer = CountingWriter::new(Some(2));

    assert!(write!(&mut writer, "{error}").is_err());
}

#[test]
fn test_validation_diagnostics_have_a_cumulative_bound() {
    let issue = MetadataError::UnknownField { key: "x".repeat(256) };
    let error = MetadataValidationError::from_issues(vec![issue; 32_640]).unwrap();
    for text in [error.to_string(), format!("{error:?}")] {
        assert!(
            text.len() < 16_384,
            "aggregate diagnostic exceeded its budget: {}",
            text.len()
        );
        assert!(text.contains("incomplete"), "truncation must be explicit");
    }
    assert_eq!(error.issues().len(), 32_640);
}

#[test]
fn test_validation_diagnostics_bound_one_large_issue() {
    let error = MetadataValidationError::from_issue(MetadataError::UnknownField {
        key: "secret\n".repeat(100_000),
    });
    assert!(error.to_string().len() < 16_384);
    assert!(format!("{error:?}").contains("incomplete"));
    assert_eq!(error.issues().len(), 1);
}

#[test]
fn test_validation_diagnostics_share_the_callers_collection_budget() {
    let error = MetadataValidationError::from_issues(vec![
        MetadataError::UnknownField {
            key: "example".to_owned()
        };
        10
    ])
    .unwrap();
    let policy = qubit_redact::RedactionPolicy::builder()
        .limits(|limits| {
            limits.max_collection_items(1);
        })
        .unwrap()
        .build()
        .unwrap();
    let output = qubit_redact::Redactor::new(policy).redact_text(&error);
    assert!(output.text_or_marker("incomplete").contains("incomplete"));
    assert_eq!(error.into_issues().len(), 10);
}
