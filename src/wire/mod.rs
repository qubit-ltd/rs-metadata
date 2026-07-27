// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
//
// =============================================================================
//! Internal strict wire envelopes for core metadata types.

mod metadata_schema_wire;
mod metadata_wire;

pub(crate) use metadata_schema_wire::{
    METADATA_SCHEMA_WIRE_VERSION,
    MetadataSchemaWire,
};
pub(crate) use metadata_wire::{
    METADATA_WIRE_VERSION,
    MetadataWire,
    StrictStringMap,
};
