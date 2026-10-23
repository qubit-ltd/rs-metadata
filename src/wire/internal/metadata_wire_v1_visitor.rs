// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Visitor that decodes the strict metadata v1 wire envelope.

use std::fmt;

use serde::de::DeserializeSeed;
use serde::de::Error as DeError;
use serde::de::MapAccess;
use serde::de::Visitor;

use super::super::metadata_wire_v1::MetadataWireV1;

/// Visits the strict metadata envelope while delegating values decoding.
///
/// # Type Parameters
///
/// `S` is the seed that decodes the envelope's `values` field.
pub(in crate::wire) struct MetadataWireV1Visitor<S> {
    /// Seed used to decode the envelope's values map, if it has not been
    /// consumed yet.
    pub(in crate::wire) values: Option<S>,
}

impl<'de, S> Visitor<'de> for MetadataWireV1Visitor<S>
where
    S: DeserializeSeed<'de>,
{
    type Value = MetadataWireV1<S::Value>;

    /// Describes the expected strict metadata envelope shape.
    ///
    /// # Parameters
    ///
    /// * `formatter` — formatter receiving the human-readable description.
    ///
    /// # Returns
    ///
    /// A formatting result indicating whether the description was written.
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a strict metadata V1 envelope")
    }

    /// Decodes the version and values fields, rejecting duplicate, missing, or
    /// unknown fields.
    ///
    /// # Type Parameters
    ///
    /// * `A` — map access implementation supplying the envelope fields.
    ///
    /// # Parameters
    ///
    /// * `map` — access to the serialized envelope fields.
    ///
    /// # Returns
    ///
    /// The decoded envelope with its version and values.
    ///
    /// # Errors
    ///
    /// Returns the map access error if a field cannot be decoded, a field is
    /// duplicated or missing, or an unknown field is encountered.
    ///
    /// # Panics
    ///
    /// Panics if the values seed was already consumed before the `values`
    /// field is decoded.
    fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut version = None;
        let mut values = None;
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "version" => {
                    if version.is_some() {
                        return Err(DeError::duplicate_field("version"));
                    }
                    version = Some(map.next_value::<u8>()?);
                }
                "values" => {
                    if values.is_some() {
                        return Err(DeError::duplicate_field("values"));
                    }
                    let seed = self.values.take().expect("metadata values seed is consumed once");
                    values = Some(map.next_value_seed(seed)?);
                }
                _ => {
                    return Err(DeError::unknown_field(&key, &["version", "values"]));
                }
            }
        }
        Ok(MetadataWireV1 {
            version: version.ok_or_else(|| DeError::missing_field("version"))?,
            values: values.ok_or_else(|| DeError::missing_field("values"))?,
        })
    }
}
