/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! [`MetadataResult`] — result alias for explicit `Metadata` operations.

use crate::metadata_error::MetadataError;

/// Result type used by explicit `Metadata` operations that report failure
/// reasons instead of collapsing them into `None`.
pub type MetadataResult<T> = Result<T, MetadataError>;
