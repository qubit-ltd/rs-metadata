// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Internal strict wire envelopes for core metadata types.

#[cfg(feature = "schema")]
mod metadata_schema_wire_v1;
mod metadata_wire_v1;
mod strict_string_map;

#[cfg(feature = "schema")]
pub(crate) use metadata_schema_wire_v1::{
    METADATA_SCHEMA_WIRE_VERSION_V1,
    MetadataSchemaWireV1,
};
pub(crate) use metadata_wire_v1::{
    METADATA_WIRE_VERSION_V1,
    MetadataWireV1,
};
pub(crate) use strict_string_map::StrictStringMap;
