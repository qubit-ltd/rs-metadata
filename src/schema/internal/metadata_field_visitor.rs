// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Object-only deserialization for schema field definitions.

use std::fmt;

use qubit_datatype::DataType;
use serde::Deserializer;
use serde::de::IgnoredAny;
use serde::de::MapAccess;
use serde::de::Visitor;

/// Field names accepted by the metadata field object visitor.
const FIELD_NAMES: &[&str] = &["data_type", "required"];

/// Deserializes a metadata field's two required values from a strict object.
///
/// # Errors
///
/// Returns the deserializer error when the input is not an object, a required
/// field is missing, a field is duplicated or unknown, or a value is invalid.
pub(crate) fn deserialize_metadata_field<'de, D>(deserializer: D) -> Result<(DataType, bool), D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_struct("MetadataField", FIELD_NAMES, MetadataFieldVisitor)
}

/// Visits the object representation of a metadata field.
struct MetadataFieldVisitor;

impl<'de> Visitor<'de> for MetadataFieldVisitor {
    type Value = (DataType, bool);

    /// Describes the required wire shape in deserializer diagnostics.
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a metadata field object")
    }

    /// Reads exactly one `data_type` and one `required` member.
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut data_type = None;
        let mut required = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "data_type" => {
                    if data_type.is_some() {
                        return Err(serde::de::Error::duplicate_field("data_type"));
                    }
                    data_type = Some(map.next_value::<DataType>()?);
                }
                "required" => {
                    if required.is_some() {
                        return Err(serde::de::Error::duplicate_field("required"));
                    }
                    required = Some(map.next_value::<bool>()?);
                }
                _ => {
                    let _ = map.next_value::<IgnoredAny>()?;
                    return Err(serde::de::Error::unknown_field(&key, FIELD_NAMES));
                }
            }
        }

        let data_type = data_type.ok_or_else(|| serde::de::Error::missing_field("data_type"))?;
        let required = required.ok_or_else(|| serde::de::Error::missing_field("required"))?;
        Ok((data_type, required))
    }
}
