// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::MetadataValidationError`].

use std::fmt::{
    self,
    Write,
};

use qubit_datatype::DataType;
use qubit_metadata::{
    MetadataError,
    MetadataValidationError,
};

/// Writer that counts formatting calls and can fail at a selected call.
struct CountingWriter {
    /// Number of completed or attempted writes.
    write_count: usize,
    /// Optional one-based write index that returns [`fmt::Error`].
    fail_on_write: Option<usize>,
}

impl CountingWriter {
    /// Creates a writer that optionally fails on `fail_on_write`.
    ///
    /// # Parameters
    ///
    /// * `fail_on_write` - Optional one-based write index to fail.
    ///
    /// # Returns
    ///
    /// A new counting writer.
    #[inline(always)]
    fn new(fail_on_write: Option<usize>) -> Self {
        Self {
            write_count: 0,
            fail_on_write,
        }
    }
}

impl Write for CountingWriter {
    fn write_str(&mut self, _: &str) -> fmt::Result {
        self.write_count += 1;
        if self.fail_on_write == Some(self.write_count) {
            return Err(fmt::Error);
        }
        Ok(())
    }
}

#[test]
fn validation_error_exposes_collected_issues() {
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
    assert!(!error.is_empty());
    assert_eq!(
        error.to_string(),
        "2 metadata validation issue(s); 1: Required metadata key 'id' is missing (expected string); 2: Metadata key 'extra' is not defined in schema"
    );
    assert!(std::error::Error::source(&error).is_none());
    assert_eq!(error.into_issues(), issues);
    assert!(MetadataValidationError::from_issues(Vec::new()).is_none());
}

#[test]
fn validation_error_can_wrap_single_issue() {
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
fn validation_error_display_streams_formatter_writes() {
    let error =
        MetadataValidationError::from_issue(MetadataError::UnknownField {
            key: "extra".to_string(),
        });
    let mut writer = CountingWriter::new(None);

    write!(&mut writer, "{error}").unwrap();

    assert!(writer.write_count > 1);
}

#[test]
fn validation_error_display_propagates_formatter_errors() {
    let error =
        MetadataValidationError::from_issue(MetadataError::UnknownField {
            key: "extra".to_string(),
        });
    let mut writer = CountingWriter::new(Some(2));

    assert!(write!(&mut writer, "{error}").is_err());
}
