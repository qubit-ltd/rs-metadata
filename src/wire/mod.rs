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
#[cfg(all(feature = "json", feature = "schema"))]
mod metadata_schema_wire_v1_seed;
mod metadata_wire_v1;
mod metadata_wire_v1_seed;
mod metadata_wire_values_ref;
mod strict_string_map;
mod strict_string_map_seed;

#[cfg(feature = "schema")]
pub(crate) use metadata_schema_wire_v1::METADATA_SCHEMA_WIRE_VERSION_V1;
#[cfg(feature = "schema")]
pub(crate) use metadata_schema_wire_v1::MetadataSchemaWireV1;
#[cfg(all(feature = "json", feature = "schema"))]
pub(crate) use metadata_schema_wire_v1_seed::MetadataSchemaWireV1Seed;
pub(crate) use metadata_wire_v1::METADATA_WIRE_VERSION_V1;
pub(crate) use metadata_wire_v1::MetadataWireV1;
pub(crate) use metadata_wire_v1_seed::MetadataWireV1Seed;
pub(crate) use metadata_wire_values_ref::MetadataWireValuesRef;
pub(crate) use strict_string_map::StrictStringMap;
#[cfg(feature = "json")]
pub(crate) use strict_string_map_seed::StrictStringMapSeed;
pub(crate) use strict_string_map_seed::StrictStringMapValueSeed;
