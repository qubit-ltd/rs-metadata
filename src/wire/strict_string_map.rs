// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! A strict string-keyed map for wire-format decoding.

use std::{collections::btree_map::Entry, collections::BTreeMap, fmt};

use serde::{
    de::DeserializeSeed,
    de::{self, MapAccess, Visitor},
    Deserialize,
};

/// Hard maximum number of entries accepted by strict metadata maps.
pub(crate) const STRICT_STRING_MAP_MAX_ENTRIES: usize = 4_096;

/// Hard maximum UTF-8 byte length accepted for strict metadata map keys.
pub(crate) const STRICT_STRING_MAP_MAX_KEY_BYTES: usize = 256;

/// Marker used to preserve structured map-entry limit errors across Serde.
pub(crate) const STRICT_STRING_MAP_ENTRY_LIMIT_MARKER: &str =
    "qubit_metadata_map_entry_limit:";

/// Marker used to preserve structured map-key limit errors across Serde.
pub(crate) const STRICT_STRING_MAP_KEY_LIMIT_MARKER: &str =
    "qubit_metadata_map_key_limit:";

/// A string-keyed map that rejects duplicate keys while deserializing.
pub(crate) struct StrictStringMap<V>(BTreeMap<String, V>);

/// Deserializes a strict string map with caller-provided resource bounds.
pub(crate) struct StrictStringMapSeed<V> {
    max_entries: usize,
    max_key_bytes: usize,
    marker: std::marker::PhantomData<fn() -> V>,
}

impl<V> StrictStringMapSeed<V> {
    /// Creates a bounded strict-map seed.
    pub(crate) const fn new(max_entries: usize, max_key_bytes: usize) -> Self {
        Self {
            max_entries,
            max_key_bytes,
            marker: std::marker::PhantomData,
        }
    }
}

impl<V> StrictStringMap<V> {
    /// Returns the decoded entries.
    ///
    /// # Returns
    ///
    /// The complete map after duplicate-key validation.
    pub(crate) fn into_inner(self) -> BTreeMap<String, V> {
        self.0
    }
}

impl<'de, V> Deserialize<'de> for StrictStringMap<V>
where
    V: Deserialize<'de>,
{
    /// Deserializes a map while rejecting a repeated key.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        StrictStringMapSeed::new(
            STRICT_STRING_MAP_MAX_ENTRIES,
            STRICT_STRING_MAP_MAX_KEY_BYTES,
        )
        .deserialize(deserializer)
    }
}

impl<'de, V> DeserializeSeed<'de> for StrictStringMapSeed<V>
where
    V: Deserialize<'de>,
{
    type Value = StrictStringMap<V>;

    /// Deserializes a map while enforcing caller-provided bounds.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StrictStringMapVisitor<V> {
            max_entries: usize,
            max_key_bytes: usize,
            marker: std::marker::PhantomData<fn() -> V>,
        }

        impl<'de, V> Visitor<'de> for StrictStringMapVisitor<V>
        where
            V: Deserialize<'de>,
        {
            type Value = StrictStringMap<V>;

            /// Describes the expected input shape.
            fn expecting(
                &self,
                formatter: &mut fmt::Formatter<'_>,
            ) -> fmt::Result {
                formatter.write_str("a map with unique string keys")
            }

            /// Decodes entries until a duplicate or resource limit is found.
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut values = BTreeMap::new();
                while let Some(key) = map.next_key::<String>()? {
                    if key.len() > self.max_key_bytes {
                        return Err(de::Error::custom(format!(
                            "{}{}:{} (map key has {} bytes, exceeding the limit of {} bytes)",
                            STRICT_STRING_MAP_KEY_LIMIT_MARKER,
                            key.len(),
                            self.max_key_bytes,
                            key.len(),
                            self.max_key_bytes,
                        )));
                    }
                    if values.len() >= self.max_entries {
                        return Err(de::Error::custom(format!(
                            "{}{} (map has more than the limit of {} entries)",
                            STRICT_STRING_MAP_ENTRY_LIMIT_MARKER,
                            self.max_entries,
                            self.max_entries,
                        )));
                    }
                    match values.entry(key) {
                        Entry::Occupied(entry) => {
                            return Err(de::Error::custom(format!(
                                "duplicate map key '{}'",
                                entry.key()
                            )));
                        }
                        Entry::Vacant(entry) => {
                            let value = map.next_value::<V>()?;
                            entry.insert(value);
                        }
                    }
                }
                Ok(StrictStringMap(values))
            }
        }

        deserializer.deserialize_map(StrictStringMapVisitor {
            max_entries: self.max_entries,
            max_key_bytes: self.max_key_bytes,
            marker: std::marker::PhantomData,
        })
    }
}
