// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Borrowed value-map adapter for the strict metadata V1 wire format.

use std::collections::BTreeMap;

use qubit_value::Value;
use qubit_value::ValueWirePayloadRefV1;
use serde::Serialize;
use serde::Serializer;
use serde::ser::Error as SerError;
use serde::ser::SerializeMap;

/// Borrowed map adapter that validates and serializes V1 value payloads.
pub(crate) struct MetadataWireValuesRef<'a>(
    /// Metadata values borrowed for strict V1 wire serialization.
    pub(crate) &'a BTreeMap<String, Value>,
);

impl Serialize for MetadataWireValuesRef<'_> {
    /// Serializes each value as a validated borrowed V1 payload.
    ///
    /// # Type Parameters
    ///
    /// * `S` — the serializer implementation.
    ///
    /// # Parameters
    ///
    /// * `serializer` — the destination receiving the metadata map.
    ///
    /// # Returns
    ///
    /// The serializer's result after writing all metadata entries.
    ///
    /// # Errors
    ///
    /// Returns the serializer's error if a value cannot be represented as a
    /// V1 payload or if serializing the map fails.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut values = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in self.0 {
            let payload = ValueWirePayloadRefV1::try_from(value).map_err(<S::Error as SerError>::custom)?;
            values.serialize_entry(key, &payload)?;
        }
        values.end()
    }
}
