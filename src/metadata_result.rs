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

/// Result type used by explicit `Metadata` operations that report failure
/// reasons instead of collapsing them into `None`.
pub type MetadataResult<T> = Result<T, MetadataError>;
