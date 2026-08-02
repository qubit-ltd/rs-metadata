// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Resource categories bounded while decoding metadata JSON wire envelopes.

/// Identifies one decoded metadata wire resource bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MetadataWireLimitKind {
    /// Metadata entries in one decoded metadata object.
    MetadataEntries,
    /// Field definitions in one decoded metadata schema.
    SchemaFields,
    /// UTF-8 bytes in one decoded metadata or schema key.
    KeyBytes,
}
