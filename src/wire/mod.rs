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
#[cfg(all(feature = "json", feature = "schema"))]
pub(crate) use metadata_schema_wire_v1::MetadataSchemaWireV1Seed;
pub(crate) use metadata_wire_v1::{
    METADATA_WIRE_VERSION_V1,
    MetadataWireV1,
};
#[cfg(feature = "json")]
pub(crate) use metadata_wire_v1::MetadataWireV1Seed;
#[cfg(feature = "json")]
pub(crate) use strict_string_map::{
    STRICT_STRING_MAP_ENTRY_LIMIT_MARKER,
    STRICT_STRING_MAP_MAX_ENTRIES,
    STRICT_STRING_MAP_MAX_KEY_BYTES,
    STRICT_STRING_MAP_KEY_LIMIT_MARKER,
};
pub(crate) use strict_string_map::{
    StrictStringMap,
};
#[cfg(feature = "json")]
pub(crate) use strict_string_map::StrictStringMapSeed;
