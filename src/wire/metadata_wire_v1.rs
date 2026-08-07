// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Strict v1 wire envelope for [`crate::Metadata`].

use serde::{
    Deserialize,
    Serialize,
};
#[cfg(feature = "json")]
use serde::de::{DeserializeSeed, MapAccess, Visitor};
#[cfg(feature = "json")]
use std::fmt;

/// Current serialized metadata format version.
pub(crate) const METADATA_WIRE_VERSION_V1: u8 = 1;

/// Strict metadata envelope with a generic borrowed or owned value map.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MetadataWireV1<T> {
    /// Wire-format version.
    pub(crate) version: u8,
    /// Borrowed or owned metadata values.
    pub(crate) values: T,
}

/// Deserializes a metadata envelope with a bounded values seed.
#[cfg(feature = "json")]
pub(crate) struct MetadataWireV1Seed<S> {
    values: S,
}

#[cfg(feature = "json")]
impl<S> MetadataWireV1Seed<S> {
    /// Creates a metadata-envelope seed.
    pub(crate) const fn new(values: S) -> Self {
        Self { values }
    }
}

#[cfg(feature = "json")]
impl<'de, S> DeserializeSeed<'de> for MetadataWireV1Seed<S>
where
    S: DeserializeSeed<'de>,
{
    type Value = MetadataWireV1<S::Value>;

    /// Deserializes the strict envelope and delegates values to its seed.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct MetadataWireVisitor<S> {
            values: Option<S>,
        }

        impl<'de, S> Visitor<'de> for MetadataWireVisitor<S>
        where
            S: DeserializeSeed<'de>,
        {
            type Value = MetadataWireV1<S::Value>;

            /// Describes the strict metadata envelope.
            fn expecting(
                &self,
                formatter: &mut fmt::Formatter<'_>,
            ) -> fmt::Result {
                formatter.write_str("a strict metadata V1 envelope")
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
                let mut values = None;
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
                        "values" => {
                            if values.is_some() {
                                return Err(serde::de::Error::duplicate_field(
                                    "values",
                                ));
                            }
                            let seed = self.values.take().ok_or_else(|| {
                                serde::de::Error::custom(
                                    "metadata values seed was already consumed",
                                )
                            })?;
                            values = Some(map.next_value_seed(seed)?);
                        }
                        _ => {
                            return Err(serde::de::Error::unknown_field(
                                &key,
                                &["version", "values"],
                            ));
                        }
                    }
                }
                Ok(MetadataWireV1 {
                    version: version.ok_or_else(|| {
                        serde::de::Error::missing_field("version")
                    })?,
                    values: values.ok_or_else(|| {
                        serde::de::Error::missing_field("values")
                    })?,
                })
            }
        }

        deserializer.deserialize_struct(
            "MetadataWireV1",
            &["version", "values"],
            MetadataWireVisitor {
                values: Some(self.values),
            },
        )
    }
}
