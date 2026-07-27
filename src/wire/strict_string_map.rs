// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! A strict string-keyed map for wire-format decoding.

use std::{
    collections::BTreeMap,
    fmt,
};

use serde::{
    Deserialize,
    de::{
        self,
        MapAccess,
        Visitor,
    },
};

/// A string-keyed map that rejects duplicate keys while deserializing.
pub(crate) struct StrictStringMap<V>(BTreeMap<String, V>);

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
        struct StrictStringMapVisitor<V>(std::marker::PhantomData<V>);

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

            /// Decodes all entries and rejects the first duplicate key.
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut values = BTreeMap::new();
                while let Some((key, value)) = map.next_entry::<String, V>()? {
                    if values.insert(key.clone(), value).is_some() {
                        return Err(de::Error::custom(format!(
                            "duplicate map key '{key}'"
                        )));
                    }
                }
                Ok(StrictStringMap(values))
            }
        }

        deserializer
            .deserialize_map(StrictStringMapVisitor(std::marker::PhantomData))
    }
}
