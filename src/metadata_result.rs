/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! [`MetadataResult`] — result alias for explicit `Metadata` operations.

use crate::metadata_error::MetadataError;
use crate::metadata_validation_error::MetadataValidationError;

/// Result type used by explicit `Metadata` operations that report failure
/// reasons instead of collapsing them into `None`.
pub type MetadataResult<T> = Result<T, MetadataError>;

/// Result type used by schema-level validation APIs that can report multiple
/// independent issues from one validation pass.
pub type MetadataValidationResult<T> = Result<T, MetadataValidationError>;
