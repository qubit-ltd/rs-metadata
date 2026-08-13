// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Seed decoder for bounded strict string-keyed maps.
// qubit-style: allow multiple-public-types

use std::fmt;

use serde::de::{
    self,
    DeserializeSeed,
    MapAccess,
    Visitor,
};

use super::strict_string_map::StrictStringMap;

/// Deserializes a strict string map with caller-provided resource bounds.
pub(crate) struct StrictStringMapSeed<'a, V> {
    max_entries: usize,
    max_key_bytes: usize,
    lifetime: std::marker::PhantomData<&'a ()>,
    marker: std::marker::PhantomData<fn() -> V>,
}

impl<V> StrictStringMapSeed<'static, V> {
    /// Creates a bounded strict-map seed.
    pub(crate) const fn new(max_entries: usize, max_key_bytes: usize) -> Self {
        Self {
            max_entries,
            max_key_bytes,
            lifetime: std::marker::PhantomData,
            marker: std::marker::PhantomData,
        }
    }
}

impl<'de, 'a, V> DeserializeSeed<'de> for StrictStringMapSeed<'a, V>
where
    V: serde::Deserialize<'de>,
{
    type Value = StrictStringMap<V>;

    /// Deserializes a map while enforcing caller-provided bounds.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let max_entries = self.max_entries;
        let max_key_bytes = self.max_key_bytes;
        struct StrictStringMapVisitor<'a, V> {
            max_entries: usize,
            max_key_bytes: usize,
            lifetime: std::marker::PhantomData<&'a ()>,
            marker: std::marker::PhantomData<fn() -> V>,
        }

        impl<'de, 'a, V> Visitor<'de> for StrictStringMapVisitor<'a, V>
        where
            V: serde::Deserialize<'de>,
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
                let mut values = std::collections::BTreeMap::new();
                while let Some(key) = map.next_key::<String>()? {
                    if key.len() > self.max_key_bytes {
                        return Err(de::Error::custom(format!(
                            "map key has {} bytes, exceeding the limit of {} bytes",
                            key.len(),
                            self.max_key_bytes,
                        )));
                    }
                    if values.len() >= self.max_entries {
                        return Err(de::Error::custom(format!(
                            "map has more than the limit of {} entries",
                            self.max_entries,
                        )));
                    }
                    match values.entry(key) {
                        std::collections::btree_map::Entry::Occupied(entry) => {
                            return Err(de::Error::custom(format!(
                                "duplicate map key '{}'",
                                entry.key()
                            )));
                        }
                        std::collections::btree_map::Entry::Vacant(entry) => {
                            let value = map.next_value::<V>()?;
                            entry.insert(value);
                        }
                    }
                }
                Ok(StrictStringMap(values))
            }
        }

        deserializer.deserialize_map(StrictStringMapVisitor {
            max_entries,
            max_key_bytes,
            lifetime: std::marker::PhantomData,
            marker: std::marker::PhantomData,
        })
    }
}
