// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Seed decoder for bounded strict string-keyed maps.
// qubit-style: allow multiple-public-types

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use serde::Deserialize;
use serde::de;
use serde::de::DeserializeSeed;
use serde::de::Deserializer;
use serde::de::MapAccess;
use serde::de::Visitor;

use super::strict_string_map::StrictStringMap;
use crate::MetadataError;
use crate::MetadataWireLimitKind;

/// Deserializes a strict string map with caller-provided resource bounds.
pub(crate) struct StrictStringMapSeed<'a, V> {
    max_entries: usize,
    max_key_bytes: usize,
    error_slot: Option<Rc<RefCell<Option<MetadataError>>>>,
    lifetime: std::marker::PhantomData<&'a ()>,
    marker: std::marker::PhantomData<fn() -> V>,
}

/// Deserializes a strict string map with a caller-provided value seed.
pub(crate) struct StrictStringMapValueSeed<S> {
    max_entries: usize,
    max_key_bytes: usize,
    error_slot: Option<Rc<RefCell<Option<MetadataError>>>>,
    value_seed: S,
}

impl<S> StrictStringMapValueSeed<S> {
    /// Attaches a request-local slot retaining value-free domain failures.
    #[cfg(feature = "json")]
    pub(crate) fn with_error_slot(mut self, slot: Rc<RefCell<Option<MetadataError>>>) -> Self {
        self.error_slot = Some(slot);
        self
    }

    /// Creates a bounded strict-map seed with explicit value construction.
    pub(crate) const fn new(max_entries: usize, max_key_bytes: usize, value_seed: S) -> Self {
        Self {
            max_entries,
            max_key_bytes,
            error_slot: None,
            value_seed,
        }
    }
}

impl<'de, S> DeserializeSeed<'de> for StrictStringMapValueSeed<S>
where
    S: DeserializeSeed<'de> + Clone,
{
    type Value = StrictStringMap<S::Value>;

    /// Deserializes a map while applying the supplied seed to every value.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StrictStringMapValueVisitor<S> {
            max_entries: usize,
            max_key_bytes: usize,
            error_slot: Option<Rc<RefCell<Option<MetadataError>>>>,
            value_seed: S,
        }

        impl<'de, S> Visitor<'de> for StrictStringMapValueVisitor<S>
        where
            S: DeserializeSeed<'de> + Clone,
        {
            type Value = StrictStringMap<S::Value>;

            /// Describes the expected input shape.
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a map with unique string keys")
            }

            /// Decodes entries while enforcing key, count, and value bounds.
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut values = std::collections::BTreeMap::new();
                while let Some(key) = map.next_key::<String>()? {
                    if key.len() > self.max_key_bytes {
                        if let Some(slot) = &self.error_slot {
                            *slot.borrow_mut() = Some(MetadataError::WireLimitExceeded {
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
                        if let Some(slot) = &self.error_slot {
                            *slot.borrow_mut() = Some(MetadataError::WireLimitExceeded {
                                kind: MetadataWireLimitKind::Entries,
                                value: values.len() + 1,
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
                            return Err(de::Error::custom(format!("duplicate map key '{}'", entry.key())));
                        }
                        std::collections::btree_map::Entry::Vacant(entry) => {
                            entry.insert(map.next_value_seed(self.value_seed.clone())?);
                        }
                    }
                }
                Ok(StrictStringMap(values))
            }
        }

        deserializer.deserialize_map(StrictStringMapValueVisitor {
            max_entries: self.max_entries,
            max_key_bytes: self.max_key_bytes,
            error_slot: self.error_slot,
            value_seed: self.value_seed,
        })
    }
}

impl<V> StrictStringMapSeed<'static, V> {
    /// Attaches a request-local slot retaining value-free domain failures.
    #[cfg(all(feature = "json", feature = "schema"))]
    pub(crate) fn with_error_slot(mut self, slot: Rc<RefCell<Option<MetadataError>>>) -> Self {
        self.error_slot = Some(slot);
        self
    }

    /// Creates a bounded strict-map seed.
    pub(crate) const fn new(max_entries: usize, max_key_bytes: usize) -> Self {
        Self {
            max_entries,
            max_key_bytes,
            error_slot: None,
            lifetime: std::marker::PhantomData,
            marker: std::marker::PhantomData,
        }
    }
}

impl<'de, 'a, V> DeserializeSeed<'de> for StrictStringMapSeed<'a, V>
where
    V: Deserialize<'de>,
{
    type Value = StrictStringMap<V>;

    /// Deserializes a map while enforcing caller-provided bounds.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let max_entries = self.max_entries;
        let max_key_bytes = self.max_key_bytes;
        struct StrictStringMapVisitor<'a, V> {
            max_entries: usize,
            max_key_bytes: usize,
            error_slot: Option<Rc<RefCell<Option<MetadataError>>>>,
            lifetime: std::marker::PhantomData<&'a ()>,
            marker: std::marker::PhantomData<fn() -> V>,
        }

        impl<'de, 'a, V> Visitor<'de> for StrictStringMapVisitor<'a, V>
        where
            V: Deserialize<'de>,
        {
            type Value = StrictStringMap<V>;

            /// Describes the expected input shape.
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
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
                        if let Some(slot) = &self.error_slot {
                            *slot.borrow_mut() = Some(MetadataError::WireLimitExceeded {
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
                        if let Some(slot) = &self.error_slot {
                            *slot.borrow_mut() = Some(MetadataError::WireLimitExceeded {
                                kind: MetadataWireLimitKind::Entries,
                                value: values.len() + 1,
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
                            return Err(de::Error::custom(format!("duplicate map key '{}'", entry.key())));
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
            error_slot: self.error_slot,
            lifetime: std::marker::PhantomData,
            marker: std::marker::PhantomData,
        })
    }
}
