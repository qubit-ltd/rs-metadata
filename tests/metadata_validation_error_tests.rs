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

mod support;

use support::counting_writer::CountingWriter;

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
        "2 metadata validation issue(s); 1: Required metadata key 'id' is missing (expected string); 2: Metadata key 'extra' is not defined in schema"
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

    assert_eq!(error.issues(), &[issue]);
    assert_eq!(
        error.to_string(),
        "1 metadata validation issue(s); 1: Metadata filter references key 'missing' not defined in schema"
    );
}

#[test]
fn test_validation_error_display_streams_formatter_writes() {
    let error =
        MetadataValidationError::from_issue(MetadataError::UnknownField {
            key: "extra".to_string(),
        });
    let mut writer = CountingWriter::new(None);

    write!(&mut writer, "{error}").unwrap();

    assert!(writer.write_count() > 1);
}

#[test]
fn test_validation_error_display_propagates_formatter_errors() {
    let error =
        MetadataValidationError::from_issue(MetadataError::UnknownField {
            key: "extra".to_string(),
        });
    let mut writer = CountingWriter::new(Some(2));

    assert!(write!(&mut writer, "{error}").is_err());
}
