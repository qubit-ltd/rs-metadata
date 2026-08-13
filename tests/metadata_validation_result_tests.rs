// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::MetadataValidationResult`].

#![cfg(feature = "schema")]

use qubit_metadata::{
    MetadataError,
    MetadataValidationError,
    MetadataValidationResult,
};

#[test]
fn test_metadata_validation_result_preserves_aggregate_error() {
    let error = MetadataValidationError::from_issue(
        MetadataError::MissingFilterExpression,
    );
    let result: MetadataValidationResult<()> = Err(error.clone());

    assert_eq!(result, Err(error));
}
