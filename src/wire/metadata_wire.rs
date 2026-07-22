// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
//! Strict v1 wire envelope for [`crate::Metadata`].

use serde::{
    Deserialize,
    Serialize,
};

/// Current serialized metadata format version.
pub(crate) const METADATA_WIRE_VERSION: u8 = 1;

/// Strict metadata envelope with a generic borrowed or owned value map.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MetadataWire<T> {
    /// Wire-format version.
    pub(crate) version: u8,
    /// Borrowed or owned metadata values.
    pub(crate) values: T,
}
