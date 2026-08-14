// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Strict v1 wire envelope for [`crate::MetadataSchema`].

use serde::Deserialize;
use serde::Serialize;

use crate::UnknownFilterFieldPolicy;
use crate::UnknownMetadataFieldPolicy;

/// Current serialized metadata-schema format version.
pub(crate) const METADATA_SCHEMA_WIRE_VERSION_V1: u8 = 1;

/// Strict schema envelope with a generic borrowed or owned field map.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MetadataSchemaWireV1<F> {
    /// Wire-format version.
    pub(crate) version: u8,
    /// Borrowed or owned metadata field definitions.
    pub(crate) fields: F,
    /// Policy for metadata keys absent from the schema.
    pub(crate) unknown_metadata_field_policy: UnknownMetadataFieldPolicy,
    /// Policy for filter keys absent from the schema.
    pub(crate) unknown_filter_field_policy: UnknownFilterFieldPolicy,
}
