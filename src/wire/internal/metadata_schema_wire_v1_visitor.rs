// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Visitor for the strict metadata-schema V1 envelope.

use std::fmt;

use serde::de::DeserializeSeed;
use serde::de::Error as DeError;
use serde::de::MapAccess;
use serde::de::Visitor;

use super::super::metadata_schema_wire_v1::MetadataSchemaWireV1;
use crate::UnknownFilterFieldPolicy;
use crate::UnknownMetadataFieldPolicy;

/// Visits a schema envelope while delegating its fields map to one seed.
///
/// The envelope is a strict map with exactly the supported field names;
/// missing, duplicate, and unknown fields produce deserialization errors.
/// The fields-map seed is consumed only when the `fields` entry is first
/// decoded, and all deserialization errors are propagated to the caller.
///
/// # Type Parameters
///
/// * `S` - Seed that decodes the fields map under the caller's limits.
pub(in crate::wire) struct MetadataSchemaWireVisitor<S> {
    /// Fields-map seed, consumed only when the fields entry is first decoded.
    pub(in crate::wire) fields: Option<S>,
}

impl<'de, S> Visitor<'de> for MetadataSchemaWireVisitor<S>
where
    S: DeserializeSeed<'de>,
{
    type Value = MetadataSchemaWireV1<S::Value>;

    /// Describes the strict metadata-schema envelope.
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a strict metadata schema V1 envelope")
    }

    /// Decodes the envelope map and returns its schema value.
    ///
    /// Errors from key or value decoding, duplicate or unknown fields, and
    /// missing required fields are returned to the deserializer caller.
    fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut version = None;
        let mut fields = None;
        let mut unknown_metadata_field_policy = None;
        let mut unknown_filter_field_policy = None;
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "version" => {
                    if version.is_some() {
                        return Err(DeError::duplicate_field("version"));
                    }
                    version = Some(map.next_value::<u8>()?);
                }
                "fields" => {
                    if fields.is_some() {
                        return Err(DeError::duplicate_field("fields"));
                    }
                    let seed = self
                        .fields
                        .take()
                        .expect("metadata schema fields seed is consumed once");
                    fields = Some(map.next_value_seed(seed)?);
                }
                "unknown_metadata_field_policy" => {
                    if unknown_metadata_field_policy.is_some() {
                        return Err(DeError::duplicate_field("unknown_metadata_field_policy"));
                    }
                    unknown_metadata_field_policy = Some(map.next_value::<UnknownMetadataFieldPolicy>()?);
                }
                "unknown_filter_field_policy" => {
                    if unknown_filter_field_policy.is_some() {
                        return Err(DeError::duplicate_field("unknown_filter_field_policy"));
                    }
                    unknown_filter_field_policy = Some(map.next_value::<UnknownFilterFieldPolicy>()?);
                }
                _ => {
                    return Err(DeError::unknown_field(
                        &key,
                        &[
                            "version",
                            "fields",
                            "unknown_metadata_field_policy",
                            "unknown_filter_field_policy",
                        ],
                    ));
                }
            }
        }
        Ok(MetadataSchemaWireV1 {
            version: version.ok_or_else(|| DeError::missing_field("version"))?,
            fields: fields.ok_or_else(|| DeError::missing_field("fields"))?,
            unknown_metadata_field_policy: unknown_metadata_field_policy
                .ok_or_else(|| DeError::missing_field("unknown_metadata_field_policy"))?,
            unknown_filter_field_policy: unknown_filter_field_policy
                .ok_or_else(|| DeError::missing_field("unknown_filter_field_policy"))?,
        })
    }
}
