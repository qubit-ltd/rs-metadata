// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Seed decoder for the strict metadata v1 wire envelope.

use serde::de::DeserializeSeed;
use serde::de::Deserializer;

use super::internal::MetadataWireV1Visitor;
use super::metadata_wire_v1::MetadataWireV1;

/// Deserializes a metadata envelope with a bounded values seed.
///
/// # Type Parameters
///
/// * `S` — the seed that decodes the envelope's values map.
pub(crate) struct MetadataWireV1Seed<S> {
    /// Seed used to decode the values map.
    values: S,
}

impl<S> MetadataWireV1Seed<S> {
    /// Creates a seed that delegates values decoding to `values`.
    ///
    /// # Parameters
    ///
    /// * `values` — the seed used to decode the envelope's values map.
    ///
    /// # Returns
    ///
    /// A seed configured with `values`.
    #[must_use]
    #[inline]
    pub(crate) const fn new(values: S) -> Self {
        Self { values }
    }
}

impl<'de, S> DeserializeSeed<'de> for MetadataWireV1Seed<S>
where
    S: DeserializeSeed<'de>,
{
    /// Metadata envelope produced by the seed's values decoder.
    type Value = MetadataWireV1<S::Value>;

    /// Deserializes the strict envelope and delegates values to its seed.
    ///
    /// # Type Parameters
    ///
    /// * `D` — the deserializer providing the envelope data.
    ///
    /// # Parameters
    ///
    /// * `deserializer` — the source of the version and values fields.
    ///
    /// # Returns
    ///
    /// The decoded metadata envelope.
    ///
    /// # Errors
    ///
    /// Returns the deserializer's error for malformed, duplicate, missing, or
    /// unknown envelope fields, or when values decoding fails.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            "MetadataWireV1",
            &["version", "values"],
            MetadataWireV1Visitor {
                values: Some(self.values),
            },
        )
    }
}
