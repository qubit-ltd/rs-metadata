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
#[cfg(feature = "json")]
use crate::MetadataWireLimitKind;

/// Records a strict-map resource limit discovered during Serde decoding.
#[cfg(feature = "json")]
#[derive(Debug, Clone, Copy)]
pub(crate) struct StrictStringMapLimitExceeded {
    /// Resource category enforced by the caller.
    pub(crate) kind: MetadataWireLimitKind,
    /// Observed resource value.
    pub(crate) value: usize,
    /// Configured resource maximum.
    pub(crate) maximum: usize,
}

/// Deserializes a strict string map with caller-provided resource bounds.
pub(crate) struct StrictStringMapSeed<'a, V> {
    max_entries: usize,
    max_key_bytes: usize,
    #[cfg(feature = "json")]
    limit_kind: Option<MetadataWireLimitKind>,
    #[cfg(feature = "json")]
    limit_error: Option<&'a mut Option<StrictStringMapLimitExceeded>>,
    #[cfg(not(feature = "json"))]
    lifetime: std::marker::PhantomData<&'a ()>,
    marker: std::marker::PhantomData<fn() -> V>,
}

impl<V> StrictStringMapSeed<'static, V> {
    /// Creates a bounded strict-map seed.
    pub(crate) const fn new(max_entries: usize, max_key_bytes: usize) -> Self {
        Self {
            max_entries,
            max_key_bytes,
            #[cfg(feature = "json")]
            limit_kind: None,
            #[cfg(feature = "json")]
            limit_error: None,
            #[cfg(not(feature = "json"))]
            lifetime: std::marker::PhantomData,
            marker: std::marker::PhantomData,
        }
    }
}

#[cfg(feature = "json")]
impl<'a, V> StrictStringMapSeed<'a, V> {
    /// Creates a bounded seed that reports map limit failures to `limit_error`.
    pub(crate) fn with_limit_reporting(
        max_entries: usize,
        max_key_bytes: usize,
        limit_kind: MetadataWireLimitKind,
        limit_error: &'a mut Option<StrictStringMapLimitExceeded>,
    ) -> Self {
        Self {
            max_entries,
            max_key_bytes,
            limit_kind: Some(limit_kind),
            limit_error: Some(limit_error),
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
        struct StrictStringMapVisitor<'a, V> {
            max_entries: usize,
            max_key_bytes: usize,
            #[cfg(feature = "json")]
            limit_kind: Option<MetadataWireLimitKind>,
            #[cfg(feature = "json")]
            limit_error: Option<&'a mut Option<StrictStringMapLimitExceeded>>,
            #[cfg(not(feature = "json"))]
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
            #[cfg_attr(not(feature = "json"), allow(unused_mut))]
            fn visit_map<A>(
                mut self,
                mut map: A,
            ) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut values = std::collections::BTreeMap::new();
                while let Some(key) = map.next_key::<String>()? {
                    if key.len() > self.max_key_bytes {
                        #[cfg(feature = "json")]
                        if let Some(limit_error) =
                            self.limit_error.as_deref_mut()
                        {
                            *limit_error = Some(StrictStringMapLimitExceeded {
                                kind: MetadataWireLimitKind::KeyBytes,
                                value: key.len(),
                                maximum: self.max_key_bytes,
                            });
                        }
                        return Err(de::Error::custom(format!(
                            "map key has {} bytes, exceeding the limit of {} bytes",
                            key.len(),
                            self.max_key_bytes,
                        )));
                    }
                    if values.len() >= self.max_entries {
                        #[cfg(feature = "json")]
                        if let (Some(kind), Some(limit_error)) =
                            (self.limit_kind, self.limit_error.as_deref_mut())
                        {
                            *limit_error = Some(StrictStringMapLimitExceeded {
                                kind,
                                value: values.len().saturating_add(1),
                                maximum: self.max_entries,
                            });
                        }
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
            max_entries: self.max_entries,
            max_key_bytes: self.max_key_bytes,
            #[cfg(feature = "json")]
            limit_kind: self.limit_kind,
            #[cfg(feature = "json")]
            limit_error: self.limit_error,
            #[cfg(not(feature = "json"))]
            lifetime: std::marker::PhantomData,
            marker: std::marker::PhantomData,
        })
    }
}
