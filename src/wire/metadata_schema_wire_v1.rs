// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Strict v1 wire envelope for [`crate::MetadataSchema`].

use serde::{Deserialize, Serialize};
#[cfg(feature = "json")]
use serde::de::{DeserializeSeed, MapAccess, Visitor};
#[cfg(feature = "json")]
use std::fmt;

use crate::{
    UnknownFilterFieldPolicy,
    UnknownMetadataFieldPolicy,
};

/// Current serialized metadata-schema format version.
pub(crate) const METADATA_SCHEMA_WIRE_VERSION_V1: u8 = 1;

/// Strict schema envelope with a generic borrowed or owned field map.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MetadataSchemaWireV1<F> {
    /// Wire-format version.
    pub(crate) version: u8,
    /// Borrowed or owned metadata field definitions.
    pub(crate) fields: F,
    /// Policy for metadata keys absent from the schema.
    pub(crate) unknown_metadata_field_policy: UnknownMetadataFieldPolicy,
    /// Policy for filter keys absent from the schema.
    pub(crate) unknown_filter_field_policy: UnknownFilterFieldPolicy,
}

/// Deserializes a schema envelope with a bounded fields seed.
#[cfg(feature = "json")]
pub(crate) struct MetadataSchemaWireV1Seed<S> {
    fields: S,
}

#[cfg(feature = "json")]
impl<S> MetadataSchemaWireV1Seed<S> {
    /// Creates a schema-envelope seed.
    pub(crate) const fn new(fields: S) -> Self {
        Self { fields }
    }
}

#[cfg(feature = "json")]
impl<'de, S> DeserializeSeed<'de> for MetadataSchemaWireV1Seed<S>
where
    S: DeserializeSeed<'de>,
{
    type Value = MetadataSchemaWireV1<S::Value>;

    /// Deserializes the strict schema envelope and delegates fields to its seed.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct MetadataSchemaWireVisitor<S> {
            fields: Option<S>,
        }

        impl<'de, S> Visitor<'de> for MetadataSchemaWireVisitor<S>
        where
            S: DeserializeSeed<'de>,
        {
            type Value = MetadataSchemaWireV1<S::Value>;

            /// Describes the strict metadata-schema envelope.
            fn expecting(
                &self,
                formatter: &mut fmt::Formatter<'_>,
            ) -> fmt::Result {
                formatter.write_str("a strict metadata schema V1 envelope")
            }

            /// Decodes each envelope field and rejects duplicates or unknowns.
            fn visit_map<A>(
                mut self,
                mut map: A,
            ) -> Result<Self::Value, A::Error>
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
                                return Err(serde::de::Error::duplicate_field(
                                    "version",
                                ));
                            }
                            version = Some(map.next_value::<u8>()?);
                        }
                        "fields" => {
                            if fields.is_some() {
                                return Err(serde::de::Error::duplicate_field(
                                    "fields",
                                ));
                            }
                            let seed = self.fields.take().ok_or_else(|| {
                                serde::de::Error::custom(
                                    "metadata schema fields seed was already consumed",
                                )
                            })?;
                            fields = Some(map.next_value_seed(seed)?);
                        }
                        "unknown_metadata_field_policy" => {
                            if unknown_metadata_field_policy.is_some() {
                                return Err(serde::de::Error::duplicate_field(
                                    "unknown_metadata_field_policy",
                                ));
                            }
                            unknown_metadata_field_policy = Some(
                                map.next_value::<UnknownMetadataFieldPolicy>()?,
                            );
                        }
                        "unknown_filter_field_policy" => {
                            if unknown_filter_field_policy.is_some() {
                                return Err(serde::de::Error::duplicate_field(
                                    "unknown_filter_field_policy",
                                ));
                            }
                            unknown_filter_field_policy = Some(
                                map.next_value::<UnknownFilterFieldPolicy>()?,
                            );
                        }
                        _ => {
                            return Err(serde::de::Error::unknown_field(
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
                    version: version.ok_or_else(|| {
                        serde::de::Error::missing_field("version")
                    })?,
                    fields: fields.ok_or_else(|| {
                        serde::de::Error::missing_field("fields")
                    })?,
                    unknown_metadata_field_policy:
                        unknown_metadata_field_policy.ok_or_else(|| {
                            serde::de::Error::missing_field(
                                "unknown_metadata_field_policy",
                            )
                        })?,
                    unknown_filter_field_policy:
                        unknown_filter_field_policy.ok_or_else(|| {
                            serde::de::Error::missing_field(
                                "unknown_filter_field_policy",
                            )
                        })?,
                })
            }
        }

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
