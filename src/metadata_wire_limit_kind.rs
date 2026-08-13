// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Resource categories used by the metadata V1 wire contract.

/// A metadata resource category bounded by the strict V1 wire contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MetadataWireLimitKind {
    /// Number of metadata entries or schema fields.
    Entries,
    /// UTF-8 byte length of a metadata or schema key.
    KeyBytes,
}
