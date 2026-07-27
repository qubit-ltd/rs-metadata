// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! [`MetadataValidationResult`] — aggregate schema-validation result alias.

use crate::MetadataValidationError;

/// Result type for schema validation that may report multiple issues.
pub type MetadataValidationResult<T> = Result<T, MetadataValidationError>;
