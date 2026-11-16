// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Categories of configurable metadata-domain limits.
//!
//! # Examples
//!
//! ```
//! use qubit_metadata::MetadataError;
//! use qubit_metadata::MetadataLimitKind;
//! use qubit_metadata::MetadataLimits;
//!
//! let error = MetadataLimits::builder()
//!     .max_schema_fields(usize::MAX)
//!     .build()
//!     .unwrap_err();
//! assert!(matches!(error, MetadataError::InvalidMetadataLimit {
//!     kind: MetadataLimitKind::SchemaFields,
//!     value: usize::MAX,
//!     maximum: 4_096,
//! }));
//! ```

/// Metadata-domain limit configured above its protocol maximum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MetadataLimitKind {
    /// Maximum number of entries in a metadata object.
    MetadataEntries,
    /// Maximum number of fields in a metadata schema.
    SchemaFields,
    /// Maximum UTF-8 byte length of a metadata or schema key.
    KeyBytes,
}
