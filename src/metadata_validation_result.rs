// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! [`MetadataValidationResult`] — aggregate schema-validation result alias.

use crate::MetadataValidationError;

/// Reports the outcome of schema validation that may collect multiple issues.
///
/// # Type Parameters
///
/// - `T`: The value produced when validation succeeds.
///
/// # Returns
///
/// `Ok(T)` contains the validated value. `Err(MetadataValidationError)`
/// contains the collected validation issues.
pub type MetadataValidationResult<T> = Result<T, MetadataValidationError>;
