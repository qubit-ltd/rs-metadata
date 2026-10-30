// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Seed decoder for the strict metadata-schema v1 wire envelope.

use serde::de::DeserializeSeed;
use serde::de::Deserializer;

use super::internal::MetadataSchemaWireVisitor;
use super::metadata_schema_wire_v1::MetadataSchemaWireV1;

/// Deserializes a schema envelope with a bounded fields seed.
pub(crate) struct MetadataSchemaWireV1Seed<S> {
    /// Seed used to decode the schema fields map.
    fields: S,
}

impl<S> MetadataSchemaWireV1Seed<S> {
    /// Creates a schema-envelope seed.
    #[inline]
    pub(crate) const fn new(fields: S) -> Self {
        Self { fields }
    }
}

impl<'de, S> DeserializeSeed<'de> for MetadataSchemaWireV1Seed<S>
where
    S: DeserializeSeed<'de>,
{
    type Value = MetadataSchemaWireV1<S::Value>;

    /// Deserializes the strict schema envelope and delegates fields to its
    /// seed.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            "MetadataSchemaWireV1",
            &[
                "version",
                "fields",
                "unknown_metadata_field_policy",
                "unknown_filter_field_policy",
            ],
            MetadataSchemaWireVisitor {
                fields: Some(self.fields),
            },
        )
    }
}
